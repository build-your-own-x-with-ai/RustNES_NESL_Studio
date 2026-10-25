//! Minimal NES "test machine": mapper-0 cartridge, RAM, a register-level PPU stub that
//! tracks VRAM/OAM writes, controllers, an APU write log and simple frame/NMI timing.

use crate::cpu::{Bus, Cpu};

/// CPU cycles per (NTSC) frame.
pub const CYCLES_PER_FRAME: u64 = 29781;
/// Extra CPU cycles charged for an OAM DMA.
pub const OAM_DMA_CYCLES: u64 = 513;
/// CPU cycles of vblank after each frame boundary (20 scanlines) in which VRAM may be written.
pub const VBLANK_CYCLES: u64 = 2273;

/// CPU cycle (relative to the frame boundary) at which the PPU draws pixel (x, y):
/// vblank, then the pre-render line, then 341 dots per line at 3 dots per CPU cycle.
pub fn pixel_cycle(x: u8, y: u8) -> u64 {
    VBLANK_CYCLES + (341 * (1 + y as u64) + x as u64) / 3
}

/// Rendered lines per frame that clock the MMC3 counter: the pre-render line and 240 visible.
const CLOCKED_LINES: u64 = 241;

/// Number of MMC3 scanline clocks (dot 260 of the pre-render line and each visible line)
/// that happened before absolute CPU cycle `c`.
fn scanline_clocks(c: u64) -> u64 {
    let (frame, rel) = (c / CYCLES_PER_FRAME, c % CYCLES_PER_FRAME);
    // clock k (0 = pre-render) is at rel cycle VBLANK_CYCLES + (341 * k + 260) / 3
    let in_frame = match (rel * 3).checked_sub(VBLANK_CYCLES * 3 + 260) {
        Some(0) | None => 0,
        Some(d) => (d.div_ceil(341)).min(CLOCKED_LINES),
    };
    frame * CLOCKED_LINES + in_frame
}

#[derive(Clone)]
pub struct NesBus {
    pub ram: [u8; 0x800],
    /// SRAM at $6000-$7FFF (8KB, battery-backed on real hardware).
    pub sram: [u8; 0x2000],
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub vram: Vec<u8>,
    pub oam: [u8; 256],
    pub ppuctrl: u8,
    pub ppumask: u8,
    /// Effective scroll: the PPU's `t` register as (x, y) pixels. Set by `$2005`, and
    /// also by `$2006` (a VRAM address write loads the coarse scroll and fine y of `t`).
    pub scroll: (u8, u8),
    /// Effective nametable select (bits 10-11 of `t`): the last `$2000` NT bits or the
    /// last `$2006` address bits, whichever was written most recently.
    pub nt_sel: u8,
    pub buttons: [u8; 2],
    pub apu_writes: Vec<(u16, u8)>,
    /// `$2006`/`$2007` writes made while rendering is on and outside vblank
    /// (these corrupt the picture on real hardware).
    pub render_conflicts: u64,
    /// CPU cycle at which the current instruction started (set by `TestNes::step`).
    pub now: u64,
    /// Absolute CPU cycle of this frame's sprite-0 hit, latched when rendering starts.
    pub sprite0_at: Option<u64>,
    /// Cycle at which the sprite-0 flag clears (the next pre-render line).
    pub sprite0_clear: u64,
    /// `(cycle, kind, value)` for every change of the effective render registers:
    /// kind 0 = PPUCTRL with its NT bits replaced by `nt_sel`, 1 = scroll x, 2 = scroll y.
    /// Logged on `$2000` / `$2005` writes and on `$2006` writes (which load `t` too).
    pub reg_log: Vec<(u64, u8, u8)>,
    /// Mapper 2 bus conflicts (write value != ROM byte).
    pub bus_conflicts: usize,
    /// Mapper 0 (NROM), 2 (UNROM), or 4 (MMC3).
    pub mapper: u8,
    /// Mapper 2: current PRG bank in $8000-$BFFF. Mapper 4: bank_data[6/7].
    pub prg_bank: usize,
    /// Total PRG banks.
    pub prg_banks: usize,
    /// Nametable mirroring from iNES flags6 bit 0: false = vertical (bit set), true = horizontal.
    pub(crate) horizontal: bool,
    // MMC3 (mapper 4) registers
    mmc3_bank_select: u8,
    mmc3_bank_data: [u8; 8],
    mmc3_irq_latch: u8,
    mmc3_irq_counter: u8,
    mmc3_irq_enabled: bool,
    mmc3_irq_reload: bool,
    /// MMC3 IRQ line (level): set when the counter reaches 0, cleared by a $E000 write.
    pub irq_pending: bool,
    /// Number of CHR banks (for mapper 4).
    chr_banks: usize,
    oam_addr: u8,
    write_toggle: bool,
    ppu_addr: u16,
    read_buffer: u8,
    shift: [u8; 2],
    reads: [u8; 2],
    strobe: bool,
    pending_dma_cycles: u64,
}

impl NesBus {
    /// Returns a copy of SRAM ($6000-$7FFF, 8KB).
    pub fn sram_snapshot(&self) -> Vec<u8> {
        self.sram.to_vec()
    }

    pub fn new(prg: Vec<u8>, chr: Vec<u8>) -> Self {
        NesBus {
            ram: [0; 0x800],
            sram: [0; 0x2000],
            prg,
            chr,
            vram: vec![0; 0x4000],
            oam: [0; 256],
            ppuctrl: 0,
            ppumask: 0,
            scroll: (0, 0),
            nt_sel: 0,
            buttons: [0; 2],
            apu_writes: Vec::new(),
            render_conflicts: 0,
            now: 0,
            sprite0_at: None,
            sprite0_clear: 0,
            reg_log: Vec::new(),
            bus_conflicts: 0,
            mapper: 0,
            prg_bank: 0,
            prg_banks: 1,
            horizontal: false,
            mmc3_bank_select: 0,
            mmc3_bank_data: [0; 8],
            mmc3_irq_latch: 0,
            mmc3_irq_counter: 0,
            mmc3_irq_enabled: false,
            mmc3_irq_reload: false,
            irq_pending: false,
            chr_banks: 0,
            oam_addr: 0,
            write_toggle: false,
            ppu_addr: 0,
            read_buffer: 0,
            shift: [0; 2],
            reads: [0; 2],
            strobe: false,
            pending_dma_cycles: 0,
        }
    }

    /// Returns the CHR byte at PPU address (with MMC3 banking if mapper 4).
    fn chr_read(&self, addr: u16) -> u8 {
        if self.mapper != 4 || self.chr.is_empty() {
            return self.vram[addr as usize];
        }
        // MMC3: 8 banks of 1KB each ($0000-$1FFF); bit 7 of bank_select swaps $0000/$1000 pattern tables
        let bank = match addr {
            0x0000..=0x03FF => {
                if self.mmc3_bank_select & 0x80 == 0 {
                    self.mmc3_bank_data[0] & 0xFE
                } else {
                    self.mmc3_bank_data[2]
                }
            }
            0x0400..=0x07FF => {
                if self.mmc3_bank_select & 0x80 == 0 {
                    self.mmc3_bank_data[0] | 1
                } else {
                    self.mmc3_bank_data[3]
                }
            }
            0x0800..=0x0BFF => {
                if self.mmc3_bank_select & 0x80 == 0 {
                    self.mmc3_bank_data[1] & 0xFE
                } else {
                    self.mmc3_bank_data[4]
                }
            }
            0x0C00..=0x0FFF => {
                if self.mmc3_bank_select & 0x80 == 0 {
                    self.mmc3_bank_data[1] | 1
                } else {
                    self.mmc3_bank_data[5]
                }
            }
            0x1000..=0x13FF => {
                if self.mmc3_bank_select & 0x80 == 0 {
                    self.mmc3_bank_data[2]
                } else {
                    self.mmc3_bank_data[0] & 0xFE
                }
            }
            0x1400..=0x17FF => {
                if self.mmc3_bank_select & 0x80 == 0 {
                    self.mmc3_bank_data[3]
                } else {
                    self.mmc3_bank_data[0] | 1
                }
            }
            0x1800..=0x1BFF => {
                if self.mmc3_bank_select & 0x80 == 0 {
                    self.mmc3_bank_data[4]
                } else {
                    self.mmc3_bank_data[1] & 0xFE
                }
            }
            0x1C00..=0x1FFF => {
                if self.mmc3_bank_select & 0x80 == 0 {
                    self.mmc3_bank_data[5]
                } else {
                    self.mmc3_bank_data[1] | 1
                }
            }
            _ => return self.vram[addr as usize],
        };
        let off = (bank as usize % (self.chr_banks * 8)) * 0x400 + (addr as usize & 0x3FF);
        self.chr[off % self.chr.len()]
    }

    fn prg_read(&self, addr: u16) -> u8 {
        if self.prg.is_empty() {
            return 0;
        }
        if self.mapper == 2 {
            // UNROM: $8000-$BFFF = switchable, $C000-$FFFF = fixed last bank
            let bank = if addr < 0xC000 { self.prg_bank } else { self.prg_banks - 1 };
            let off = bank * 0x4000 + (addr as usize & 0x3FFF);
            return self.prg[off % self.prg.len()];
        }
        if self.mapper == 4 {
            // MMC3: $8000-$9FFF and $C000-$DFFF are switchable, $E000-$FFFF = fixed last bank
            // bit 6 of bank_select swaps $8000/$C000 windows
            let bank = match addr {
                0x8000..=0x9FFF => {
                    if self.mmc3_bank_select & 0x40 == 0 {
                        self.mmc3_bank_data[6]
                    } else {
                        (self.prg_banks * 2 - 2) as u8
                    }
                }
                0xA000..=0xBFFF => self.mmc3_bank_data[7],
                0xC000..=0xDFFF => {
                    if self.mmc3_bank_select & 0x40 == 0 {
                        (self.prg_banks * 2 - 2) as u8
                    } else {
                        self.mmc3_bank_data[6]
                    }
                }
                0xE000..=0xFFFF => (self.prg_banks * 2 - 1) as u8,
                _ => 0,
            };
            let off = (bank as usize % (self.prg_banks * 2)) * 0x2000 + (addr as usize & 0x1FFF);
            return self.prg[off % self.prg.len()];
        }
        let off = (addr as usize - 0x8000) % self.prg.len();
        self.prg[off]
    }

    /// Side-effect-free read: RAM (mirrored) and PRG; everything else reads as 0.
    pub fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x7FF],
            0x6000..=0x7FFF => self.sram[(addr - 0x6000) as usize],
            0x8000..=0xFFFF => self.prg_read(addr),
            _ => 0,
        }
    }

    /// Side-effect-free read of a pattern-table byte ($0000-$1FFF), with mapper CHR banking.
    pub fn chr_peek(&self, addr: u16) -> u8 {
        self.chr_read(addr & 0x1FFF)
    }

    /// Current PPU address register value (14-bit).
    pub fn ppu_addr(&self) -> u16 {
        self.ppu_addr
    }

    pub(crate) fn vram_index(addr: u16) -> usize {
        let mut a = addr & 0x3FFF;
        if a >= 0x3F00 {
            a = 0x3F00 | (a & 0x1F);
            if a & 0x13 == 0x10 {
                a &= !0x10;
            }
        }
        a as usize
    }

    /// VRAM index for a PPU address, with nametables folded by the cartridge's mirroring.
    pub(crate) fn nt_index(&self, addr: u16) -> usize {
        let i = Self::vram_index(addr);
        if !(0x2000..0x3F00).contains(&i) {
            return i;
        }
        let a = 0x2000 | (i & 0x0FFF);
        if self.horizontal { a & !0x400 } else { a & !0x800 }
    }

    /// Base address of the nametable that screen-space column `wx` (0-511) reads from.
    pub(crate) fn nt_base(&self, wx: usize) -> usize {
        if self.horizontal { 0x2000 } else { 0x2000 + ((wx >> 8) & 1) * 0x400 }
    }

    fn ppu_increment(&mut self) {
        let inc = if self.ppuctrl & 0x04 != 0 { 32 } else { 1 };
        self.ppu_addr = self.ppu_addr.wrapping_add(inc) & 0x3FFF;
    }

    fn latch_controllers(&mut self) {
        self.shift = self.buttons;
        self.reads = [0; 2];
    }

    fn read_controller(&mut self, port: usize) -> u8 {
        if self.strobe {
            self.latch_controllers();
            return 0x40 | (self.buttons[port] >> 7);
        }
        let bit = if self.reads[port] >= 8 {
            1
        } else {
            let b = self.shift[port] >> 7;
            self.shift[port] <<= 1;
            self.reads[port] += 1;
            b
        };
        0x40 | bit
    }

    /// PPUCTRL as the renderer sees it: NT bits from `t` (`$2000` or `$2006`, whichever
    /// came last), everything else from the register.
    fn effective_ctrl(&self) -> u8 {
        (self.ppuctrl & !3) | (self.nt_sel & 3)
    }

    /// A `$2006` write also loads the PPU's `t` register: the first byte sets the NT
    /// bits, fine y and the top of coarse y; the second sets coarse x and the rest of
    /// coarse y. Fine x is untouched. The renderer and sprite-0 logic follow `t`.
    fn vram_addr_write_loads_t(&mut self, val: u8, second: bool) {
        let (sx, sy) = self.scroll;
        if second {
            let coarse_y = (sy >> 3) & 0x18 | (val >> 5);
            self.scroll = ((val & 0x1F) << 3 | (sx & 7), coarse_y << 3 | (sy & 7));
        } else {
            self.nt_sel = (val >> 2) & 3;
            let coarse_y = (sy >> 3) & 7 | (val & 3) << 3;
            self.scroll = (sx, coarse_y << 3 | (val >> 4) & 3);
        }
        self.reg_log.push((self.now, 0, self.effective_ctrl()));
        self.reg_log.push((self.now, 1, self.scroll.0));
        self.reg_log.push((self.now, 2, self.scroll.1));
    }

    /// Whether the background pixel at screen (x, y) is opaque, for the current scroll.
    fn bg_opaque(&self, x: u8, y: u8) -> bool {
        let wx = (self.nt_sel as u16 & 1) * 256 + self.scroll.0 as u16 + x as u16;
        let mut wy = self.scroll.1 as u16 + y as u16;
        if wy >= 240 {
            wy -= 240; // vertical mirroring: the vertical nametable bit has no effect
        }
        let nt = self.nt_base(wx as usize) as u16;
        let tile = self.vram[(nt + (wy / 8) * 32 + (wx & 0xFF) / 8) as usize] as u16;
        let base = if self.ppuctrl & 0x10 != 0 { 0x1000 } else { 0 };
        let a = base + tile * 16 + wy % 8;
        let bit = 0x80 >> (wx % 8);
        (self.chr_read(a) | self.chr_read(a + 8)) & bit != 0
    }

    /// First screen pixel where an opaque pixel of sprite 0 (8x8) overlaps an opaque
    /// background pixel, in scanline order. Pixel x = 255 never counts (as on hardware).
    pub fn sprite0_hit(&self) -> Option<(u8, u8)> {
        if self.ppumask & 0x18 != 0x18 {
            return None;
        }
        let [y, tile, attr, x] = [self.oam[0], self.oam[1], self.oam[2], self.oam[3]];
        let base = if self.ppuctrl & 0x08 != 0 { 0x1000 } else { 0 };
        for r in 0..8u16 {
            let sy = y as u16 + 1 + r;
            if sy >= 240 {
                break;
            }
            let row = if attr & 0x80 != 0 { 7 - r } else { r };
            let a = base + tile as u16 * 16 + row;
            let bits = self.chr_read(a) | self.chr_read(a + 8);
            for c in 0..8u16 {
                let sx = x as u16 + c;
                if sx >= 255 {
                    break;
                }
                let clipped = sx < 8 && self.ppumask & 0x06 != 0x06;
                let bit = if attr & 0x40 != 0 { 1 << c } else { 0x80 >> c };
                if !clipped && bits & bit != 0 && self.bg_opaque(sx as u8, sy as u8) {
                    return Some((sx as u8, sy as u8));
                }
            }
        }
        None
    }

    /// Re-evaluates this frame's sprite-0 hit after rendering is (re)enabled part-way
    /// through the visible picture: the hardware checks every rendered pixel, so a game
    /// that turns rendering off for a long vblank update and back on a few lines later
    /// (SMB's column writes) still gets its hit, as long as the hit pixel is still ahead.
    fn recheck_sprite0(&mut self) {
        let frame_start = self.now / CYCLES_PER_FRAME * CYCLES_PER_FRAME;
        if self.now < frame_start + pixel_cycle(0, 0) {
            return; // still in vblank: the normal latch at the first pixel handles it
        }
        if self.sprite0_at.is_some_and(|h| h >= frame_start) {
            return; // already latched a hit for this frame
        }
        if let Some((x, y)) = self.sprite0_hit() {
            let at = frame_start + pixel_cycle(x, y);
            if at >= self.now {
                self.sprite0_at = Some(at);
                self.sprite0_clear = frame_start + CYCLES_PER_FRAME + VBLANK_CYCLES;
            }
        }
    }

    /// One MMC3 scanline clock (the PPU's A12 rise near dot 260 of each rendered line).
    fn mmc3_clock(&mut self) {
        if self.mmc3_irq_counter == 0 || self.mmc3_irq_reload {
            self.mmc3_irq_counter = self.mmc3_irq_latch;
            self.mmc3_irq_reload = false;
        } else {
            self.mmc3_irq_counter -= 1;
        }
        if self.mmc3_irq_counter == 0 && self.mmc3_irq_enabled {
            self.irq_pending = true;
        }
    }

    fn check_vram_access(&mut self) {
        let rendering = self.ppumask & 0x18 != 0;
        let in_vblank = self.now % CYCLES_PER_FRAME < VBLANK_CYCLES;
        if rendering && !in_vblank {
            self.render_conflicts += 1;
        }
    }

    fn oam_dma(&mut self, page: u8) {
        let base = (page as u16) << 8;
        for i in 0..256u16 {
            let v = self.read(base | i);
            self.oam[self.oam_addr.wrapping_add(i as u8) as usize] = v;
        }
        self.pending_dma_cycles += OAM_DMA_CYCLES;
    }
}

impl Bus for NesBus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x7FF],
            0x2000..=0x3FFF => match addr & 7 {
                2 => {
                    self.write_toggle = false;
                    let hit = self.sprite0_at.is_some_and(|h| self.now >= h && self.now < self.sprite0_clear);
                    0x80 | if hit { 0x40 } else { 0 }
                }
                4 => self.oam[self.oam_addr as usize],
                7 => {
                    let a = self.ppu_addr;
                    let val = if a < 0x2000 { self.chr_read(a) } else { self.vram[self.nt_index(a)] };
                    let out = if a >= 0x3F00 {
                        // Palette reads are unbuffered; buffer gets the nametable underneath.
                        self.read_buffer = self.vram[self.nt_index(a - 0x1000)];
                        val
                    } else {
                        std::mem::replace(&mut self.read_buffer, val)
                    };
                    self.ppu_increment();
                    out
                }
                _ => 0,
            },
            0x4016 => self.read_controller(0),
            0x4017 => self.read_controller(1),
            0x6000..=0x7FFF => self.sram[(addr - 0x6000) as usize],
            0x8000..=0xFFFF => self.prg_read(addr),
            _ => 0,
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x7FF] = val,
            0x2000..=0x3FFF => match addr & 7 {
                0 => {
                    self.ppuctrl = val;
                    self.nt_sel = val & 3;
                    self.reg_log.push((self.now, 0, val));
                }
                1 => self.ppumask = val,
                3 => self.oam_addr = val,
                4 => {
                    self.oam[self.oam_addr as usize] = val;
                    self.oam_addr = self.oam_addr.wrapping_add(1);
                }
                5 => {
                    self.reg_log.push((self.now, 1 + self.write_toggle as u8, val));
                    if self.write_toggle {
                        self.scroll.1 = val;
                    } else {
                        self.scroll.0 = val;
                    }
                    self.write_toggle = !self.write_toggle;
                }
                6 => {
                    self.check_vram_access();
                    if self.write_toggle {
                        self.ppu_addr = (self.ppu_addr & 0xFF00) | val as u16;
                    } else {
                        self.ppu_addr = ((val as u16 & 0x3F) << 8) | (self.ppu_addr & 0x00FF);
                    }
                    self.write_toggle = !self.write_toggle;
                }
                7 => {
                    self.check_vram_access();
                    let i = self.nt_index(self.ppu_addr);
                    self.vram[i] = val;
                    self.ppu_increment();
                }
                _ => {}
            },
            0x4014 => self.oam_dma(val),
            0x8000..=0xFFFF if self.mapper == 2 => {
                let rom_byte = self.prg_read(addr);
                if val != rom_byte {
                    self.bus_conflicts += 1;
                }
                self.prg_bank = ((val & rom_byte) as usize) % self.prg_banks;
            }
            0x8000..=0xFFFF if self.mapper == 4 => {
                // MMC3
                match addr & 0xE001 {
                    0x8000 => self.mmc3_bank_select = val,
                    0x8001 => {
                        let reg = (self.mmc3_bank_select & 7) as usize;
                        self.mmc3_bank_data[reg] = val;
                    }
                    0xA000 => {
                        // Mirroring: bit 0 = 0 vertical, 1 horizontal
                        self.horizontal = val & 1 != 0;
                    }
                    0xC000 => self.mmc3_irq_latch = val,
                    0xC001 => {
                        self.mmc3_irq_counter = 0;
                        self.mmc3_irq_reload = true;
                    }
                    0xE000 => {
                        self.mmc3_irq_enabled = false;
                        self.irq_pending = false;
                    }
                    0xE001 => self.mmc3_irq_enabled = true,
                    _ => {}
                }
            }
            0x4016 => {
                let new = val & 1 != 0;
                if self.strobe || new {
                    self.latch_controllers();
                }
                self.strobe = new;
            }
            0x4000..=0x4013 | 0x4015 | 0x4017 => self.apu_writes.push((addr, val)),
            0x6000..=0x7FFF => self.sram[(addr - 0x6000) as usize] = val,
            _ => {}
        }
    }
}

pub struct TestNes {
    pub cpu: Cpu,
    pub bus: NesBus,
    pub frames: u64,
}

impl TestNes {
    /// Side-effectful write (through the CPU write path).
    pub fn poke(&mut self, addr: u16, val: u8) {
        self.bus.write(addr, val);
    }

    /// Parses an iNES image (mapper 0, 2, or 4), loads PRG/CHR and resets the CPU.
    pub fn from_ines(rom: &[u8]) -> Result<TestNes, String> {
        if rom.len() < 16 || &rom[0..4] != b"NES\x1A" {
            return Err("not an iNES file (missing NES\\x1A header)".into());
        }
        let prg_banks = rom[4] as usize;
        let chr_banks = rom[5] as usize;
        let mapper = (rom[6] >> 4) | (rom[7] & 0xF0);
        if mapper != 0 && mapper != 2 && mapper != 4 {
            return Err(format!("unsupported mapper {mapper} (supported: 0 NROM, 2 UNROM, 4 MMC3)"));
        }
        if mapper == 0 && prg_banks != 1 && prg_banks != 2 {
            return Err(format!("NROM requires 1 or 2 PRG banks, got {prg_banks}"));
        }
        if mapper == 2 && (prg_banks == 0 || prg_banks > 16) {
            return Err(format!("UNROM supports 1-16 PRG banks, got {prg_banks}"));
        }
        if mapper == 4 && (prg_banks == 0 || prg_banks > 128 || chr_banks == 0 || chr_banks > 128) {
            return Err(format!("MMC3 requires 1-128 PRG and 1-128 CHR banks, got PRG={prg_banks} CHR={chr_banks}"));
        }
        let trainer = if rom[6] & 0x04 != 0 { 512 } else { 0 };
        let prg_start = 16 + trainer;
        let prg_end = prg_start + prg_banks * 0x4000;
        let chr_end = prg_end + chr_banks * 0x2000;
        if rom.len() < chr_end {
            return Err(format!(
                "iNES file truncated: expected at least {chr_end} bytes, got {}",
                rom.len()
            ));
        }
        let prg = rom[prg_start..prg_end].to_vec();
        let chr = rom[prg_end..chr_end].to_vec();
        let mut bus = NesBus::new(prg, chr);
        bus.mapper = mapper;
        bus.prg_banks = prg_banks;
        bus.chr_banks = chr_banks;
        bus.horizontal = rom[6] & 0x01 == 0 && rom[6] & 0x08 == 0;
        if mapper == 4 {
            // MMC3: CHR registers start as identity (0-5); R6/R7 map PRG banks 0/1
            bus.mmc3_bank_data = [0, 1, 2, 3, 4, 5, 0, 1];
        }
        if !bus.chr.is_empty() && mapper != 4 {
            let n = bus.chr.len().min(0x2000);
            bus.vram[..n].copy_from_slice(&bus.chr[..n]);
        }
        let mut cpu = Cpu::new();
        cpu.reset(&mut bus);
        Ok(TestNes { cpu, bus, frames: 0 })
    }

    /// Executes one CPU instruction (plus any DMA stall), then handles frame boundaries,
    /// firing NMI if enabled. Returns the total cycles consumed.
    pub fn step(&mut self) -> Result<u32, String> {
        let before = self.cpu.cycles;
        self.bus.now = before;
        self.cpu.step(&mut self.bus).map_err(|e| e.to_string())?;
        let dma = std::mem::take(&mut self.bus.pending_dma_cycles);
        self.cpu.cycles += dma;
        while self.cpu.cycles >= (self.frames + 1) * CYCLES_PER_FRAME {
            self.frames += 1;
            if self.bus.ppuctrl & 0x80 != 0 {
                self.cpu.nmi(&mut self.bus);
            }
        }
        if self.bus.mapper == 4 && self.bus.ppumask & 0x18 != 0 {
            for _ in scanline_clocks(before)..scanline_clocks(self.cpu.cycles) {
                self.bus.mmc3_clock();
            }
        }
        if self.bus.irq_pending {
            self.cpu.irq(&mut self.bus);
        }
        // Rendering of the visible picture starts: latch this frame's sprite-0 hit.
        let frame_start = self.frames * CYCLES_PER_FRAME;
        let latch = frame_start + pixel_cycle(0, 0);
        if before < latch && self.cpu.cycles >= latch {
            self.bus.sprite0_at = self.bus.sprite0_hit().map(|(x, y)| frame_start + pixel_cycle(x, y));
            self.bus.sprite0_clear = frame_start + CYCLES_PER_FRAME + VBLANK_CYCLES;
        }
        Ok((self.cpu.cycles - before) as u32)
    }

    /// Runs until `cpu.pc == pc` (checked before each instruction).
    pub fn run_until_pc(&mut self, pc: u16, max_cycles: u64) -> Result<(), String> {
        let start = self.cpu.cycles;
        loop {
            if self.cpu.pc == pc {
                return Ok(());
            }
            if self.cpu.cycles - start > max_cycles {
                return Err(format!(
                    "timeout: pc ${pc:04X} not reached within {max_cycles} cycles (pc=${:04X})",
                    self.cpu.pc
                ));
            }
            self.step()?;
        }
    }

    /// Runs until `n` more frames have elapsed.
    pub fn run_frames(&mut self, n: u64) -> Result<(), String> {
        let target = self.frames + n;
        while self.frames < target {
            self.step()?;
        }
        Ok(())
    }

    pub fn peek(&self, addr: u16) -> u8 {
        self.bus.peek(addr)
    }

    pub fn set_buttons(&mut self, port: usize, mask: u8) {
        self.bus.buttons[port] = mask;
    }

    /// Returns a copy of SRAM ($6000-$7FFF, 8KB).
    pub fn sram_snapshot(&self) -> Vec<u8> {
        self.bus.sram_snapshot()
    }

    /// Restores SRAM from a previous snapshot (simulates battery-backed save).
    pub fn restore_sram(&mut self, data: &[u8]) {
        let len = data.len().min(0x2000);
        self.bus.sram[..len].copy_from_slice(&data[..len]);
    }
}

#[cfg(test)]
mod sprite0_tests {
    use super::*;

    /// Solid background everywhere, sprite 0 (solid tile) at (100, 50).
    fn bus() -> NesBus {
        let mut b = NesBus::new(vec![], vec![]);
        b.vram[16..24].fill(0xFF);
        b.vram[0x2000..0x23C0].fill(1);
        b.oam[..4].copy_from_slice(&[49, 1, 0, 100]);
        b
    }

    fn latch_at_first_pixel(b: &mut NesBus) {
        b.now = pixel_cycle(0, 0);
        b.sprite0_at = b.sprite0_hit().map(|(x, y)| pixel_cycle(x, y));
        b.sprite0_clear = CYCLES_PER_FRAME + VBLANK_CYCLES;
    }

    #[test]
    fn rendering_enabled_after_the_first_pixel_still_hits_a_later_sprite0() {
        let mut b = bus();
        b.ppumask = 0x06; // rendering off while a long vblank update finishes
        latch_at_first_pixel(&mut b);
        assert_eq!(b.sprite0_at, None);
        b.now = pixel_cycle(0, 10);
        b.write(0x2001, 0x1E); // rendering back on at line 10, sprite 0 is at line 50
        assert_eq!(b.sprite0_at, Some(pixel_cycle(100, 50)));
        b.now = pixel_cycle(0, 40);
        assert_eq!(b.read(0x2002) & 0x40, 0, "not hit yet");
        b.now = pixel_cycle(0, 60);
        assert_ne!(b.read(0x2002) & 0x40, 0, "hit once the sprite line was rendered");
    }

    #[test]
    fn rendering_enabled_after_the_sprite_line_does_not_hit() {
        let mut b = bus();
        b.ppumask = 0x06;
        latch_at_first_pixel(&mut b);
        b.now = pixel_cycle(0, 80);
        b.write(0x2001, 0x1E);
        assert_eq!(b.sprite0_at, None);
    }

    /// SMB's NMI: a column write selects nametable 1 via `$2000` (with +32 increment),
    /// then `UpdateScreen` ends with `$2006 <- $00, $00`, which resets `t` so the
    /// status bar at the top is still fetched from nametable 0, where sprite 0 hits.
    #[test]
    fn vram_address_writes_reload_the_nametable_and_scroll_of_t() {
        let mut b = bus(); // nametable 0 solid, nametable 1 blank
        b.ppumask = 0x1E;
        b.now = 500;
        b.write(0x2000, 0x05); // nametable 1, +32 increment (column write in progress)
        assert_eq!(b.nt_sel, 1);
        b.write(0x2006, 0x3F);
        b.write(0x2006, 0x00);
        b.write(0x2006, 0x00);
        b.write(0x2006, 0x00);
        assert_eq!(b.nt_sel, 0, "$2006 <- $0000 clears the NT bits of t");
        assert_eq!(b.scroll, (0, 0));
        assert_eq!(b.ppuctrl, 0x05, "the software-visible register is unchanged");
        latch_at_first_pixel(&mut b);
        assert_eq!(b.sprite0_at, Some(pixel_cycle(100, 50)), "hit on nametable 0");
        let last_ctrl = b.reg_log.iter().rev().find(|e| e.1 == 0).unwrap().2;
        assert_eq!(last_ctrl & 3, 0, "the renderer log carries the effective NT bits");
        // $24E5 = fine y 2 (bit 13), nametable 1, coarse y 7, coarse x 5; fine x is kept
        b.scroll = (3, 0);
        b.write(0x2006, 0x24);
        b.write(0x2006, 0xE5);
        assert_eq!(b.nt_sel, 1);
        assert_eq!(b.scroll, (5 * 8 + 3, 7 * 8 + 2), "coarse x/y and fine y from the address");
    }

    #[test]
    fn mask_writes_in_vblank_or_after_a_latched_hit_change_nothing() {
        let mut b = bus();
        b.ppumask = 0x1E;
        latch_at_first_pixel(&mut b);
        let hit = b.sprite0_at;
        assert!(hit.is_some());
        b.now = pixel_cycle(0, 5);
        b.write(0x2001, 0x1E);
        assert_eq!(b.sprite0_at, hit);
        b.now = 100; // vblank of the next frame
        b.write(0x2001, 0x1E);
        assert_eq!(b.sprite0_at, hit);
    }
}

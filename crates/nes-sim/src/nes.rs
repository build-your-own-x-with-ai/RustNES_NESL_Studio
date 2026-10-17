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

pub struct NesBus {
    pub ram: [u8; 0x800],
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub vram: Vec<u8>,
    pub oam: [u8; 256],
    pub ppuctrl: u8,
    pub ppumask: u8,
    pub scroll: (u8, u8),
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
    /// `(cycle, kind, value)` for every PPUCTRL / PPUSCROLL write:
    /// kind 0 = `$2000`, 1 = `$2005` x, 2 = `$2005` y.
    pub reg_log: Vec<(u64, u8, u8)>,
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
    pub fn new(prg: Vec<u8>, chr: Vec<u8>) -> Self {
        NesBus {
            ram: [0; 0x800],
            prg,
            chr,
            vram: vec![0; 0x4000],
            oam: [0; 256],
            ppuctrl: 0,
            ppumask: 0,
            scroll: (0, 0),
            buttons: [0; 2],
            apu_writes: Vec::new(),
            render_conflicts: 0,
            now: 0,
            sprite0_at: None,
            sprite0_clear: 0,
            reg_log: Vec::new(),
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

    fn prg_read(&self, addr: u16) -> u8 {
        if self.prg.is_empty() {
            return 0;
        }
        let off = (addr as usize - 0x8000) % self.prg.len();
        self.prg[off]
    }

    /// Side-effect-free read: RAM (mirrored) and PRG; everything else reads as 0.
    pub fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x7FF],
            0x8000..=0xFFFF => self.prg_read(addr),
            _ => 0,
        }
    }

    /// Current PPU address register value (14-bit).
    pub fn ppu_addr(&self) -> u16 {
        self.ppu_addr
    }

    fn vram_index(addr: u16) -> usize {
        let mut a = addr & 0x3FFF;
        if a >= 0x3F00 {
            a = 0x3F00 | (a & 0x1F);
            if a & 0x13 == 0x10 {
                a &= !0x10;
            }
        }
        a as usize
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

    /// Whether the background pixel at screen (x, y) is opaque, for the current scroll.
    fn bg_opaque(&self, x: u8, y: u8) -> bool {
        let wx = (self.ppuctrl as u16 & 1) * 256 + self.scroll.0 as u16 + x as u16;
        let mut wy = self.scroll.1 as u16 + y as u16;
        if wy >= 240 {
            wy -= 240; // vertical mirroring: the vertical nametable bit has no effect
        }
        let nt = 0x2000 + ((wx >> 8) & 1) * 0x400; // vertical mirroring
        let tile = self.vram[(nt + (wy / 8) * 32 + (wx & 0xFF) / 8) as usize] as u16;
        let base = if self.ppuctrl & 0x10 != 0 { 0x1000 } else { 0 };
        let a = (base + tile * 16 + wy % 8) as usize;
        let bit = 0x80 >> (wx % 8);
        (self.vram[a] | self.vram[a + 8]) & bit != 0
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
            let a = (base + tile as u16 * 16 + row) as usize;
            let bits = self.vram[a] | self.vram[a + 8];
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
                    let val = self.vram[Self::vram_index(a)];
                    let out = if a >= 0x3F00 {
                        // Palette reads are unbuffered; buffer gets the nametable underneath.
                        self.read_buffer = self.vram[Self::vram_index(a - 0x1000)];
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
                    self.vram[Self::vram_index(self.ppu_addr)] = val;
                    self.ppu_increment();
                }
                _ => {}
            },
            0x4014 => self.oam_dma(val),
            0x4016 => {
                let new = val & 1 != 0;
                if self.strobe || new {
                    self.latch_controllers();
                }
                self.strobe = new;
            }
            0x4000..=0x4013 | 0x4015 | 0x4017 => self.apu_writes.push((addr, val)),
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
    /// Parses an iNES image (mapper 0 only), loads PRG/CHR and resets the CPU.
    pub fn from_ines(rom: &[u8]) -> Result<TestNes, String> {
        if rom.len() < 16 || &rom[0..4] != b"NES\x1A" {
            return Err("not an iNES file (missing NES\\x1A header)".into());
        }
        let prg_banks = rom[4] as usize;
        let chr_banks = rom[5] as usize;
        let mapper = (rom[6] >> 4) | (rom[7] & 0xF0);
        if mapper != 0 {
            return Err(format!("unsupported mapper {mapper} (only mapper 0 / NROM)"));
        }
        if prg_banks != 1 && prg_banks != 2 {
            return Err(format!("NROM requires 1 or 2 PRG banks, got {prg_banks}"));
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
        if !bus.chr.is_empty() {
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
}

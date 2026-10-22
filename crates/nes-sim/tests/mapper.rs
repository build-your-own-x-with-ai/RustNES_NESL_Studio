//! 010: mapper 2 (UNROM) support.
//! 011: mapper 4 (MMC3) support.
use nes_sim::TestNes;

/// Four 16KB banks: switchable 0/1/2 go in $8000-$BFFF, fixed 3 at $C000-$FFFF.
/// Bank N is filled with N*0x11; vectors point to $C000 (fixed bank).
fn unrom4() -> Vec<u8> {
    let mut rom = b"NES\x1A\x04\x00\x20\x00".to_vec(); // 4 PRG banks, 0 CHR, mapper 2
    rom.resize(16, 0);
    for b in 0..4u8 {
        let mut bank = vec![b * 0x11; 0x4000];
        // bus-conflict-safe bank table at $8000: ROM[i] = i for i in 0..16
        for (i, v) in bank.iter_mut().take(16).enumerate() {
            *v = i as u8;
        }
        rom.extend_from_slice(&bank);
    }
    rom[0x4000 * 3 + 0x3FFA - 0xC000] = 0; // NMI = $C000
    rom[0x4000 * 3 + 0x3FFB - 0xC000] = 0xC0;
    rom[0x4000 * 3 + 0x3FFC - 0xC000] = 0; // reset = $C000
    rom[0x4000 * 3 + 0x3FFD - 0xC000] = 0xC0;
    rom[0x4000 * 3 + 0x3FFE - 0xC000] = 0; // IRQ = $C000
    rom[0x4000 * 3 + 0x3FFF - 0xC000] = 0xC0;
    rom
}

#[test]
fn mapper2_loads_bank0_and_fixed_bank() {
    let nes = TestNes::from_ines(&unrom4()).unwrap();
    assert_eq!(nes.peek(0x8000), 0x00, "bank 0 at $8000");
    assert_eq!(nes.peek(0xBFFF), 0x00);
    assert_eq!(nes.peek(0xC000), 0x00, "fixed bank 3 at $C000 (bus-conflict table)");
    assert_eq!(nes.peek(0xFFFF), 0x33, "end of fixed bank");
}

#[test]
fn mapper2_switches_banks() {
    let mut nes = TestNes::from_ines(&unrom4()).unwrap();
    nes.poke(0x8002, 2); // write to $8000+N where ROM[addr]=N
    assert_eq!(nes.peek(0x8010), 0x22, "bank 2 selected (past the bus-conflict table)");
    assert_eq!(nes.peek(0xC010), 0x33, "fixed bank unchanged (past the table)");
    nes.poke(0x8001, 1);
    assert_eq!(nes.peek(0x8010), 0x11, "bank 1 selected");
}

#[test]
fn mapper2_detects_bus_conflicts() {
    let mut nes = TestNes::from_ines(&unrom4()).unwrap();
    assert_eq!(nes.bus.bus_conflicts, 0);
    nes.poke(0x8000, 0); // ROM[$8000]=0, write 0: no conflict
    assert_eq!(nes.bus.bus_conflicts, 0);
    nes.poke(0x8000, 0xFF); // ROM[$8000]=0, write 0xFF: conflict, ANDed to 0
    assert_eq!(nes.bus.bus_conflicts, 1);
    assert_eq!(nes.peek(0x8000), 0x00, "AND of written & ROM: 0xFF & 0x00 = 0x00, bank unchanged");
}

/// Writes `val` to nametable address `addr` through PPUADDR/PPUDATA.
fn ppu_write(nes: &mut TestNes, addr: u16, val: u8) {
    nes.poke(0x2006, (addr >> 8) as u8);
    nes.poke(0x2006, addr as u8);
    nes.poke(0x2007, val);
}

#[test]
fn horizontal_mirroring_folds_2400_onto_2000() {
    let rom = unrom4(); // flags6 = 0x20: mirroring bit clear = horizontal
    let mut nes = TestNes::from_ines(&rom).unwrap();
    ppu_write(&mut nes, 0x2000, 0x11);
    ppu_write(&mut nes, 0x2400, 0x22);
    assert_eq!(nes.bus.vram[0x2000], 0x22, "$2400 mirrors $2000 with horizontal mirroring");
}

#[test]
fn vertical_mirroring_keeps_2000_and_2400_separate() {
    let mut rom = unrom4();
    rom[6] |= 0x01; // vertical
    let mut nes = TestNes::from_ines(&rom).unwrap();
    ppu_write(&mut nes, 0x2000, 0x11);
    ppu_write(&mut nes, 0x2400, 0x22);
    ppu_write(&mut nes, 0x2800, 0x33);
    assert_eq!(nes.bus.vram[0x2000], 0x33, "$2800 mirrors $2000 with vertical mirroring");
    assert_eq!(nes.bus.vram[0x2400], 0x22);
}

/// 8 PRG banks (8KB each = 64KB total = 4 iNES units), 8 CHR banks (1KB each = 8KB total = 1 iNES unit). Vectors at $FFFC point to $E000.
/// PRG: bank N filled with N*0x11; CHR: bank N filled with N*0x22.
fn mmc3_rom() -> Vec<u8> {
    let mut rom = b"NES\x1A\x04\x01\x40\x00".to_vec(); // 4 iNES PRG units (16KB each) = 64KB = 8 MMC3 banks (8KB each), 1 CHR unit = 8KB = 8 banks (1KB each), mapper 4
    rom.resize(16, 0);
    for b in 0..8u8 {
        let bank = vec![b * 0x11; 0x2000]; // 8KB PRG bank
        rom.extend_from_slice(&bank);
    }
    for b in 0..8u8 {
        let chr = vec![b * 0x22; 0x400]; // 1KB CHR bank
        rom.extend_from_slice(&chr);
    }
    // Vectors in last bank ($E000-$FFFF = bank 7)
    rom[16 + 7 * 0x2000 + 0x1FFC] = 0x00; // reset = $E000
    rom[16 + 7 * 0x2000 + 0x1FFD] = 0xE0;
    rom
}

#[test]
fn mmc3_loads_default_banks() {
    let nes = TestNes::from_ines(&mmc3_rom()).unwrap();
    // Default: banks 6,7 fixed at E000/FFFF; $8000/$C000 depend on bank_select bit 6
    // But initial bank_select = 0, so $8000 = bank 0, $A000 = bank 1, $C000 = bank 6, $E000 = bank 7
    assert_eq!(nes.peek(0x8000), 0x00, "PRG bank 0 at $8000");
    assert_eq!(nes.peek(0xA000), 0x11, "PRG bank 1 at $A000");
    assert_eq!(nes.peek(0xC000), 0x66, "PRG bank 6 at $C000");
    assert_eq!(nes.peek(0xE000), 0x77, "PRG bank 7 (fixed) at $E000");
}

#[test]
fn mmc3_switches_prg_banks() {
    let mut nes = TestNes::from_ines(&mmc3_rom()).unwrap();
    nes.poke(0x8000, 6); // bank register 6 (PRG)
    nes.poke(0x8001, 2); // bank 2
    assert_eq!(nes.peek(0x8001), 0x22, "bank 2 at $8000");
    nes.poke(0x8000, 7); // bank register 7
    nes.poke(0x8001, 3); // bank 3
    assert_eq!(nes.peek(0xA001), 0x33, "bank 3 at $A000");
}

#[test]
fn mmc3_switches_chr_banks() {
    let mut nes = TestNes::from_ines(&mmc3_rom()).unwrap();
    nes.poke(0x8000, 0); // CHR bank register 0 (2KB at $0000)
    nes.poke(0x8001, 4); // CHR banks 4/5
    assert_eq!(nes.bus.chr_peek(0x0000), 0x88, "CHR bank 4 at $0000 (4*0x22 = 0x88)");
    assert_eq!(nes.bus.chr_peek(0x0400), 0xAA, "2KB register: bank 5 at $0400");
    assert_eq!(nes.bus.chr_peek(0x1000), 0x44, "R2 (default 2) at $1000");
}


/// MMC3 ROM whose fixed bank runs: rendering on (PPUMASK = `mask`), IRQ latch = 9,
/// reload, enable, CLI, spin. The IRQ handler at $E01A does INC $10, acknowledges
/// ($E000), re-enables ($E001) and returns.
fn mmc3_irq_rom(mask: u8) -> Vec<u8> {
    let mut rom = mmc3_rom();
    #[rustfmt::skip]
    let code = [
        0x78,                   // E000 SEI
        0xA9, 0x00,             // E001 LDA #0
        0x8D, 0x00, 0x20,       // E003 STA $2000
        0xA9, mask,             // E006 LDA #mask
        0x8D, 0x01, 0x20,       // E008 STA $2001
        0xA9, 0x09,             // E00B LDA #9
        0x8D, 0x00, 0xC0,       // E00D STA $C000  latch
        0x8D, 0x01, 0xC0,       // E010 STA $C001  reload
        0x8D, 0x01, 0xE0,       // E013 STA $E001  enable
        0x58,                   // E016 CLI
        0x4C, 0x17, 0xE0,       // E017 JMP $E017
        0xE6, 0x10,             // E01A INC $10    (IRQ handler)
        0x8D, 0x00, 0xE0,       // E01C STA $E000  acknowledge
        0x8D, 0x01, 0xE0,       // E01F STA $E001  re-enable
        0x40,                   // E022 RTI
    ];
    let fixed = 16 + 7 * 0x2000;
    rom[fixed..fixed + code.len()].copy_from_slice(&code);
    for (off, v) in [(0x1FFA, 0x22u8), (0x1FFB, 0xE0), (0x1FFC, 0x00), (0x1FFD, 0xE0), (0x1FFE, 0x1A), (0x1FFF, 0xE0)] {
        rom[fixed + off] = v;
    }
    rom
}

const IRQ_HANDLER: u16 = 0xE01A;

#[test]
fn mmc3_irq_fires_after_latch_scanlines() {
    use nes_sim::nes::{CYCLES_PER_FRAME, VBLANK_CYCLES};
    let mut nes = TestNes::from_ines(&mmc3_irq_rom(0x18)).unwrap();
    nes.run_until_pc(IRQ_HANDLER, CYCLES_PER_FRAME).unwrap();
    // The pre-render line reloads the counter to 9; lines 0..=8 count it down, so the IRQ
    // arrives at dot 260 of line 8 and a handler's changes show from line 9.
    let expected = VBLANK_CYCLES + (341 * 9 + 260) / 3;
    let at = nes.cpu.cycles % CYCLES_PER_FRAME;
    assert!((expected..expected + 20).contains(&at), "first IRQ at cycle {at}, expected ~{expected}");
    // The next IRQ is 10 scanlines (~1137 CPU cycles) later.
    let first = nes.cpu.cycles;
    nes.step().unwrap();
    nes.run_until_pc(IRQ_HANDLER, CYCLES_PER_FRAME).unwrap();
    let gap = nes.cpu.cycles - first;
    assert!((1120..1160).contains(&gap), "IRQ spacing {gap} cycles, expected ~1137 (10 lines)");
}

#[test]
fn mmc3_irq_counts_every_frame() {
    let mut nes = TestNes::from_ines(&mmc3_irq_rom(0x18)).unwrap();
    nes.run_frames(10).unwrap();
    // 241 clocks per frame (240 visible + pre-render), one IRQ per 10 clocks
    let n = nes.peek(0x10) as u64;
    assert!((240..=242).contains(&n), "IRQs in 10 frames: {n}, expected 241");
}

#[test]
fn mmc3_irq_needs_rendering() {
    let mut nes = TestNes::from_ines(&mmc3_irq_rom(0x00)).unwrap();
    nes.run_frames(3).unwrap();
    assert_eq!(nes.peek(0x10), 0, "no scanline clocks while rendering is off");
}

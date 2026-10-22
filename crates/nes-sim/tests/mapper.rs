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


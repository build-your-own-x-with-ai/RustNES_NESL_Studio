//! 010: mapper 2 (UNROM) support.
use nes_sim::TestNes;

/// Four 16KB banks: switchable 0/1/2 go in $8000-$BFFF, fixed 3 at $C000-$FFFF.
/// Bank N is filled with N*0x11; vectors point to $C000 (fixed bank).
fn unrom4() -> Vec<u8> {
    let mut rom = b"NES\x1A\x04\x00\x20\x00".to_vec(); // 4 PRG banks, 0 CHR, mapper 2
    rom.resize(16, 0);
    for b in 0..4u8 {
        let mut bank = vec![b * 0x11; 0x4000];
        // bus-conflict-safe bank table at $8000: ROM[i] = i for i in 0..16
        for i in 0..16 {
            bank[i] = i as u8;
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

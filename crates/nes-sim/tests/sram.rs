//! SRAM ($6000-$7FFF) is battery-backed save memory, preserved across power cycles.
use nes_sim::TestNes;

/// A minimal ROM that writes to SRAM and halts.
fn sram_test_rom() -> Vec<u8> {
    // NROM-256: 32KB PRG, 8KB CHR, vertical mirroring, battery/PRG-RAM (flags6 = 0x03)
    let mut rom = vec![
        b'N', b'E', b'S', 0x1A,
        2, // PRG banks
        1, // CHR banks
        0x03, // flags6: mapper 0, vertical, battery
        0x00,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    // PRG: 32KB (16KB code + 16KB empty)
    let mut prg = vec![0; 0x8000];

    // Code at $C000: write 0x42 to $6000, 0x99 to $7FFF, then halt
    let code = [
        0xA9, 0x42,       // LDA #$42
        0x8D, 0x00, 0x60, // STA $6000
        0xA9, 0x99,       // LDA #$99
        0x8D, 0xFF, 0x7F, // STA $7FFF
        0x4C, 0x0A, 0xC0, // JMP $C00A (infinite loop)
    ];
    prg[0x4000..0x4000 + code.len()].copy_from_slice(&code);

    // Reset vector at $FFFC points to $C000
    prg[0x7FFC] = 0x00;
    prg[0x7FFD] = 0xC0;

    rom.extend_from_slice(&prg);

    // CHR: 8KB empty
    rom.extend_from_slice(&vec![0; 0x2000]);

    rom
}

#[test]
fn sram_persists_across_power_cycles() {
    // First run: write to SRAM
    let rom = sram_test_rom();
    let mut nes1 = TestNes::from_ines(&rom).unwrap();
    nes1.run_frames(2).unwrap();

    assert_eq!(nes1.peek(0x6000), 0x42, "SRAM write succeeded");
    assert_eq!(nes1.peek(0x7FFF), 0x99, "SRAM high byte");

    // Save SRAM
    let save = nes1.sram_snapshot();
    assert_eq!(save.len(), 0x2000, "SRAM is 8KB");
    assert_eq!(save[0], 0x42);
    assert_eq!(save[0x1FFF], 0x99);

    // Second run: fresh ROM, restore SRAM
    let mut nes2 = TestNes::from_ines(&rom).unwrap();
    assert_eq!(nes2.peek(0x6000), 0, "fresh SRAM is zeroed");
    nes2.restore_sram(&save);
    assert_eq!(nes2.peek(0x6000), 0x42, "SRAM restored from save");
    assert_eq!(nes2.peek(0x7FFF), 0x99);
}

#[test]
fn sram_is_readable_and_writable_during_execution() {
    let rom = sram_test_rom();
    let mut nes = TestNes::from_ines(&rom).unwrap();

    // Before reset runs
    assert_eq!(nes.peek(0x6000), 0);

    // After 1 frame
    nes.run_frames(1).unwrap();
    assert_eq!(nes.peek(0x6000), 0x42);

    // Modify via peek/poke
    nes.poke(0x6100, 0x77);
    assert_eq!(nes.peek(0x6100), 0x77);

    // Snapshot sees the change
    let save = nes.sram_snapshot();
    assert_eq!(save[0x100], 0x77);
}

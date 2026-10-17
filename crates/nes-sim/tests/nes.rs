//! Tests for the NES test machine (bus + frame timing).

use nes_sim::{NesBus, TestNes};

/// Builds a 16KB-PRG / 8KB-CHR mapper-0 iNES image. `prog` is placed at $8000 (reset),
/// `nmi` at $9000 (NMI vector).
fn rom(prog: &[u8], nmi: &[u8]) -> Vec<u8> {
    let mut v = vec![b'N', b'E', b'S', 0x1A, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut prg = vec![0xEA; 0x4000];
    prg[..prog.len()].copy_from_slice(prog);
    prg[0x1000..0x1000 + nmi.len()].copy_from_slice(nmi);
    // vectors: NMI=$9000, RESET=$8000, IRQ=$8000
    prg[0x3FFA..].copy_from_slice(&[0x00, 0x90, 0x00, 0x80, 0x00, 0x80]);
    v.extend_from_slice(&prg);
    v.extend(std::iter::repeat_n(0x11, 0x2000));
    v
}

fn machine(prog: &[u8]) -> TestNes {
    TestNes::from_ines(&rom(prog, &[0x40])).unwrap()
}

fn run_n(nes: &mut TestNes, n: usize) {
    for _ in 0..n {
        nes.step().unwrap();
    }
}

#[test]
fn parses_ines_and_resets() {
    let nes = machine(&[]);
    assert_eq!(nes.cpu.pc, 0x8000);
    assert_eq!(nes.bus.prg.len(), 0x4000);
    assert_eq!(nes.bus.chr.len(), 0x2000);
    assert_eq!(nes.bus.chr[0], 0x11);
    assert_eq!(nes.bus.vram.len(), 0x4000);
    assert_eq!(nes.frames, 0);
}

#[test]
fn rejects_bad_roms() {
    assert!(TestNes::from_ines(b"NOPE").is_err());
    let mut r = rom(&[], &[]);
    r[6] = 0x10; // mapper 1
    assert!(TestNes::from_ines(&r).is_err());
    let r = rom(&[], &[]);
    assert!(TestNes::from_ines(&r[..0x100]).is_err());
}

#[test]
fn prg_16k_is_mirrored_and_32k_is_flat() {
    let nes = machine(&[0xA9, 0x42]);
    assert_eq!(nes.peek(0x8000), 0xA9);
    assert_eq!(nes.peek(0xC000), 0xA9);
    assert_eq!(nes.peek(0xC001), 0x42);

    let mut r = vec![b'N', b'E', b'S', 0x1A, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut prg = vec![0u8; 0x8000];
    prg[0] = 0x12;
    prg[0x4000] = 0x34;
    prg[0x7FFC] = 0x00;
    prg[0x7FFD] = 0xC0;
    r.extend_from_slice(&prg);
    let nes = TestNes::from_ines(&r).unwrap();
    assert_eq!(nes.peek(0x8000), 0x12);
    assert_eq!(nes.peek(0xC000), 0x34);
    assert_eq!(nes.cpu.pc, 0xC000);
    assert!(nes.bus.chr.is_empty() || nes.bus.chr.len() == 0x2000);
}

#[test]
fn ram_is_mirrored() {
    // LDA #$5A; STA $0812
    let mut nes = machine(&[0xA9, 0x5A, 0x8D, 0x12, 0x08]);
    run_n(&mut nes, 2);
    assert_eq!(nes.bus.ram[0x12], 0x5A);
    assert_eq!(nes.peek(0x0012), 0x5A);
    assert_eq!(nes.peek(0x1812), 0x5A);
    assert_eq!(nes.peek(0x6000), 0);
}

#[test]
fn ppu_status_reports_vblank() {
    // LDA $2002
    let mut nes = machine(&[0xAD, 0x02, 0x20]);
    run_n(&mut nes, 1);
    assert_eq!(nes.cpu.a & 0x80, 0x80);
}

#[test]
fn vram_writes_increment_by_one() {
    // LDA $2002; LDA #$20; STA $2006; LDA #$00; STA $2006; LDA #$01; STA $2007; LDA #$02; STA $2007
    let prog = [
        0xAD, 0x02, 0x20, 0xA9, 0x20, 0x8D, 0x06, 0x20, 0xA9, 0x00, 0x8D, 0x06, 0x20, 0xA9, 0x01,
        0x8D, 0x07, 0x20, 0xA9, 0x02, 0x8D, 0x07, 0x20,
    ];
    let mut nes = machine(&prog);
    run_n(&mut nes, 9);
    assert_eq!(nes.bus.vram[0x2000], 1);
    assert_eq!(nes.bus.vram[0x2001], 2);
}

#[test]
fn vram_writes_increment_by_32() {
    // LDA #$04; STA $2000; LDA #$21; STA $2006; LDA #$05; STA $2006; LDA #$AA; STA $2007; STA $2007
    let prog = [
        0xA9, 0x04, 0x8D, 0x00, 0x20, 0xA9, 0x21, 0x8D, 0x06, 0x20, 0xA9, 0x05, 0x8D, 0x06, 0x20,
        0xA9, 0xAA, 0x8D, 0x07, 0x20, 0x8D, 0x07, 0x20,
    ];
    let mut nes = machine(&prog);
    run_n(&mut nes, 9);
    assert_eq!(nes.bus.ppuctrl, 0x04);
    assert_eq!(nes.bus.vram[0x2105], 0xAA);
    assert_eq!(nes.bus.vram[0x2125], 0xAA);
    assert_eq!(nes.bus.vram[0x2106], 0);
}

#[test]
fn ppu_registers_mirror_every_8_bytes() {
    // STA $3FF8 (-> $2000)
    let mut nes = machine(&[0xA9, 0x80, 0x8D, 0xF8, 0x3F]);
    run_n(&mut nes, 2);
    assert_eq!(nes.bus.ppuctrl, 0x80);
}

/// Writes `val` through $2007 at PPU address `addr` using the bus directly.
fn ppu_write(bus: &mut NesBus, addr: u16, val: u8) {
    use nes_sim::Bus;
    bus.read(0x2002);
    bus.write(0x2006, (addr >> 8) as u8);
    bus.write(0x2006, addr as u8);
    bus.write(0x2007, val);
}

#[test]
fn palette_mirroring() {
    let mut nes = machine(&[]);
    let bus = &mut nes.bus;
    ppu_write(bus, 0x3F10, 0x0F);
    assert_eq!(bus.vram[0x3F00], 0x0F);
    ppu_write(bus, 0x3F1C, 0x21);
    assert_eq!(bus.vram[0x3F0C], 0x21);
    ppu_write(bus, 0x3F25, 0x16);
    assert_eq!(bus.vram[0x3F05], 0x16);
    ppu_write(bus, 0x3F11, 0x30);
    assert_eq!(bus.vram[0x3F11], 0x30);
    // address wraps to 14 bits
    ppu_write(bus, 0x7F01, 0x2A);
    assert_eq!(bus.vram[0x3F01], 0x2A);
}

#[test]
fn ppudata_reads_are_buffered() {
    use nes_sim::Bus;
    let mut nes = machine(&[]);
    let bus = &mut nes.bus;
    bus.vram[0x2400] = 0x77;
    bus.vram[0x2401] = 0x88;
    bus.vram[0x3F00] = 0x0D;
    bus.read(0x2002);
    bus.write(0x2006, 0x24);
    bus.write(0x2006, 0x00);
    let _stale = bus.read(0x2007);
    assert_eq!(bus.read(0x2007), 0x77);
    assert_eq!(bus.read(0x2007), 0x88);
    bus.write(0x2006, 0x3F);
    bus.write(0x2006, 0x00);
    assert_eq!(bus.read(0x2007), 0x0D);
}

#[test]
fn scroll_writes() {
    use nes_sim::Bus;
    let mut nes = machine(&[]);
    let bus = &mut nes.bus;
    bus.write(0x2005, 0x12);
    bus.write(0x2005, 0x34);
    assert_eq!(bus.scroll, (0x12, 0x34));
    // $2002 read resets toggle
    bus.write(0x2005, 0x56);
    bus.read(0x2002);
    bus.write(0x2005, 0x78);
    bus.write(0x2005, 0x9A);
    assert_eq!(bus.scroll, (0x78, 0x9A));
}

#[test]
fn oam_addr_and_data() {
    use nes_sim::Bus;
    let mut nes = machine(&[]);
    let bus = &mut nes.bus;
    bus.write(0x2003, 0x10);
    bus.write(0x2004, 0xAB);
    bus.write(0x2004, 0xCD);
    assert_eq!(bus.oam[0x10], 0xAB);
    assert_eq!(bus.oam[0x11], 0xCD);
}

#[test]
fn oam_dma() {
    // LDA #$02; STA $4014
    let mut nes = machine(&[0xA9, 0x02, 0x8D, 0x14, 0x40]);
    for i in 0..256 {
        nes.bus.ram[0x200 + i] = i as u8;
    }
    run_n(&mut nes, 1);
    let before = nes.cpu.cycles;
    let used = nes.step().unwrap();
    assert_eq!(nes.cpu.cycles - before, 4 + 513);
    assert_eq!(used, 4 + 513);
    for i in 0..256 {
        assert_eq!(nes.bus.oam[i], i as u8);
    }
}

#[test]
fn controller_reads() {
    use nes_sim::Bus;
    let mut nes = machine(&[]);
    nes.set_buttons(0, 0b1001_0001); // A, Start, Right
    nes.set_buttons(1, 0b0100_0000); // B on pad 2
    let bus = &mut nes.bus;
    bus.write(0x4016, 1);
    // while strobe high, always A
    assert_eq!(bus.read(0x4016) & 1, 1);
    assert_eq!(bus.read(0x4016) & 1, 1);
    bus.write(0x4016, 0);
    let bits: Vec<u8> = (0..8).map(|_| bus.read(0x4016) & 1).collect();
    assert_eq!(bits, vec![1, 0, 0, 1, 0, 0, 0, 1]);
    // after 8 reads: 1s
    assert_eq!(bus.read(0x4016) & 1, 1);
    assert_eq!(bus.read(0x4016) & 1, 1);
    let bits2: Vec<u8> = (0..8).map(|_| bus.read(0x4017) & 1).collect();
    assert_eq!(bits2, vec![0, 1, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn apu_writes_are_logged() {
    // LDA #$3F; STA $4000; STA $4015; STA $4017; STA $4016 (not logged)
    let mut nes = machine(&[
        0xA9, 0x3F, 0x8D, 0x00, 0x40, 0x8D, 0x15, 0x40, 0x8D, 0x17, 0x40, 0x8D, 0x16, 0x40,
    ]);
    run_n(&mut nes, 5);
    assert_eq!(nes.bus.apu_writes, vec![(0x4000, 0x3F), (0x4015, 0x3F), (0x4017, 0x3F)]);
}

#[test]
fn frames_advance_without_nmi_when_disabled() {
    // infinite loop: JMP $8000
    let mut nes = machine(&[0x4C, 0x00, 0x80]);
    nes.run_frames(3).unwrap();
    assert_eq!(nes.frames, 3);
    assert!(nes.cpu.cycles >= 3 * 29781);
    assert!(nes.cpu.cycles < 4 * 29781);
    // no NMI fired -> stack untouched
    assert_eq!(nes.cpu.sp, 0xFD);
}

#[test]
fn nmi_fires_each_frame_when_enabled() {
    // main: LDA #$80; STA $2000; loop: JMP loop ($8005)
    // nmi:  INC $10; RTI
    let r = rom(
        &[0xA9, 0x80, 0x8D, 0x00, 0x20, 0x4C, 0x05, 0x80],
        &[0xE6, 0x10, 0x40],
    );
    let mut nes = TestNes::from_ines(&r).unwrap();
    nes.run_frames(5).unwrap();
    assert_eq!(nes.frames, 5);
    // the NMI handler for the 5th frame may not have run yet; run a bit more
    nes.run_until_pc(0x8005, 100).unwrap();
    assert_eq!(nes.bus.ram[0x10], 5);
    assert_eq!(nes.cpu.sp, 0xFD);
}

#[test]
fn nmi_not_fired_after_disabling() {
    // LDA #$80; STA $2000; LDA #$00; STA $2000; loop
    let r = rom(
        &[0xA9, 0x80, 0x8D, 0x00, 0x20, 0xA9, 0x00, 0x8D, 0x00, 0x20, 0x4C, 0x0A, 0x80],
        &[0xE6, 0x10, 0x40],
    );
    let mut nes = TestNes::from_ines(&r).unwrap();
    nes.run_frames(2).unwrap();
    assert_eq!(nes.bus.ram[0x10], 0);
}

#[test]
fn run_until_pc_reaches_target_and_times_out() {
    // LDX #$05; loop: DEX; BNE loop; done: JMP done ($8005)
    let mut nes = machine(&[0xA2, 0x05, 0xCA, 0xD0, 0xFD, 0x4C, 0x05, 0x80]);
    nes.run_until_pc(0x8005, 1000).unwrap();
    assert_eq!(nes.cpu.x, 0);
    assert_eq!(nes.cpu.pc, 0x8005);

    let mut nes = machine(&[0x4C, 0x00, 0x80]);
    let err = nes.run_until_pc(0x9000, 1000).unwrap_err();
    assert!(err.to_lowercase().contains("timeout") || err.to_lowercase().contains("timed out"));
}

#[test]
fn illegal_opcode_is_reported() {
    let mut nes = machine(&[0x02]);
    assert!(nes.step().is_err());
    let mut nes = machine(&[0x02]);
    assert!(nes.run_until_pc(0x9000, 1000).is_err());
}

#[test]
fn counts_vram_writes_made_while_rendering_outside_vblank() {
    // Main: rendering on, NMI on, then a loop that writes $2007 forever.
    //   LDA #$18 / STA $2001 / LDA #$80 / STA $2000 / loop: STA $2007 / JMP loop
    let main = [0xA9, 0x18, 0x8D, 0x01, 0x20, 0xA9, 0x80, 0x8D, 0x00, 0x20, 0x8D, 0x07, 0x20, 0x4C, 0x0A, 0x80];
    // NMI: a few $2006/$2007 writes (inside vblank, fine) then RTI.
    let nmi = [0x8D, 0x06, 0x20, 0x8D, 0x06, 0x20, 0x8D, 0x07, 0x20, 0x40];
    let mut nes = TestNes::from_ines(&rom(&main, &nmi)).unwrap();
    nes.run_frames(2).unwrap();
    assert!(nes.bus.render_conflicts > 100, "main-loop writes are conflicts: {}", nes.bus.render_conflicts);
}

#[test]
fn vblank_and_rendering_off_writes_are_not_conflicts() {
    // Rendering off: unlimited $2007 writes are allowed.
    let main = [0x8D, 0x07, 0x20, 0x4C, 0x00, 0x80];
    let mut nes = machine(&main);
    nes.run_frames(2).unwrap();
    assert_eq!(nes.bus.render_conflicts, 0);

    // Rendering on, but only the NMI handler (which runs in vblank) touches VRAM.
    let main = [0xA9, 0x18, 0x8D, 0x01, 0x20, 0xA9, 0x80, 0x8D, 0x00, 0x20, 0x4C, 0x0A, 0x80];
    let nmi = [0x8D, 0x06, 0x20, 0x8D, 0x06, 0x20, 0x8D, 0x07, 0x20, 0x8D, 0x07, 0x20, 0x40];
    let mut nes = TestNes::from_ines(&rom(&main, &nmi)).unwrap();
    nes.run_frames(5).unwrap();
    assert_eq!(nes.bus.render_conflicts, 0);
}

#[test]
fn bus_new_starts_with_no_conflicts() {
    assert_eq!(NesBus::new(vec![], vec![]).render_conflicts, 0);
}

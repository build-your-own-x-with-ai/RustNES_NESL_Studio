//! Unit tests for the 6502 core, using a flat 64KB RAM bus.

use nes_sim::cpu::*;

struct Ram(Vec<u8>);

impl Bus for Ram {
    fn read(&mut self, addr: u16) -> u8 {
        self.0[addr as usize]
    }
    fn write(&mut self, addr: u16, val: u8) {
        self.0[addr as usize] = val;
    }
}

/// Builds a machine with `prog` loaded at $8000 and the reset vector pointing there.
fn setup(prog: &[u8]) -> (Cpu, Ram) {
    let mut ram = Ram(vec![0; 0x10000]);
    ram.0[0x8000..0x8000 + prog.len()].copy_from_slice(prog);
    ram.0[0xFFFC] = 0x00;
    ram.0[0xFFFD] = 0x80;
    let mut cpu = Cpu::new();
    cpu.reset(&mut ram);
    (cpu, ram)
}

fn run(cpu: &mut Cpu, ram: &mut Ram, n: usize) -> u32 {
    let mut total = 0;
    for _ in 0..n {
        total += cpu.step(ram).unwrap();
    }
    total
}

fn flag(cpu: &Cpu, f: u8) -> bool {
    cpu.p & f != 0
}

#[test]
fn reset_state() {
    let (cpu, _) = setup(&[]);
    assert_eq!(cpu.pc, 0x8000);
    assert_eq!(cpu.sp, 0xFD);
    assert_eq!(cpu.p, 0x24);
    assert!(flag(&cpu, FLAG_I));
}

#[test]
fn lda_imm_flags_and_cycles() {
    let (mut cpu, mut ram) = setup(&[0xA9, 0x00, 0xA9, 0x80, 0xA9, 0x01]);
    assert_eq!(cpu.step(&mut ram).unwrap(), 2);
    assert!(flag(&cpu, FLAG_Z) && !flag(&cpu, FLAG_N));
    cpu.step(&mut ram).unwrap();
    assert!(!flag(&cpu, FLAG_Z) && flag(&cpu, FLAG_N));
    cpu.step(&mut ram).unwrap();
    assert!(!flag(&cpu, FLAG_Z) && !flag(&cpu, FLAG_N));
    assert_eq!(cpu.a, 1);
}

#[test]
fn adc_flags() {
    // CLC; LDA #$50; ADC #$50  -> $A0, V set, N set, C clear
    let (mut cpu, mut ram) = setup(&[0x18, 0xA9, 0x50, 0x69, 0x50]);
    run(&mut cpu, &mut ram, 3);
    assert_eq!(cpu.a, 0xA0);
    assert!(flag(&cpu, FLAG_V) && flag(&cpu, FLAG_N) && !flag(&cpu, FLAG_C));

    // SEC; LDA #$FF; ADC #$00 -> $00, C set, Z set, V clear
    let (mut cpu, mut ram) = setup(&[0x38, 0xA9, 0xFF, 0x69, 0x00]);
    run(&mut cpu, &mut ram, 3);
    assert_eq!(cpu.a, 0x00);
    assert!(flag(&cpu, FLAG_C) && flag(&cpu, FLAG_Z) && !flag(&cpu, FLAG_V));
}

#[test]
fn adc_ignores_decimal_mode() {
    // SED; CLC; LDA #$09; ADC #$01 -> binary $0A
    let (mut cpu, mut ram) = setup(&[0xF8, 0x18, 0xA9, 0x09, 0x69, 0x01]);
    run(&mut cpu, &mut ram, 4);
    assert_eq!(cpu.a, 0x0A);
    assert!(flag(&cpu, FLAG_D));
}

#[test]
fn sbc_flags() {
    // SEC; LDA #$50; SBC #$F0 -> $60, C clear (borrow), V clear
    let (mut cpu, mut ram) = setup(&[0x38, 0xA9, 0x50, 0xE9, 0xF0]);
    run(&mut cpu, &mut ram, 3);
    assert_eq!(cpu.a, 0x60);
    assert!(!flag(&cpu, FLAG_C) && !flag(&cpu, FLAG_V));

    // SEC; LDA #$50; SBC #$B0 -> $A0, V set, C clear
    let (mut cpu, mut ram) = setup(&[0x38, 0xA9, 0x50, 0xE9, 0xB0]);
    run(&mut cpu, &mut ram, 3);
    assert_eq!(cpu.a, 0xA0);
    assert!(flag(&cpu, FLAG_V) && !flag(&cpu, FLAG_C) && flag(&cpu, FLAG_N));

    // SEC; LDA #$05; SBC #$05 -> 0, C set, Z set
    let (mut cpu, mut ram) = setup(&[0x38, 0xA9, 0x05, 0xE9, 0x05]);
    run(&mut cpu, &mut ram, 3);
    assert_eq!(cpu.a, 0);
    assert!(flag(&cpu, FLAG_C) && flag(&cpu, FLAG_Z));
}

#[test]
fn cmp_flags() {
    // LDA #$40; CMP #$30 / CMP #$40 / CMP #$50
    let (mut cpu, mut ram) = setup(&[0xA9, 0x40, 0xC9, 0x30, 0xC9, 0x40, 0xC9, 0x50]);
    run(&mut cpu, &mut ram, 2);
    assert!(flag(&cpu, FLAG_C) && !flag(&cpu, FLAG_Z) && !flag(&cpu, FLAG_N));
    run(&mut cpu, &mut ram, 1);
    assert!(flag(&cpu, FLAG_C) && flag(&cpu, FLAG_Z));
    run(&mut cpu, &mut ram, 1);
    assert!(!flag(&cpu, FLAG_C) && !flag(&cpu, FLAG_Z) && flag(&cpu, FLAG_N));
}

#[test]
fn cpx_cpy() {
    // LDX #$10; CPX #$10; LDY #$01; CPY #$02
    let (mut cpu, mut ram) = setup(&[0xA2, 0x10, 0xE0, 0x10, 0xA0, 0x01, 0xC0, 0x02]);
    run(&mut cpu, &mut ram, 2);
    assert!(flag(&cpu, FLAG_Z) && flag(&cpu, FLAG_C));
    run(&mut cpu, &mut ram, 2);
    assert!(!flag(&cpu, FLAG_C) && flag(&cpu, FLAG_N));
}

#[test]
fn bit_flags() {
    // LDA #$01; BIT $10 where $10 = $C0 -> Z set, N set, V set
    let (mut cpu, mut ram) = setup(&[0xA9, 0x01, 0x24, 0x10]);
    ram.0[0x10] = 0xC0;
    run(&mut cpu, &mut ram, 2);
    assert!(flag(&cpu, FLAG_Z) && flag(&cpu, FLAG_N) && flag(&cpu, FLAG_V));
    assert_eq!(cpu.a, 1);
}

#[test]
fn shifts_and_rotates() {
    // LDA #$81; ASL A -> $02, C=1
    let (mut cpu, mut ram) = setup(&[0xA9, 0x81, 0x0A]);
    run(&mut cpu, &mut ram, 2);
    assert_eq!(cpu.a, 0x02);
    assert!(flag(&cpu, FLAG_C));

    // LDA #$01; LSR A -> 0, C=1, Z=1
    let (mut cpu, mut ram) = setup(&[0xA9, 0x01, 0x4A]);
    run(&mut cpu, &mut ram, 2);
    assert_eq!(cpu.a, 0);
    assert!(flag(&cpu, FLAG_C) && flag(&cpu, FLAG_Z));

    // SEC; LDA #$80; ROL A -> $01, C=1
    let (mut cpu, mut ram) = setup(&[0x38, 0xA9, 0x80, 0x2A]);
    run(&mut cpu, &mut ram, 3);
    assert_eq!(cpu.a, 0x01);
    assert!(flag(&cpu, FLAG_C));

    // SEC; LDA #$01; ROR A -> $80, C=1, N=1
    let (mut cpu, mut ram) = setup(&[0x38, 0xA9, 0x01, 0x6A]);
    run(&mut cpu, &mut ram, 3);
    assert_eq!(cpu.a, 0x80);
    assert!(flag(&cpu, FLAG_C) && flag(&cpu, FLAG_N));

    // ASL $20 (memory, 5 cycles)
    let (mut cpu, mut ram) = setup(&[0x06, 0x20]);
    ram.0[0x20] = 0x40;
    assert_eq!(cpu.step(&mut ram).unwrap(), 5);
    assert_eq!(ram.0[0x20], 0x80);
    assert!(flag(&cpu, FLAG_N) && !flag(&cpu, FLAG_C));
}

#[test]
fn inc_dec_memory() {
    let (mut cpu, mut ram) = setup(&[0xE6, 0x20, 0xCE, 0x00, 0x03]);
    ram.0[0x20] = 0xFF;
    ram.0[0x300] = 0x01;
    assert_eq!(cpu.step(&mut ram).unwrap(), 5);
    assert_eq!(ram.0[0x20], 0);
    assert!(flag(&cpu, FLAG_Z));
    assert_eq!(cpu.step(&mut ram).unwrap(), 6);
    assert_eq!(ram.0[0x300], 0);
}

#[test]
fn branch_cycles() {
    // LDA #$00 ; BEQ +2 (taken, same page) => 3 cycles
    let (mut cpu, mut ram) = setup(&[0xA9, 0x00, 0xF0, 0x02, 0, 0, 0xEA]);
    run(&mut cpu, &mut ram, 1);
    assert_eq!(cpu.step(&mut ram).unwrap(), 3);
    assert_eq!(cpu.pc, 0x8006);

    // not taken => 2 cycles
    let (mut cpu, mut ram) = setup(&[0xA9, 0x01, 0xF0, 0x02]);
    run(&mut cpu, &mut ram, 1);
    assert_eq!(cpu.step(&mut ram).unwrap(), 2);
    assert_eq!(cpu.pc, 0x8004);

    // backward branch crossing a page => 4 cycles
    let (mut cpu, mut ram) = setup(&[0xA9, 0x01, 0xD0, 0xF0]); // BNE -16 from $8004 -> $7FF4
    run(&mut cpu, &mut ram, 1);
    assert_eq!(cpu.step(&mut ram).unwrap(), 4);
    assert_eq!(cpu.pc, 0x7FF4);
}

#[test]
fn page_cross_penalty_on_indexed_reads() {
    // LDX #$01; LDA $80FF,X (crosses) ; LDA $8000,X (no cross); STA $80FF,X (always 5)
    let (mut cpu, mut ram) = setup(&[0xA2, 0x01, 0xBD, 0xFF, 0x80, 0xBD, 0x00, 0x80, 0x9D, 0xFF, 0x02]);
    run(&mut cpu, &mut ram, 1);
    assert_eq!(cpu.step(&mut ram).unwrap(), 5);
    assert_eq!(cpu.step(&mut ram).unwrap(), 4);
    assert_eq!(cpu.step(&mut ram).unwrap(), 5);

    // (zp),Y crossing
    let (mut cpu, mut ram) = setup(&[0xA0, 0x01, 0xB1, 0x10]);
    ram.0[0x10] = 0xFF;
    ram.0[0x11] = 0x02;
    ram.0[0x300] = 0x77;
    run(&mut cpu, &mut ram, 1);
    assert_eq!(cpu.step(&mut ram).unwrap(), 6);
    assert_eq!(cpu.a, 0x77);
}

#[test]
fn zero_page_index_wraps() {
    // LDX #$FF; LDA $10,X -> reads $0F
    let (mut cpu, mut ram) = setup(&[0xA2, 0xFF, 0xB5, 0x10]);
    ram.0[0x0F] = 0x42;
    run(&mut cpu, &mut ram, 2);
    assert_eq!(cpu.a, 0x42);
}

#[test]
fn indexed_indirect_pointer_wraps() {
    // LDX #$00; LDA ($FF,X) -> pointer lo at $FF, hi at $00
    let (mut cpu, mut ram) = setup(&[0xA2, 0x00, 0xA1, 0xFF]);
    ram.0[0xFF] = 0x34;
    ram.0[0x00] = 0x02;
    ram.0[0x0234] = 0x99;
    run(&mut cpu, &mut ram, 2);
    assert_eq!(cpu.a, 0x99);
}

#[test]
fn stack_push_pull() {
    // LDA #$42; PHA; LDA #$00; PLA
    let (mut cpu, mut ram) = setup(&[0xA9, 0x42, 0x48, 0xA9, 0x00, 0x68]);
    run(&mut cpu, &mut ram, 2);
    assert_eq!(cpu.sp, 0xFC);
    assert_eq!(ram.0[0x1FD], 0x42);
    run(&mut cpu, &mut ram, 1);
    assert_eq!(cpu.step(&mut ram).unwrap(), 4);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cpu.sp, 0xFD);
}

#[test]
fn php_sets_b_and_plp_ignores_it() {
    // PHP; PLA  -> pushed value has B and U set
    let (mut cpu, mut ram) = setup(&[0x08, 0x68]);
    run(&mut cpu, &mut ram, 2);
    assert_eq!(cpu.a, 0x24 | FLAG_B | FLAG_U);

    // LDA #$FF; PHA; PLP -> p = $EF (B clear, U set)
    let (mut cpu, mut ram) = setup(&[0xA9, 0xFF, 0x48, 0x28]);
    run(&mut cpu, &mut ram, 3);
    assert_eq!(cpu.p, 0xEF);
    // LDA #$00; PHA; PLP -> p = $20
    let (mut cpu, mut ram) = setup(&[0xA9, 0x00, 0x48, 0x28]);
    run(&mut cpu, &mut ram, 3);
    assert_eq!(cpu.p, 0x20);
}

#[test]
fn jsr_rts() {
    // $8000: JSR $8010 ; $8003: NOP ; $8010: RTS
    let mut prog = vec![0u8; 0x20];
    prog[0..3].copy_from_slice(&[0x20, 0x10, 0x80]);
    prog[3] = 0xEA;
    prog[0x10] = 0x60;
    let (mut cpu, mut ram) = setup(&prog);
    assert_eq!(cpu.step(&mut ram).unwrap(), 6);
    assert_eq!(cpu.pc, 0x8010);
    assert_eq!(ram.0[0x1FD], 0x80);
    assert_eq!(ram.0[0x1FC], 0x02);
    assert_eq!(cpu.step(&mut ram).unwrap(), 6);
    assert_eq!(cpu.pc, 0x8003);
}

#[test]
fn jmp_indirect_page_wrap_bug() {
    // JMP ($02FF) -> lo from $02FF, hi from $0200 (not $0300)
    let (mut cpu, mut ram) = setup(&[0x6C, 0xFF, 0x02]);
    ram.0[0x02FF] = 0x34;
    ram.0[0x0200] = 0x12;
    ram.0[0x0300] = 0x56;
    assert_eq!(cpu.step(&mut ram).unwrap(), 5);
    assert_eq!(cpu.pc, 0x1234);
}

#[test]
fn brk_and_rti() {
    let (mut cpu, mut ram) = setup(&[0x00, 0xFF]);
    ram.0[0xFFFE] = 0x00;
    ram.0[0xFFFF] = 0x90;
    ram.0[0x9000] = 0x40; // RTI
    assert_eq!(cpu.step(&mut ram).unwrap(), 7);
    assert_eq!(cpu.pc, 0x9000);
    // pushed: pc+2 = $8002, then p with B set
    assert_eq!(ram.0[0x1FD], 0x80);
    assert_eq!(ram.0[0x1FC], 0x02);
    assert_eq!(ram.0[0x1FB], 0x24 | FLAG_B | FLAG_U);
    assert!(flag(&cpu, FLAG_I));
    assert_eq!(cpu.step(&mut ram).unwrap(), 6);
    assert_eq!(cpu.pc, 0x8002);
    assert_eq!(cpu.p, 0x24);
    assert_eq!(cpu.sp, 0xFD);
}

#[test]
fn nmi_pushes_state() {
    let (mut cpu, mut ram) = setup(&[0x18, 0x58]); // CLC; CLI
    ram.0[0xFFFA] = 0x00;
    ram.0[0xFFFB] = 0xA0;
    ram.0[0xA000] = 0x40; // RTI
    run(&mut cpu, &mut ram, 2);
    let before = cpu.cycles;
    cpu.nmi(&mut ram);
    assert_eq!(cpu.cycles - before, 7);
    assert_eq!(cpu.pc, 0xA000);
    assert_eq!(ram.0[0x1FD], 0x80);
    assert_eq!(ram.0[0x1FC], 0x02);
    assert_eq!(ram.0[0x1FB] & FLAG_B, 0);
    assert_ne!(ram.0[0x1FB] & FLAG_U, 0);
    assert_eq!(ram.0[0x1FB] & FLAG_I, 0);
    assert!(flag(&cpu, FLAG_I));
    cpu.step(&mut ram).unwrap();
    assert_eq!(cpu.pc, 0x8002);
    assert!(!flag(&cpu, FLAG_I));
}

#[test]
fn transfers() {
    // LDA #$80; TAX; TAY; LDX #$00; TXS; TSX ; TXA
    let (mut cpu, mut ram) = setup(&[0xA9, 0x80, 0xAA, 0xA8, 0xA2, 0x05, 0x9A, 0xA2, 0x00, 0xBA, 0x8A]);
    run(&mut cpu, &mut ram, 3);
    assert_eq!((cpu.x, cpu.y), (0x80, 0x80));
    run(&mut cpu, &mut ram, 2);
    assert_eq!(cpu.sp, 0x05);
    run(&mut cpu, &mut ram, 2);
    assert_eq!(cpu.x, 0x05);
    run(&mut cpu, &mut ram, 1);
    assert_eq!(cpu.a, 0x05);
    assert!(!flag(&cpu, FLAG_N));
}

#[test]
fn illegal_opcode_errors() {
    let (mut cpu, mut ram) = setup(&[0x02]);
    assert_eq!(
        cpu.step(&mut ram),
        Err(CpuError::IllegalOpcode { opcode: 0x02, pc: 0x8000 })
    );
}

#[test]
fn cycles_accumulate() {
    let (mut cpu, mut ram) = setup(&[0xEA, 0xEA]);
    let start = cpu.cycles;
    run(&mut cpu, &mut ram, 2);
    assert_eq!(cpu.cycles - start, 4);
}

//! 010: assembler Equ (define a symbol without emitting bytes).
use nesl::asm::{assemble, Item, Op::*, Operand};
use nesl::codegen::Emitter;

#[test]
fn equ_defines_a_symbol_at_a_fixed_address() {
    let mut e = Emitter::new();
    e.label("start");
    e.imm(LDA, 0x42);
    e.items.push(Item::Equ("elsewhere".into(), 0xC123));
    e.ins(STA, Operand::abs("elsewhere"));
    let asm = assemble(&e.items, 0x8000).unwrap();
    assert_eq!(asm.symbols["start"], 0x8000);
    assert_eq!(asm.symbols["elsewhere"], 0xC123);
    assert_eq!(asm.code.len(), 5, "LDA #$42 (2) + STA $C123 (3)");
    assert_eq!(&asm.code[2..5], &[0x8D, 0x23, 0xC1]);
}

#[test]
fn equ_can_appear_before_or_after_references() {
    let mut e = Emitter::new();
    e.items.push(Item::Equ("before".into(), 0x1234));
    e.ins(LDA, Operand::abs("before"));
    e.ins(LDA, Operand::abs("after"));
    e.items.push(Item::Equ("after".into(), 0x5678));
    let asm = assemble(&e.items, 0x8000).unwrap();
    assert_eq!(&asm.code, &[0xAD, 0x34, 0x12, 0xAD, 0x78, 0x56]);
}

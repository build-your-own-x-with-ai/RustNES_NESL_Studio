//! 010: assembler Equ (define a symbol without emitting bytes).
use nesl::asm::{assemble, Imm, Item, Op::*, Operand, Val};

#[test]
fn equ_defines_a_symbol_at_a_fixed_address() {
    let items = vec![
        Item::Label("start".into()),
        Item::Ins(LDA, Operand::Imm(Imm::Num(0x42))),
        Item::Equ("elsewhere".into(), 0xC123),
        Item::Ins(STA, Operand::Abs(Val::Label("elsewhere".into(), 0))),
    ];
    let asm = assemble(&items, 0x8000).unwrap();
    assert_eq!(asm.symbols["start"], 0x8000);
    assert_eq!(asm.symbols["elsewhere"], 0xC123);
    assert_eq!(asm.code.len(), 5, "LDA #$42 (2) + STA $C123 (3)");
    assert_eq!(&asm.code[2..5], &[0x8D, 0x23, 0xC1]);
}

#[test]
fn equ_can_appear_before_or_after_references() {
    let items = vec![
        Item::Equ("before".into(), 0x1234),
        Item::Ins(LDA, Operand::Abs(Val::Label("before".into(), 0))),
        Item::Ins(LDA, Operand::Abs(Val::Label("after".into(), 0))),
        Item::Equ("after".into(), 0x5678),
    ];
    let asm = assemble(&items, 0x8000).unwrap();
    assert_eq!(&asm.code, &[0xAD, 0x34, 0x12, 0xAD, 0x78, 0x56]);
}

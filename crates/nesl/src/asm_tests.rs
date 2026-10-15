use super::*;

fn asm(items: Vec<Item>) -> Assembled {
    assemble(&items, 0x8000).expect("assemble")
}

#[test]
fn encodes_implied_and_immediate() {
    let out = asm(vec![
        Item::ins(Op::SEI, Operand::Imp),
        Item::ins(Op::LDA, Operand::imm(0x42)),
        Item::ins(Op::ASL, Operand::Acc),
        Item::ins(Op::RTS, Operand::Imp),
    ]);
    assert_eq!(out.code, vec![0x78, 0xA9, 0x42, 0x0A, 0x60]);
}

#[test]
fn picks_zero_page_form_for_small_numeric_addresses() {
    let out = asm(vec![
        Item::ins(Op::STA, Operand::abs(0x10)),
        Item::ins(Op::STA, Operand::abs(0x0300)),
        Item::ins(Op::LDA, Operand::abs_x(0x20)),
        Item::ins(Op::LDX, Operand::abs_y(0x30)),
        // STA has no zp,Y form: must fall back to absolute,Y
        Item::ins(Op::STA, Operand::abs_y(0x40)),
    ]);
    assert_eq!(
        out.code,
        vec![0x85, 0x10, 0x8D, 0x00, 0x03, 0xB5, 0x20, 0xB6, 0x30, 0x99, 0x40, 0x00]
    );
}

#[test]
fn resolves_labels_and_label_bytes() {
    let out = asm(vec![
        Item::ins(Op::JMP, Operand::label("end")),
        Item::ins(Op::LDA, Operand::imm_lo("data")),
        Item::ins(Op::LDX, Operand::imm_hi("data")),
        Item::Label("data".into()),
        Item::Bytes(vec![1, 2]),
        Item::Label("end".into()),
        Item::ins(Op::LDA, Operand::Abs(Val::Label("data".into(), 1))),
    ]);
    // JMP $8009 ; LDA #$07 ; LDX #$80 ; .byte 1,2 ; LDA $8008
    assert_eq!(
        out.code,
        vec![0x4C, 0x09, 0x80, 0xA9, 0x07, 0xA2, 0x80, 1, 2, 0xAD, 0x08, 0x80]
    );
    assert_eq!(out.symbols["data"], 0x8007);
    assert_eq!(out.symbols["end"], 0x8009);
}

#[test]
fn short_branches_encode_relative_offsets() {
    let out = asm(vec![
        Item::Label("top".into()),
        Item::ins(Op::DEX, Operand::Imp),
        Item::Branch(Op::BNE, "top".into()),
        Item::Branch(Op::BEQ, "next".into()),
        Item::Label("next".into()),
    ]);
    assert_eq!(out.code, vec![0xCA, 0xD0, 0xFD, 0xF0, 0x00]);
}

#[test]
fn far_branches_are_relaxed_into_inverted_branch_plus_jmp() {
    let mut items = vec![Item::Branch(Op::BEQ, "far".into())];
    items.push(Item::Bytes(vec![0xEA; 200]));
    items.push(Item::Label("far".into()));
    let out = asm(items);
    // BNE +3 ; JMP far
    let far = 0x8000 + 5 + 200;
    assert_eq!(&out.code[..5], &[0xD0, 0x03, 0x4C, (far & 0xFF) as u8, (far >> 8) as u8]);
    assert_eq!(out.symbols["far"], far as u16);
}

#[test]
fn indirect_modes() {
    let out = asm(vec![
        Item::ins(Op::LDA, Operand::IndY(0x04)),
        Item::ins(Op::STA, Operand::IndX(0x06)),
        Item::ins(Op::JMP, Operand::Ind(Val::Num(0x0300))),
    ]);
    assert_eq!(out.code, vec![0xB1, 0x04, 0x81, 0x06, 0x6C, 0x00, 0x03]);
}

#[test]
fn errors_on_unknown_label_duplicate_label_and_invalid_mode() {
    assert!(assemble(&[Item::ins(Op::JMP, Operand::label("nowhere"))], 0x8000)
        .unwrap_err()
        .contains("nowhere"));
    assert!(assemble(&[Item::Label("a".into()), Item::Label("a".into())], 0x8000)
        .unwrap_err()
        .contains("duplicate"));
    assert!(assemble(&[Item::ins(Op::STA, Operand::imm(1))], 0x8000).is_err());
}

#[test]
fn listing_contains_addresses_and_mnemonics() {
    let out = asm(vec![
        Item::Label("start".into()),
        Item::ins(Op::LDA, Operand::imm(5)),
    ]);
    assert!(out.listing.contains("start:"));
    assert!(out.listing.contains("8000"));
    assert!(out.listing.contains("LDA #$05"));
}

//! Runtime library linked into every ROM: reset/NMI handlers and helper routines.
//! All routines are expressed as assembler IR, so they share the program's labels.

use crate::asm::{Item, Op, Op::*, Operand};

// Zero-page locations reserved for the runtime ($00-$15). User variables start at $16.
pub const TMP0: u16 = 0x00;
pub const TMP1: u16 = 0x01;
pub const TMP2: u16 = 0x02;
pub const TMP3: u16 = 0x03;
pub const PTR: u16 = 0x04; // 2 bytes
pub const PTR2: u16 = 0x06; // 2 bytes
pub const FRAME: u16 = 0x08;
pub const CTRL_SHADOW: u16 = 0x09;
pub const MASK_SHADOW: u16 = 0x0A;
pub const SCROLL_X: u16 = 0x0B;
pub const SCROLL_Y: u16 = 0x0C;
pub const RNG: u16 = 0x0D;
pub const TMP4: u16 = 0x0E;
pub const TMP5: u16 = 0x0F;
// Argument slots for built-in routines.
pub const ARG0: u16 = 0x10;
pub const ARG1: u16 = 0x11;
pub const ARG2: u16 = 0x12;
pub const ARG3: u16 = 0x13;
pub const ARG4: u16 = 0x14;
pub const USER_ZP_START: u16 = 0x16;
pub const OAM_SHADOW: u16 = 0x0200;
pub const PRINT_BUF: u16 = 0x0300; // 3 bytes used by print_u8
pub const USER_RAM_START: u16 = 0x0310;
pub const USER_RAM_END: u16 = 0x0800;

pub const PPUCTRL: u16 = 0x2000;
pub const PPUMASK: u16 = 0x2001;
pub const PPUSTATUS: u16 = 0x2002;
pub const OAMADDR: u16 = 0x2003;
pub const PPUSCROLL: u16 = 0x2005;
pub const PPUADDR: u16 = 0x2006;
pub const PPUDATA: u16 = 0x2007;
pub const OAMDMA: u16 = 0x4014;
pub const JOY1: u16 = 0x4016;

/// Small builder used by the runtime and the code generator.
#[derive(Default)]
pub struct Emitter {
    pub items: Vec<Item>,
    next_label: usize,
}

impl Emitter {
    pub fn ins(&mut self, op: Op, o: Operand) {
        self.items.push(Item::Ins(op, o));
    }
    pub fn imp(&mut self, op: Op) {
        self.ins(op, Operand::Imp);
    }
    pub fn acc(&mut self, op: Op) {
        self.ins(op, Operand::Acc);
    }
    pub fn imm(&mut self, op: Op, v: u8) {
        self.ins(op, Operand::imm(v));
    }
    pub fn abs(&mut self, op: Op, a: u16) {
        self.ins(op, Operand::abs(a));
    }
    pub fn label(&mut self, l: &str) {
        self.items.push(Item::Label(l.to_string()));
    }
    pub fn br(&mut self, op: Op, l: &str) {
        self.items.push(Item::Branch(op, l.to_string()));
    }
    pub fn jmp(&mut self, l: &str) {
        self.ins(JMP, Operand::label(l));
    }
    pub fn jsr(&mut self, l: &str) {
        self.ins(JSR, Operand::label(l));
    }
    pub fn comment(&mut self, c: impl Into<String>) {
        self.items.push(Item::Comment(c.into()));
    }
    pub fn new_label(&mut self) -> String {
        self.next_label += 1;
        format!("L{}", self.next_label)
    }
}

/// Emits reset, NMI and IRQ handlers. `__init_globals` and `f_main` are provided by codegen.
pub fn emit_core(e: &mut Emitter) {
    e.comment("---- runtime: reset ----");
    e.label("__reset");
    e.imp(SEI);
    e.imp(CLD);
    e.imm(LDX, 0x40);
    e.abs(STX, 0x4017); // disable APU frame IRQ
    e.imm(LDX, 0xFF);
    e.imp(TXS);
    e.imp(INX);
    e.abs(STX, PPUCTRL);
    e.abs(STX, PPUMASK);
    e.abs(STX, 0x4010); // disable DMC IRQ
    e.abs(BIT, PPUSTATUS);
    e.label("__vb1");
    e.abs(BIT, PPUSTATUS);
    e.br(BPL, "__vb1");
    e.imm(LDA, 0);
    e.label("__clr");
    for page in [0x0000u16, 0x0100, 0x0300, 0x0400, 0x0500, 0x0600, 0x0700] {
        e.ins(STA, Operand::abs_x(page));
    }
    e.imp(INX);
    e.br(BNE, "__clr");
    e.imm(LDA, 0xFF); // hide all sprites (y = $FF)
    e.label("__clr_oam");
    e.ins(STA, Operand::abs_x(OAM_SHADOW));
    e.imp(INX);
    e.br(BNE, "__clr_oam");
    e.label("__vb2");
    e.abs(BIT, PPUSTATUS);
    e.br(BPL, "__vb2");
    e.imm(LDA, 0xA5);
    e.abs(STA, RNG);
    e.imm(LDA, 0x80); // NMI on, both pattern tables at $0000
    e.abs(STA, CTRL_SHADOW);
    e.abs(STA, PPUCTRL);
    e.jsr("__init_globals");
    e.jsr("f_main");
    e.label("__halt");
    e.jmp("__halt");

    e.comment("---- runtime: NMI ----");
    e.label("__nmi");
    e.imp(PHA);
    e.imp(TXA);
    e.imp(PHA);
    e.imp(TYA);
    e.imp(PHA);
    // Only touch PPU registers while rendering is enabled; with rendering off the main
    // program may be in the middle of $2006/$2007 writes.
    e.abs(LDA, MASK_SHADOW);
    e.imm(AND, 0x18);
    e.br(BEQ, "__nmi_skip");
    e.imm(LDA, 0);
    e.abs(STA, OAMADDR);
    e.imm(LDA, (OAM_SHADOW >> 8) as u8);
    e.abs(STA, OAMDMA);
    e.abs(LDA, CTRL_SHADOW);
    e.abs(STA, PPUCTRL);
    e.abs(LDA, SCROLL_X);
    e.abs(STA, PPUSCROLL);
    e.abs(LDA, SCROLL_Y);
    e.abs(STA, PPUSCROLL);
    e.label("__nmi_skip");
    e.abs(INC, FRAME);
    e.imp(PLA);
    e.imp(TAY);
    e.imp(PLA);
    e.imp(TAX);
    e.imp(PLA);
    e.label("__irq");
    e.imp(RTI);
}

/// Math helpers used by `*`, `/` and `%`.
pub fn emit_math(e: &mut Emitter) {
    e.comment("---- runtime: A = TMP0 * TMP1 (mod 256) ----");
    e.label("__mul8");
    e.imm(LDA, 0);
    e.imm(LDX, 8);
    e.label("__mul8_loop");
    e.acc(ASL);
    e.abs(ASL, TMP0);
    e.br(BCC, "__mul8_skip");
    e.imp(CLC);
    e.abs(ADC, TMP1);
    e.label("__mul8_skip");
    e.imp(DEX);
    e.br(BNE, "__mul8_loop");
    e.imp(RTS);

    e.comment("---- runtime: A = TMP0 / TMP1, TMP2 = TMP0 % TMP1 (x/0 = $FF rem x) ----");
    e.label("__div8");
    e.abs(LDA, TMP1);
    e.br(BNE, "__div8_ok");
    e.abs(LDA, TMP0);
    e.abs(STA, TMP2);
    e.imm(LDA, 0xFF);
    e.imp(RTS);
    e.label("__div8_ok");
    e.imm(LDA, 0);
    e.imm(LDX, 8);
    e.label("__div8_loop");
    e.abs(ASL, TMP0);
    e.acc(ROL);
    e.br(BCS, "__div8_sub");
    e.abs(CMP, TMP1);
    e.br(BCC, "__div8_skip");
    e.label("__div8_sub");
    e.imp(SEC);
    e.abs(SBC, TMP1);
    e.abs(INC, TMP0);
    e.label("__div8_skip");
    e.imp(DEX);
    e.br(BNE, "__div8_loop");
    e.abs(STA, TMP2);
    e.abs(LDA, TMP0);
    e.imp(RTS);
}

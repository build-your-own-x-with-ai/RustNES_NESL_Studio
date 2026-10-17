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
/// Committed length of the VRAM update queue.
pub const VQ_LEN: u16 = 0x15;
/// Zero-page pointer used only inside the NMI (the main program may be using PTR).
pub const NMI_PTR: u16 = 0x16;
pub const USER_ZP_START: u16 = 0x18;
pub const OAM_SHADOW: u16 = 0x0200;
/// VRAM update queue: entries of `[addr_hi, addr_lo, len, bytes...]`, flushed by NMI.
pub const VQ: u16 = 0x0300;
pub const VQ_SIZE: u8 = 96;
pub const PRINT_BUF: u16 = 0x0360; // 3 bytes used by print_u8 / set_tile
/// Non-zero while the main program is appending to the queue; NMI then skips the flush.
pub const VQ_LOCK: u16 = 0x0363;
// Sound effect state, advanced once per frame by the NMI.
pub const SFX_TIMER: u16 = 0x0364;
pub const SFX_CH: u16 = 0x0365; // 0 = pulse 2, otherwise noise
pub const SFX_PLO: u16 = 0x0366;
pub const SFX_DELTA: u16 = 0x0367;
pub const SFX_MAXVOL: u16 = 0x0368;
pub const SFX_DUTY: u16 = 0x0369;
/// Shadow of $4015 (bit 0 = play_tone's pulse 1, bit 1 = pulse 2, bit 3 = noise).
pub const APU_EN: u16 = 0x036A;
// Camera / streaming state for wide maps.
pub const CAM_X: u16 = 0x036B; // 2 bytes, world x of the left screen edge
pub const CAM_COL: u16 = 0x036D;
pub const MAP_T: u16 = 0x036E; // 2 bytes, tile columns of the followed map
pub const MAP_A: u16 = 0x0370; // 2 bytes, attribute groups of the followed map
pub const MAP_W: u16 = 0x0372;
pub const SC_COL: u16 = 0x0373;
pub const SC_G: u16 = 0x0374;
pub const SC_R: u16 = 0x0375;
/// PPUCTRL / scroll x last applied by the NMI; reused while the main program is
/// mid-update (VQ_LOCK set) so the two halves of a scroll change never tear.
pub const LAST_CTRL: u16 = 0x0376;
pub const LAST_SX: u16 = 0x0377;
// Scratch for map address arithmetic.
pub const CA_LO: u16 = 0x0378;
pub const CA_HI: u16 = 0x0379;
pub const SUB_LO: u16 = 0x037A;
pub const SUB_HI: u16 = 0x037B;
pub const NGROUPS: u16 = 0x037C;
pub const AG: u16 = 0x037D;
/// Music voices (melody / bass / harmony), 8 bytes each:
/// `[ptr lo, ptr hi, start lo, start hi, timer, volume, duty bits, -]`.
pub const VOICES: [u16; 3] = [0x0380, 0x0388, 0x0390];
pub const MU_ON: u16 = 0x0398; // bit per playing voice
pub const MU_NOTE: u16 = 0x0399;
pub const MU_ARG: u16 = 0x039A;
// Editable maps (006) and the status bar.
pub const OV_COUNT: u16 = 0x039B;
pub const STATUS_ROWS: u16 = 0x039C;
/// Map drawn / followed last (1-based declaration number, 0 = none).
pub const MAP_ID: u16 = 0x039D;
pub const OV_RES: u16 = 0x039E;
/// The change list below is only reserved (user RAM moves to `OV_END`) when the
/// program calls `set_map_tile`; otherwise `OV_COUNT` stays 0 and it is never touched.
pub const COL_BUF: u16 = 0x03A0; // 30 bytes
pub const OV_MAX: u8 = 64;
pub const OV_ID: u16 = 0x03BE;
pub const OV_COL: u16 = OV_ID + OV_MAX as u16;
pub const OV_ROW: u16 = OV_COL + OV_MAX as u16;
pub const OV_TILE: u16 = OV_ROW + OV_MAX as u16;
pub const OV_END: u16 = OV_TILE + OV_MAX as u16;
pub const USER_RAM_START: u16 = 0x03A0;
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
    e.abs(STA, LAST_CTRL);
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
    // VRAM writes first: they must finish inside vblank.
    e.abs(LDA, VQ_LOCK);
    e.br(BNE, "__nmi_no_flush");
    e.jsr("__vq_flush");
    e.label("__nmi_no_flush");
    e.imm(LDA, 0);
    e.abs(STA, OAMADDR);
    e.imm(LDA, (OAM_SHADOW >> 8) as u8);
    e.abs(STA, OAMDMA);
    e.abs(LDA, VQ_LOCK);
    e.br(BNE, "__nmi_last_scroll");
    e.abs(LDA, CTRL_SHADOW);
    e.abs(STA, LAST_CTRL);
    e.abs(LDA, SCROLL_X);
    e.abs(STA, LAST_SX);
    e.label("__nmi_last_scroll");
    e.abs(LDA, LAST_CTRL);
    e.abs(STA, PPUCTRL);
    e.abs(LDA, LAST_SX);
    e.abs(STA, PPUSCROLL);
    e.abs(LDA, SCROLL_Y);
    e.abs(STA, PPUSCROLL);
    e.label("__nmi_skip");
    e.jsr("__sfx_tick");
    e.jsr("__music_tick");
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

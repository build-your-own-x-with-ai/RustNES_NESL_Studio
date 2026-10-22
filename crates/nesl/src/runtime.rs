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
pub const VQ_SIZE: u8 = 91;
pub const PRINT_BUF: u16 = 0x035B; // 5 bytes used by print_u8 / print_u16 / set_tile
/// Map width for `set_map_tile` (its column takes ARG0:ARG1).
pub const SMT_W: u16 = 0x0360;
/// Map chosen through a map list (`L[i]`): its id and width, set by `__map_sel`.
pub const SEL_ID: u16 = 0x0361;
pub const SEL_W: u16 = 0x0362;
/// Map's bank number for dynamic map list selection (banked mode only).
pub const SEL_BANK: u16 = 0x037E;
/// MMC3: scanline for `fn irq()` (0 = off), re-armed by NMI every frame.
pub const IRQ_LINE: u16 = 0x037F;
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
pub const STATUS_TMP: u16 = 0x039F;
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
/// Emits __bank(A) and __chr_copy for UNROM mapper 2.
pub fn emit_banked(e: &mut Emitter) {
    use crate::asm::{Item, Op::*, Operand, Val};
    
    e.comment("---- UNROM bank switching ----");
    e.label("__bank");
    // Bus-conflict-safe: writes to __bank_table+A where ROM[addr]=A
    e.imp(TAX);
    e.ins(STA, Operand::AbsX(Val::Label("__bank_table".into(), 0)));
    e.imp(RTS);
    
    // Bank table: 16 bytes where byte N = N
    e.label("__bank_table");
    e.items.push(Item::Bytes((0..16u8).collect()));
    
    // Copy 8KB CHR data from bank 0 at $8000 into PPU $0000-$1FFF
    e.comment("---- CHR copy (bank 0 -> CHR-RAM) ----");
    e.label("__chr_copy");
    e.imm(LDA, 0); // select bank 0
    e.jsr("__bank");
    e.imm(LDA, 0); // PPUCTRL: +1 increments, NMI still off
    e.abs(STA, PPUCTRL);
    e.imm(LDA, 0);
    e.abs(STA, PPUADDR);
    e.abs(STA, PPUADDR); // PPU addr = $0000
    e.imm(LDA, 0x80); // source = $8000
    e.abs(STA, PTR + 1);
    e.imm(LDA, 0);
    e.abs(STA, PTR);
    e.imm(LDX, 0x20); // copy $2000 bytes = 32 pages
    e.label("__chr_page");
    e.imm(LDY, 0);
    e.label("__chr_loop");
    e.ins(LDA, Operand::IndY(PTR as u8));
    e.abs(STA, PPUDATA);
    e.imp(INY);
    e.br(BNE, "__chr_loop");
    e.ins(INC, Operand::Abs(Val::Num(PTR + 1)));
    e.imp(DEX);
    e.br(BNE, "__chr_page");
    e.imp(RTS);
}

/// Zero-page scratch that `fn irq()` may clobber and must give back to the interrupted code.
const IRQ_SAVED: [std::ops::Range<u16>; 2] = [TMP0..PTR2 + 2, TMP4..ARG4 + 1];

pub fn emit_core(e: &mut Emitter, banked: bool, mmc3: bool) {
    e.comment("---- runtime: reset ----");
    e.label("__reset");
    e.imp(SEI);
    e.imp(CLD);
    if mmc3 {
        e.abs(STA, 0xE000); // MMC3 IRQ off until scanline_irq()
    }
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
    // NMI on; NROM/UNROM draw everything from $0000. MMC3 fetches sprites from $1000 (a copy)
    // so PPU A12 rises once per line and clocks the scanline counter.
    e.imm(LDA, if mmc3 { 0x88 } else { 0x80 });
    e.abs(STA, CTRL_SHADOW);
    e.abs(STA, LAST_CTRL);
    e.abs(STA, PPUCTRL);
    if banked { e.jsr("__chr_copy"); }
    e.jsr("__init_globals");
    if mmc3 {
        e.imp(CLI);
    }
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
    e.abs(LDA, STATUS_ROWS);
    e.br(BEQ, "__nmi_no_status");
    e.jsr("__status_nmi"); // sprite-0 marker: OAM slot 0 and its background cell
    e.label("__nmi_no_status");
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
    e.abs(BIT, PPUSTATUS); // reset the $2005 toggle (the split leaves it after one write)
    e.abs(LDA, STATUS_ROWS);
    e.br(BEQ, "__nmi_scroll_cam");
    // Status bar: the top of the picture is drawn unscrolled from nametable 0.
    e.abs(LDA, LAST_CTRL);
    e.imm(AND, 0xFE);
    e.abs(STA, PPUCTRL);
    e.imm(LDA, 0);
    e.abs(STA, PPUSCROLL);
    e.abs(STA, PPUSCROLL);
    e.jmp("__nmi_skip");
    e.label("__nmi_scroll_cam");
    e.abs(LDA, LAST_CTRL);
    e.abs(STA, PPUCTRL);
    e.abs(LDA, LAST_SX);
    e.abs(STA, PPUSCROLL);
    e.abs(LDA, SCROLL_Y);
    e.abs(STA, PPUSCROLL);
    e.label("__nmi_skip");
    if mmc3 {
        // Re-arm for this frame: the pre-render line reloads the counter, so the IRQ comes
        // (latch + 1) lines later; latch = line - 2 leaves irq() time to act by `line`.
        e.abs(STA, 0xE000);
        e.abs(LDA, IRQ_LINE);
        e.br(BEQ, "__nmi_irq_off");
        e.imp(SEC);
        e.imm(SBC, 2);
        e.br(BCS, "__nmi_latch");
        e.imm(LDA, 0);
        e.label("__nmi_latch");
        e.abs(STA, 0xC000);
        e.abs(STA, 0xC001);
        e.abs(STA, 0xE001);
        e.label("__nmi_irq_off");
    }
    e.jsr("__sfx_tick");
    e.jsr("__music_tick");
    e.abs(LDA, MASK_SHADOW);
    e.imm(AND, 0x18);
    e.br(BEQ, "__nmi_no_split");
    e.abs(LDA, STATUS_ROWS);
    e.br(BEQ, "__nmi_no_split");
    e.jsr("__status_split");
    e.label("__nmi_no_split");
    e.abs(INC, FRAME);
    e.imp(PLA);
    e.imp(TAY);
    e.imp(PLA);
    e.imp(TAX);
    e.imp(PLA);
    e.imp(RTI);

    e.comment("---- runtime: IRQ ----");
    e.label("__irq");
    if mmc3 {
        e.imp(PHA);
        e.imp(TXA);
        e.imp(PHA);
        e.imp(TYA);
        e.imp(PHA);
        for a in IRQ_SAVED.iter().flat_map(|r| r.clone()) {
            e.abs(LDA, a);
            e.imp(PHA);
        }
        e.abs(STA, 0xE000); // acknowledge; NMI re-arms for the next frame
        e.jsr("f_irq");
        for a in IRQ_SAVED.iter().flat_map(|r| r.clone()).rev() {
            e.imp(PLA);
            e.abs(STA, a);
        }
        e.imp(PLA);
        e.imp(TAY);
        e.imp(PLA);
        e.imp(TAX);
        e.imp(PLA);
    }
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

#[cfg(test)]
mod layout_tests {
    use super::*;

    /// Every runtime variable gets its own bytes: two names on one address silently
    /// corrupt each other (SEL_BANK once shared VQ_LOCK and froze the VRAM queue).
    #[test]
    fn runtime_variables_do_not_overlap() {
        let mut vars: Vec<(&str, u16, u16)> = vec![
            ("TMP0", TMP0, 1), ("TMP1", TMP1, 1), ("TMP2", TMP2, 1), ("TMP3", TMP3, 1),
            ("PTR", PTR, 2), ("PTR2", PTR2, 2), ("FRAME", FRAME, 1), ("CTRL_SHADOW", CTRL_SHADOW, 1),
            ("MASK_SHADOW", MASK_SHADOW, 1), ("SCROLL_X", SCROLL_X, 1), ("SCROLL_Y", SCROLL_Y, 1),
            ("RNG", RNG, 1), ("TMP4", TMP4, 1), ("TMP5", TMP5, 1), ("ARG0", ARG0, 1), ("ARG1", ARG1, 1),
            ("ARG2", ARG2, 1), ("ARG3", ARG3, 1), ("ARG4", ARG4, 1), ("VQ_LEN", VQ_LEN, 1),
            ("NMI_PTR", NMI_PTR, 2), ("VQ", VQ, VQ_SIZE as u16), ("PRINT_BUF", PRINT_BUF, 5),
            ("SMT_W", SMT_W, 1), ("SEL_ID", SEL_ID, 1), ("SEL_W", SEL_W, 1), ("SEL_BANK", SEL_BANK, 1), ("IRQ_LINE", IRQ_LINE, 1),
            ("VQ_LOCK", VQ_LOCK, 1), ("SFX_TIMER", SFX_TIMER, 1), ("SFX_CH", SFX_CH, 1),
            ("SFX_PLO", SFX_PLO, 1), ("SFX_DELTA", SFX_DELTA, 1), ("SFX_MAXVOL", SFX_MAXVOL, 1),
            ("SFX_DUTY", SFX_DUTY, 1), ("APU_EN", APU_EN, 1), ("CAM_X", CAM_X, 2), ("CAM_COL", CAM_COL, 1),
            ("MAP_T", MAP_T, 2), ("MAP_A", MAP_A, 2), ("MAP_W", MAP_W, 1), ("SC_COL", SC_COL, 1),
            ("SC_G", SC_G, 1), ("SC_R", SC_R, 1), ("LAST_CTRL", LAST_CTRL, 1), ("LAST_SX", LAST_SX, 1),
            ("CA_LO", CA_LO, 1), ("CA_HI", CA_HI, 1), ("SUB_LO", SUB_LO, 1), ("SUB_HI", SUB_HI, 1),
            ("NGROUPS", NGROUPS, 1), ("AG", AG, 1), ("MU_ON", MU_ON, 1), ("MU_NOTE", MU_NOTE, 1),
            ("MU_ARG", MU_ARG, 1), ("OV_COUNT", OV_COUNT, 1), ("STATUS_ROWS", STATUS_ROWS, 1),
            ("MAP_ID", MAP_ID, 1), ("OV_RES", OV_RES, 1), ("STATUS_TMP", STATUS_TMP, 1),
        ];
        for (i, v) in VOICES.iter().enumerate() {
            vars.push((["VOICE0", "VOICE1", "VOICE2"][i], *v, 8));
        }
        vars.sort_by_key(|v| v.1);
        for w in vars.windows(2) {
            let (a, b) = (w[0], w[1]);
            assert!(a.1 + a.2 <= b.1, "{} (${:04X}, {} bytes) overlaps {} (${:04X})", a.0, a.1, a.2, b.0, b.1);
        }
        let last = vars.last().unwrap();
        assert!(last.1 + last.2 <= USER_RAM_START, "{} runs into user RAM", last.0);
    }
}

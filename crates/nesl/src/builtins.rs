//! Built-in constants and NES API functions.

/// Controller button masks as returned by `joypad()`.
pub const CONSTANTS: &[(&str, u32)] = &[
    ("BTN_A", 0x80),
    ("BTN_B", 0x40),
    ("BTN_SELECT", 0x20),
    ("BTN_START", 0x10),
    ("BTN_UP", 0x08),
    ("BTN_DOWN", 0x04),
    ("BTN_LEFT", 0x02),
    ("BTN_RIGHT", 0x01),
    ("SFX_JUMP", 0),
    ("SFX_COIN", 1),
    ("SFX_SHOOT", 2),
    ("SFX_HIT", 3),
    ("SFX_EXPLODE", 4),
];

/// Built-in sound effects, 8 bytes each:
/// `[channel (0 = pulse 2, 1 = noise), period lo, period hi, delta (i8), frames, max volume, duty bits, pad]`.
pub(crate) const SFX_TABLE: [[u8; 8]; 5] = [
    [0, 0xC8, 0x00, 0xFA, 16, 12, 0xB0, 0], // JUMP: rising sweep
    [0, 0x50, 0x00, 0x00, 12, 10, 0x70, 0], // COIN: bright blip
    [0, 0x30, 0x00, 0x08, 8, 12, 0x30, 0],  // SHOOT: short falling zap
    [0, 0x40, 0x01, 0x06, 12, 15, 0xF0, 0], // HIT: low thud
    [1, 0x0C, 0x00, 0x00, 30, 15, 0x30, 0], // EXPLODE: noise burst
];

pub fn constant(name: &str) -> Option<u32> {
    CONSTANTS.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
}

use crate::asm::{Op::*, Operand};
use crate::ast::{BinOp, Expr, ExprKind, Pos, Type};
use crate::codegen::{contains_call, Codegen, Sym, R};
use crate::runtime::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Param {
    U8,
    U16,
    /// A `const NAME: [u8]` array.
    Data,
    /// A string literal or a constant array.
    Text,
    /// A `map` declaration.
    Map,
    /// A `metasprite` declaration.
    Meta,
    /// A `music` declaration.
    Music,
}

pub struct Signature {
    pub params: &'static [Param],
    pub ret: Option<Type>,
}

macro_rules! sig {
    ($($p:ident),* => $ret:expr) => { Signature { params: &[$(Param::$p),*], ret: $ret } };
}

pub fn signature(name: &str) -> Option<Signature> {
    const U8: Option<Type> = Some(Type::U8);
    Some(match name {
        "wait_vblank" | "ppu_on" | "ppu_off" | "stop_sound" => sig!(=> None),
        "frame_count" | "rand" => sig!(=> U8),
        "load_palette" => sig!(Data => None),
        "vram_addr" => sig!(U16 => None),
        "vram_put" | "hide_sprite" => sig!(U8 => None),
        "vram_fill" => sig!(U8, U16 => None),
        "print" => sig!(U8, U8, Text => None),
        "print_u8" | "set_tile" => sig!(U8, U8, U8 => None),
        "sprite" => sig!(U8, U8, U8, U8, U8 => None),
        "joypad" => sig!(U8 => U8),
        "scroll" => sig!(U8, U8 => None),
        "peek" => sig!(U16 => U8),
        "poke" => sig!(U16, U8 => None),
        "play_tone" => sig!(U16 => None),
        "draw_map" => sig!(Map => None),
        "map_tile" => sig!(Map, U16, U8 => U8),
        "camera_follow" => sig!(U16, Map => None),
        "set_map_tile" => sig!(Map, U8, U8, U8 => U8),
        "reset_map" => sig!(Map => None),
        "camera_x" => sig!(=> Some(Type::U16)),
        "metasprite" => sig!(U8, U8, U8, Meta => U8),
        "collide" => sig!(U8, U8, U8, U8, U8, U8, U8, U8 => U8),
        "sfx" => sig!(U8 => None),
        // play_music takes 1 to 3 songs; the arity is checked in builtin_call
        "play_music" => sig!(Music => None),
        "stop_music" => sig!(=> None),
        _ => return None,
    })
}

pub fn is_builtin_fn(name: &str) -> bool {
    signature(name).is_some()
}

impl Codegen {
    pub(crate) fn builtin_call(&mut self, name: &str, args: &[Expr], pos: Pos) -> R<Option<Type>> {
        let sig = signature(name).expect("caller checked");
        if name == "play_music" {
            self.play_music(args, pos)?;
            return Ok(None);
        }
        if args.len() != sig.params.len() {
            let s = if sig.params.len() == 1 { "" } else { "s" };
            return self.err(pos, format!("function `{name}` expects {} argument{s}, found {}", sig.params.len(), args.len()));
        }
        match name {
            "wait_vblank" => self.e.jsr("__wait_vblank"),
            "ppu_on" => self.e.jsr("__ppu_on"),
            "ppu_off" => self.e.jsr("__ppu_off"),
            "frame_count" => self.e.abs(LDA, FRAME),
            "rand" => self.e.jsr("__rand"),
            "stop_music" => self.e.jsr("__stop_music"),
            "stop_sound" => {
                self.e.imm(LDA, 0);
                self.e.abs(STA, MU_ON);
                self.e.abs(STA, SFX_TIMER);
                self.e.abs(STA, APU_EN);
                self.e.abs(STA, 0x4015);
            }
            "sfx" => {
                self.arg(&args[0], Type::U8)?;
                self.e.jsr("__sfx");
            }
            "vram_put" => {
                self.arg(&args[0], Type::U8)?;
                self.e.abs(STA, PPUDATA);
            }
            "vram_addr" => {
                self.arg(&args[0], Type::U16)?;
                self.e.jsr("__vram_addr");
            }
            "play_tone" => {
                self.arg(&args[0], Type::U16)?;
                self.e.jsr("__play_tone");
            }
            "joypad" => {
                self.arg(&args[0], Type::U8)?;
                self.e.imm(AND, 1);
                self.e.imp(TAX);
                self.e.jsr("__joypad");
            }
            "hide_sprite" => {
                self.arg(&args[0], Type::U8)?;
                self.e.acc(ASL);
                self.e.acc(ASL);
                self.e.imp(TAX);
                self.e.imm(LDA, 0xFF);
                self.e.ins(STA, Operand::abs_x(OAM_SHADOW));
            }
            "scroll" => {
                self.slots(&[(&args[0], Type::U8, SCROLL_X), (&args[1], Type::U8, SCROLL_Y)])?;
            }
            "peek" => match self.const_val(&args[0]) {
                Some(a) => self.e.abs(LDA, crate::codegen::fit(a, Type::U16, args[0].pos)?),
                None => {
                    self.arg(&args[0], Type::U16)?;
                    self.e.abs(STA, PTR);
                    self.e.abs(STX, PTR + 1);
                    self.e.imm(LDY, 0);
                    self.e.ins(LDA, Operand::IndY(PTR as u8));
                }
            },
            "poke" => match self.const_val(&args[0]) {
                Some(a) => {
                    let a = crate::codegen::fit(a, Type::U16, args[0].pos)?;
                    self.arg(&args[1], Type::U8)?;
                    self.e.abs(STA, a);
                }
                None => {
                    self.arg(&args[1], Type::U8)?;
                    self.e.imp(PHA);
                    self.arg(&args[0], Type::U16)?;
                    self.e.abs(STA, PTR);
                    self.e.abs(STX, PTR + 1);
                    self.e.imp(PLA);
                    self.e.imm(LDY, 0);
                    self.e.ins(STA, Operand::IndY(PTR as u8));
                }
            },
            "load_palette" => {
                let (label, len) = match &args[0].kind {
                    ExprKind::Var(n) => match self.lookup(n) {
                        Some(Sym::Data { label, len, elem: Type::U8 }) => (label, len),
                        _ => return self.err(args[0].pos, "`load_palette` expects a constant array (`const PAL: [u8] = [...]`)"),
                    },
                    _ => return self.err(args[0].pos, "`load_palette` expects a constant array (`const PAL: [u8] = [...]`)"),
                };
                if len > 32 {
                    return self.err(args[0].pos, format!("a palette has at most 32 entries, `{}` has {}", label, len));
                }
                self.pointer(&label);
                self.e.imm(LDA, len as u8);
                self.e.abs(STA, ARG2);
                self.e.jsr("__load_palette");
            }
            "print" => {
                self.slots(&[(&args[0], Type::U8, ARG0), (&args[1], Type::U8, ARG1)])?;
                let (label, len) = self.text(&args[2])?;
                self.pointer(&label);
                self.e.imm(LDA, len);
                self.e.abs(STA, ARG2);
                self.e.jsr("__print");
            }
            "print_u8" | "set_tile" => {
                self.slots(&[(&args[0], Type::U8, ARG0), (&args[1], Type::U8, ARG1), (&args[2], Type::U8, ARG3)])?;
                self.e.jsr(if name == "print_u8" { "__print_u8" } else { "__set_tile" });
            }
            "vram_fill" => {
                self.slots(&[(&args[0], Type::U8, ARG0), (&args[1], Type::U16, ARG1)])?;
                self.e.jsr("__vram_fill");
            }
            "sprite" => {
                let slots = [ARG0, ARG1, ARG2, ARG3, ARG4];
                let list: Vec<(&Expr, Type, u16)> = args.iter().zip(slots).map(|(a, s)| (a, Type::U8, s)).collect();
                self.slots(&list)?;
                self.e.jsr("__sprite");
            }
            "camera_x" => {
                self.e.abs(LDA, CAM_X);
                self.e.abs(LDX, CAM_X + 1);
            }
            "camera_follow" => {
                let (label, width, id) = self.map_label(&args[1])?;
                self.slots(&[(&args[0], Type::U16, ARG0)])?;
                self.map_pointers(&label, width, id);
                self.e.jsr("__camera_follow");
            }
            "draw_map" => {
                let (label, width, id) = self.map_label(&args[0])?;
                self.map_pointers(&label, width, id);
                self.e.jsr("__draw_map");
            }
            "map_tile" => {
                let (label, width, id) = self.map_label(&args[0])?;
                self.slots(&[(&args[1], Type::U16, ARG0), (&args[2], Type::U8, ARG2)])?;
                self.e.imm(LDA, width);
                self.e.abs(STA, ARG3);
                self.e.imm(LDA, id);
                self.e.abs(STA, ARG4);
                self.pointer(&label);
                self.e.jsr("__map_tile");
            }
            "set_map_tile" => {
                let (_, width, id) = self.map_label(&args[0])?;
                self.slots(&[(&args[1], Type::U8, ARG0), (&args[2], Type::U8, ARG2), (&args[3], Type::U8, ARG3)])?;
                self.e.imm(LDA, width);
                self.e.abs(STA, ARG1);
                self.e.imm(LDA, id);
                self.e.abs(STA, ARG4);
                self.e.jsr("__set_map_tile");
            }
            "reset_map" => {
                let (_, _, id) = self.map_label(&args[0])?;
                self.e.imm(LDA, id);
                self.e.abs(STA, ARG4);
                self.e.jsr("__reset_map");
            }
            "metasprite" => {
                let label = match &args[3].kind {
                    ExprKind::Var(n) => match self.lookup(n) {
                        Some(Sym::Metasprite { label }) => label,
                        _ => return self.err(args[3].pos, "expected a metasprite (declare one with `metasprite NAME = [...]`)"),
                    },
                    _ => return self.err(args[3].pos, "expected a metasprite (declare one with `metasprite NAME = [...]`)"),
                };
                self.slots(&[(&args[0], Type::U8, ARG0), (&args[1], Type::U8, ARG1), (&args[2], Type::U8, ARG2)])?;
                self.pointer(&label);
                self.e.jsr("__metasprite");
            }
            "collide" => self.collide(args, pos)?,
            _ => unreachable!("missing builtin {name}"),
        }
        Ok(sig.ret)
    }

    fn arg(&mut self, e: &Expr, t: Type) -> R<()> {
        self.check_fits(e, t)?;
        self.expr(e, t)
    }

    /// Evaluates arguments into fixed memory slots.
    fn slots(&mut self, args: &[(&Expr, Type, u16)]) -> R<()> {
        for (e, t, _) in args {
            self.check_fits(e, *t)?;
        }
        if args.iter().skip(1).any(|(e, ..)| contains_call(e)) {
            for (e, t, _) in args {
                self.expr(e, *t)?;
                self.e.imp(PHA);
                if *t == Type::U16 {
                    self.e.imp(TXA);
                    self.e.imp(PHA);
                }
            }
            for (_, t, slot) in args.iter().rev() {
                if *t == Type::U16 {
                    self.e.imp(PLA);
                    self.e.abs(STA, slot + 1);
                }
                self.e.imp(PLA);
                self.e.abs(STA, *slot);
            }
        } else {
            for (e, t, slot) in args {
                self.expr(e, *t)?;
                self.store(*slot, *t);
            }
        }
        Ok(())
    }

    /// Rectangles overlap iff each starts before the other ends, compared in 16 bits so
    /// `x + w` past 255 does not wrap. Arguments are evaluated once into hidden slots.
    fn collide(&mut self, args: &[Expr], pos: Pos) -> R<()> {
        const NAMES: [&str; 8] = ["$ax", "$ay", "$aw", "$ah", "$bx", "$by", "$bw", "$bh"];
        for a in args {
            self.check_fits(a, Type::U8)?;
        }
        let addrs = self.hidden_u8_scope(&NAMES, pos)?;
        let list: Vec<(&Expr, Type, u16)> = args.iter().zip(&addrs).map(|(a, s)| (a, Type::U8, *s)).collect();
        let r = self.slots(&list).and_then(|_| {
            let v = |n: &str| Expr { kind: ExprKind::Var(n.to_string()), pos };
            let wide = |n: &str| Expr { kind: ExprKind::Cast(Box::new(v(n)), Type::U16), pos };
            let bin = |op, l: Expr, r: Expr| Expr { kind: ExprKind::Binary(op, Box::new(l), Box::new(r)), pos };
            // a starts before b ends, and b starts before a ends, on both axes
            let axis = |p: &str, size_p: &str, q: &str, size_q: &str| {
                bin(
                    BinOp::LogAnd,
                    bin(BinOp::Lt, wide(p), bin(BinOp::Add, wide(q), v(size_q))),
                    bin(BinOp::Lt, wide(q), bin(BinOp::Add, wide(p), v(size_p))),
                )
            };
            let all = bin(BinOp::LogAnd, axis("$ax", "$aw", "$bx", "$bw"), axis("$ay", "$ah", "$by", "$bh"));
            self.expr(&all, Type::U8)
        });
        self.pop_scope();
        r
    }

    /// PTR = tiles, PTR2 = attributes, ARG2 = width, ARG3 = map id.
    fn map_pointers(&mut self, label: &str, width: u8, id: u8) {
        self.pointer(label);
        let attrs = format!("{label}_a");
        self.e.ins(LDA, Operand::imm_lo(&attrs));
        self.e.abs(STA, PTR2);
        self.e.ins(LDA, Operand::imm_hi(&attrs));
        self.e.abs(STA, PTR2 + 1);
        self.e.imm(LDA, width);
        self.e.abs(STA, ARG2);
        self.e.imm(LDA, id);
        self.e.abs(STA, ARG3);
    }

    /// `play_music(melody[, bass[, harmony]])`: voices are set up while the engine is
    /// stopped, and `MU_ON` is written last so the NMI never sees a half-initialised voice.
    fn play_music(&mut self, args: &[Expr], pos: Pos) -> R<()> {
        if args.is_empty() || args.len() > 3 {
            return self.err(pos, format!("`play_music` takes 1 to 3 songs (melody, bass, harmony), found {}", args.len()));
        }
        let mut songs = Vec::new();
        for (v, a) in args.iter().enumerate() {
            let Some(Sym::Music { label, lowest }) = (match &a.kind {
                ExprKind::Var(n) => self.lookup(n),
                _ => None,
            }) else {
                return self.err(a.pos, "`play_music` expects music (declare it with `music NAME = \"...\"`)");
            };
            // Pulse channels (melody, harmony) reach down to A1; the triangle (bass) to A0.
            let (floor, what) = if v == 1 { (9, "the bass voice (triangle) cannot play below A0") } else {
                (21, "pulse voices cannot play below A1; pass this music as the bass (second) voice")
            };
            if let Some((n, npos)) = lowest {
                if n < floor {
                    return self.err(npos, format!("note too low: {what}"));
                }
            }
            songs.push(label);
        }
        self.e.jsr("__stop_music");
        for (v, label) in songs.iter().enumerate() {
            let b = VOICES[v];
            for (off, imm) in [(0u16, Operand::imm_lo(label)), (1, Operand::imm_hi(label))] {
                self.e.ins(LDA, imm);
                self.e.abs(STA, b + off);
                self.e.abs(STA, b + 2 + off);
            }
            for (off, v) in [(4u16, 1u8), (5, 12), (6, 0x80)] {
                self.e.imm(LDA, v);
                self.e.abs(STA, b + off);
            }
        }
        self.e.imm(LDA, (1u8 << songs.len()) - 1);
        self.e.abs(STA, MU_ON);
        Ok(())
    }

    fn map_label(&self, e: &Expr) -> R<(String, u8, u8)> {
        if let ExprKind::Var(n) = &e.kind {
            if let Some(Sym::Map { label, width, id }) = self.lookup(n) {
                return Ok((label, width, id));
            }
        }
        self.err(e.pos, "expected a map (declare one with `map NAME = [...]`)")
    }

    fn pointer(&mut self, label: &str) {
        self.e.ins(LDA, Operand::imm_lo(label));
        self.e.abs(STA, PTR);
        self.e.ins(LDA, Operand::imm_hi(label));
        self.e.abs(STA, PTR + 1);
    }

    /// Resolves a `print` text argument to a data label and length.
    fn text(&mut self, e: &Expr) -> R<(String, u8)> {
        let (label, len) = match &e.kind {
            ExprKind::Str(bytes) => {
                if bytes.is_empty() {
                    return self.err(e.pos, "cannot print an empty string");
                }
                let label = format!("s_{}", self.e.new_label());
                self.data.push((label.clone(), bytes.clone()));
                (label, bytes.len())
            }
            ExprKind::Var(n) => match self.lookup(n) {
                Some(Sym::Data { label, len, elem: Type::U8 }) => (label, len as usize),
                _ => return self.err(e.pos, "`print` expects a string literal or a constant array"),
            },
            _ => return self.err(e.pos, "`print` expects a string literal or a constant array"),
        };
        if len > 255 {
            return self.err(e.pos, "text is longer than 255 bytes");
        }
        Ok((label, len as u8))
    }
}

/// Runtime routines backing the built-in functions.
pub fn emit_runtime(e: &mut Emitter) {
    e.comment("---- runtime: PPU ----");
    e.label("__wait_vblank");
    e.abs(LDA, FRAME);
    e.label("__wait_vblank_loop");
    e.abs(CMP, FRAME);
    e.br(BEQ, "__wait_vblank_loop");
    e.imp(RTS);

    e.label("__ppu_on");
    e.jsr("__wait_vblank");
    e.jsr("__fix_scroll");
    e.imm(LDA, 0x1E); // show background + sprites, including the leftmost 8 pixels
    e.abs(STA, MASK_SHADOW);
    e.abs(STA, PPUMASK);
    e.imp(RTS);

    e.label("__ppu_off");
    e.imm(LDA, 0);
    e.abs(STA, MASK_SHADOW);
    e.abs(STA, PPUMASK);
    e.jmp("__vq_flush"); // rendering is off now, so pending writes can go straight in

    // Rewrites PPUCTRL/PPUSCROLL from the shadows. After $2006 writes this only changes the
    // PPU's temporary address, so pending $2007 writes still land where intended.
    e.label("__fix_scroll");
    e.abs(BIT, PPUSTATUS);
    e.abs(LDA, CTRL_SHADOW);
    e.abs(STA, PPUCTRL);
    e.abs(LDA, SCROLL_X);
    e.abs(STA, PPUSCROLL);
    e.abs(LDA, SCROLL_Y);
    e.abs(STA, PPUSCROLL);
    e.imp(RTS);

    // A = low byte, X = high byte
    e.label("__vram_addr");
    e.abs(BIT, PPUSTATUS);
    e.abs(STX, PPUADDR);
    e.abs(STA, PPUADDR);
    e.jmp("__fix_scroll");

    // PTR = source, ARG2 = length
    e.label("__load_palette");
    e.imm(LDA, 0x00);
    e.abs(STA, TMP0);
    e.imm(LDA, 0x3F);
    e.abs(STA, TMP1);
    e.jmp("__vram_write");

    // ARG0 = x, ARG1 = y, PTR = text, ARG2 = length
    e.label("__print");
    e.abs(LDA, ARG1);
    for _ in 0..3 {
        e.acc(LSR);
    }
    e.imm(ORA, 0x20);
    e.abs(STA, TMP1);
    e.abs(LDA, ARG1);
    for _ in 0..5 {
        e.acc(ASL);
    }
    e.imp(CLC);
    e.abs(ADC, ARG0);
    e.abs(STA, TMP0);
    e.br(BCC, "__vram_write");
    e.abs(INC, TMP1);
    // fall through

    // TMP1:TMP0 = VRAM address, PTR = source, ARG2 = length (1-255).
    // Writes directly when rendering is off, otherwise queues 32-byte chunks for the NMI.
    // `__vram_write_f` additionally takes TMP3 = $80 for a vertical (+32) write of <= 32 bytes.
    e.label("__vram_write");
    e.imm(LDA, 0);
    e.abs(STA, TMP3);
    e.label("__vram_write_f");
    e.abs(LDA, MASK_SHADOW);
    e.imm(AND, 0x18);
    e.br(BNE, "__vw_queue");
    e.abs(LDA, CTRL_SHADOW);
    e.abs(BIT, TMP3);
    e.br(BPL, "__vw_horiz");
    e.imm(ORA, 0x04);
    e.label("__vw_horiz");
    e.abs(STA, PPUCTRL);
    e.abs(BIT, PPUSTATUS);
    e.abs(LDA, TMP1);
    e.abs(STA, PPUADDR);
    e.abs(LDA, TMP0);
    e.abs(STA, PPUADDR);
    e.imm(LDY, 0);
    e.label("__vw_loop");
    e.ins(LDA, Operand::IndY(PTR as u8));
    e.abs(STA, PPUDATA);
    e.imp(INY);
    e.abs(CPY, ARG2);
    e.br(BNE, "__vw_loop");
    e.jmp("__fix_scroll");
    e.label("__vw_queue");
    e.abs(LDA, ARG2);
    e.imm(CMP, 33);
    e.br(BCC, "__vw_last");
    e.imm(LDA, 32);
    e.abs(STA, TMP2);
    e.jsr("__vq_push");
    e.abs(LDA, ARG2);
    e.imp(SEC);
    e.imm(SBC, 32);
    e.abs(STA, ARG2);
    for (lo, hi) in [(PTR, PTR + 1), (TMP0, TMP1)] {
        let skip = e.new_label();
        e.abs(LDA, lo);
        e.imp(CLC);
        e.imm(ADC, 32);
        e.abs(STA, lo);
        e.br(BCC, &skip);
        e.abs(INC, hi);
        e.label(&skip);
    }
    e.jmp("__vw_queue");
    e.label("__vw_last");
    e.abs(STA, TMP2);
    // fall through

    // Appends [TMP1, TMP0, TMP2, PTR[0..TMP2]] to the queue, waiting for a flush when full.
    e.label("__vq_push");
    e.abs(LDA, VQ_LEN);
    e.imp(SEC); // +1 for the header, +2 more below
    e.abs(ADC, TMP2);
    e.imm(ADC, 2);
    e.imm(CMP, VQ_SIZE + 1);
    e.br(BCC, "__vq_fits");
    e.jsr("__wait_vblank");
    e.jmp("__vq_push");
    e.label("__vq_fits");
    e.abs(INC, VQ_LOCK);
    e.abs(LDX, VQ_LEN);
    for src in [TMP1, TMP0] {
        e.abs(LDA, src);
        e.ins(STA, Operand::abs_x(VQ));
        e.imp(INX);
    }
    e.abs(LDA, TMP2);
    e.abs(ORA, TMP3); // bit 7 of the length byte marks a vertical entry
    e.ins(STA, Operand::abs_x(VQ));
    e.imp(INX);
    e.imm(LDY, 0);
    e.label("__vq_copy");
    e.ins(LDA, Operand::IndY(PTR as u8));
    e.ins(STA, Operand::abs_x(VQ));
    e.imp(INX);
    e.imp(INY);
    e.abs(CPY, TMP2);
    e.br(BNE, "__vq_copy");
    e.abs(STX, VQ_LEN); // commit: NMI only ever sees whole entries
    e.abs(DEC, VQ_LOCK);
    e.imp(RTS);

    // Writes every queued entry to the PPU and empties the queue. Uses A, X, Y only.
    e.label("__vq_flush");
    e.imm(LDX, 0);
    e.label("__vqf_entry");
    e.abs(CPX, VQ_LEN);
    e.br(BCS, "__vqf_done");
    e.abs(BIT, PPUSTATUS);
    for _ in 0..2 {
        e.ins(LDA, Operand::abs_x(VQ));
        e.abs(STA, PPUADDR);
        e.imp(INX);
    }
    e.ins(LDA, Operand::abs_x(VQ));
    e.imp(INX);
    e.imp(TAY);
    e.abs(LDA, CTRL_SHADOW);
    e.imm(CPY, 0x80);
    e.br(BCC, "__vqf_horiz");
    e.imm(ORA, 0x04);
    e.label("__vqf_horiz");
    e.abs(STA, PPUCTRL);
    e.imp(TYA);
    e.imm(AND, 0x7F);
    e.imp(TAY);
    e.label("__vqf_byte");
    e.ins(LDA, Operand::abs_x(VQ));
    e.abs(STA, PPUDATA);
    e.imp(INX);
    e.imp(DEY);
    e.br(BNE, "__vqf_byte");
    e.jmp("__vqf_entry");
    e.label("__vqf_done");
    e.imm(LDA, 0);
    e.abs(STA, VQ_LEN);
    e.abs(LDA, CTRL_SHADOW);
    e.abs(STA, PPUCTRL); // back to +1 increments
    e.imp(RTS);

    // ARG0 = x, ARG1 = y, ARG3 = tile
    e.label("__set_tile");
    e.abs(LDA, ARG3);
    e.abs(STA, PRINT_BUF);
    e.imm(LDA, 1);
    e.jmp("__print_buf");

    // ARG0 = x, ARG1 = y, ARG3 = value
    e.label("__print_u8");
    e.abs(LDA, ARG3);
    for (i, (unit, label)) in [(100u8, "__pu_h"), (10, "__pu_t")].into_iter().enumerate() {
        e.imm(LDX, b'0' - 1);
        e.imp(SEC);
        e.label(label);
        e.imp(INX);
        e.imm(SBC, unit);
        e.br(BCS, label);
        e.imm(ADC, unit); // carry is clear here
        e.abs(STX, PRINT_BUF + i as u16);
    }
    e.imm(ORA, b'0');
    e.abs(STA, PRINT_BUF + 2);
    e.imm(LDA, 3);
    e.label("__print_buf"); // A = length of PRINT_BUF to print at (ARG0, ARG1)
    e.abs(STA, ARG2);
    e.imm(LDA, PRINT_BUF as u8);
    e.abs(STA, PTR);
    e.imm(LDA, (PRINT_BUF >> 8) as u8);
    e.abs(STA, PTR + 1);
    e.jmp("__print");

    // ARG0 = value, ARG1:ARG2 = count
    e.label("__vram_fill");
    e.label("__vram_fill_loop");
    e.abs(LDA, ARG1);
    e.abs(ORA, ARG2);
    e.br(BEQ, "__vram_fill_done");
    e.abs(LDA, ARG0);
    e.abs(STA, PPUDATA);
    e.abs(LDA, ARG1);
    e.br(BNE, "__vram_fill_lo");
    e.abs(DEC, ARG2);
    e.label("__vram_fill_lo");
    e.abs(DEC, ARG1);
    e.jmp("__vram_fill_loop");
    e.label("__vram_fill_done");
    e.imp(RTS);

    e.comment("---- runtime: maps ----");
    // PTR += A * 30 (maps are column-major, 30 tiles per column).
    e.label("__col_addr");
    e.abs(STA, CA_LO);
    e.imm(LDA, 0);
    e.abs(STA, CA_HI);
    e.abs(ASL, CA_LO);
    e.abs(ROL, CA_HI);
    e.abs(LDA, CA_LO);
    e.abs(STA, SUB_LO);
    e.abs(LDA, CA_HI);
    e.abs(STA, SUB_HI); // col * 2
    for _ in 0..4 {
        e.abs(ASL, CA_LO);
        e.abs(ROL, CA_HI);
    } // col * 32
    e.abs(LDA, CA_LO);
    e.imp(SEC);
    e.abs(SBC, SUB_LO);
    e.abs(STA, CA_LO);
    e.abs(LDA, CA_HI);
    e.abs(SBC, SUB_HI);
    e.abs(STA, CA_HI);
    e.abs(LDA, PTR);
    e.imp(CLC);
    e.abs(ADC, CA_LO);
    e.abs(STA, PTR);
    e.abs(LDA, PTR + 1);
    e.abs(ADC, CA_HI);
    e.abs(STA, PTR + 1);
    e.imp(RTS);

    // A = world column. Writes it (30 tiles, vertically) into ring slot column % 64 of the
    // two side-by-side nametables, directly or via the queue. Columns past the map are skipped.
    e.label("__stream_col");
    e.abs(CMP, MAP_W);
    e.br(BCS, "__sc_skip");
    e.abs(STA, SC_COL);
    e.abs(LDX, MAP_T);
    e.abs(STX, PTR);
    e.abs(LDX, MAP_T + 1);
    e.abs(STX, PTR + 1);
    e.jsr("__col_addr");
    // With map changes recorded, copy the column to RAM and apply this map's changes.
    e.abs(LDA, OV_COUNT);
    e.br(BEQ, "__sc_addr");
    e.imm(LDY, 29);
    e.label("__sc_copy");
    e.ins(LDA, Operand::IndY(PTR as u8));
    e.ins(STA, Operand::abs_y(COL_BUF));
    e.imp(DEY);
    e.br(BPL, "__sc_copy");
    e.imm(LDX, 0);
    e.label("__sc_ov");
    e.abs(CPX, OV_COUNT);
    e.br(BCS, "__sc_ov_done");
    e.ins(LDA, Operand::abs_x(OV_ID));
    e.abs(CMP, MAP_ID);
    e.br(BNE, "__sc_ov_next");
    e.ins(LDA, Operand::abs_x(OV_COL));
    e.abs(CMP, SC_COL);
    e.br(BNE, "__sc_ov_next");
    e.ins(LDY, Operand::abs_x(OV_ROW));
    e.ins(LDA, Operand::abs_x(OV_TILE));
    e.ins(STA, Operand::abs_y(COL_BUF));
    e.label("__sc_ov_next");
    e.imp(INX);
    e.jmp("__sc_ov");
    e.label("__sc_ov_done");
    e.imm(LDA, COL_BUF as u8);
    e.abs(STA, PTR);
    e.imm(LDA, (COL_BUF >> 8) as u8);
    e.abs(STA, PTR + 1);
    e.label("__sc_addr");
    e.abs(LDA, SC_COL);
    e.imm(AND, 0x1F);
    e.abs(STA, TMP0);
    e.abs(LDA, SC_COL);
    e.imm(AND, 0x20); // ring slots 32-63 live in the second nametable ($2400)
    e.acc(LSR);
    e.acc(LSR);
    e.acc(LSR);
    e.imm(ORA, 0x20);
    e.abs(STA, TMP1);
    // Skip the status bar rows: source += n, VRAM address += n * 32, length = 30 - n.
    e.abs(LDA, STATUS_ROWS);
    e.br(BEQ, "__sc_len");
    e.imp(CLC);
    e.abs(ADC, PTR);
    e.abs(STA, PTR);
    e.br(BCC, "__sc_src_nc");
    e.abs(INC, PTR + 1);
    e.label("__sc_src_nc");
    e.abs(LDA, STATUS_ROWS);
    e.acc(LSR);
    e.acc(LSR);
    e.acc(LSR); // n >> 3: 1 only for n = 8
    e.imp(CLC);
    e.abs(ADC, TMP1);
    e.abs(STA, TMP1);
    e.abs(LDA, STATUS_ROWS);
    for _ in 0..5 {
        e.acc(ASL);
    }
    e.imp(CLC);
    e.abs(ADC, TMP0);
    e.abs(STA, TMP0); // slot & 31 + (n * 32 & 255) never carries
    e.label("__sc_len");
    e.imm(LDA, 30);
    e.imp(SEC);
    e.abs(SBC, STATUS_ROWS);
    e.abs(STA, ARG2);
    e.imm(LDA, 0x80);
    e.abs(STA, TMP3);
    e.jmp("__vram_write_f");
    e.label("__sc_skip");
    e.imp(RTS);

    // A = attribute group (4 columns). Writes its 8 attribute bytes into ring group g % 16.
    e.label("__attr_group");
    e.abs(STA, AG);
    e.abs(LDA, STATUS_ROWS); // first attribute row fully below the status bar: ceil(n / 4)
    e.imp(CLC);
    e.imm(ADC, 3);
    e.acc(LSR);
    e.acc(LSR);
    e.abs(STA, SC_R);
    e.label("__ag_loop");
    e.abs(LDA, AG);
    e.abs(STA, CA_LO);
    e.imm(LDA, 0);
    e.abs(STA, CA_HI);
    for _ in 0..3 {
        e.abs(ASL, CA_LO);
        e.abs(ROL, CA_HI);
    }
    e.abs(LDA, CA_LO);
    e.imp(CLC);
    e.abs(ADC, SC_R);
    e.imp(CLC);
    e.abs(ADC, MAP_A);
    e.abs(STA, PTR);
    e.abs(LDA, CA_HI);
    e.abs(ADC, MAP_A + 1);
    e.abs(STA, PTR + 1);
    e.abs(LDA, SC_R);
    e.acc(ASL);
    e.acc(ASL);
    e.acc(ASL);
    e.abs(STA, TMP0);
    e.abs(LDA, AG);
    e.imm(AND, 0x07);
    e.abs(ORA, TMP0);
    e.imm(ORA, 0xC0);
    e.abs(STA, TMP0);
    e.abs(LDA, AG);
    e.imm(AND, 0x08);
    e.acc(LSR);
    e.imm(ORA, 0x23);
    e.abs(STA, TMP1);
    e.imm(LDA, 1);
    e.abs(STA, ARG2);
    e.jsr("__vram_write");
    e.abs(INC, SC_R);
    e.abs(LDA, SC_R);
    e.imm(CMP, 8);
    e.br(BNE, "__ag_loop");
    e.imp(RTS);

    // PTR = tiles, PTR2 = attributes, ARG2 = width. Draws the first two screens with
    // rendering off (restoring it afterwards) and resets the camera.
    e.label("__draw_map");
    e.jsr("__set_map");
    e.abs(LDA, MASK_SHADOW);
    e.imp(PHA);
    e.br(BEQ, "__dm_off");
    e.jsr("__wait_vblank");
    e.jsr("__ppu_off");
    e.label("__dm_off");
    e.imm(LDA, 0);
    e.abs(STA, SC_G);
    e.label("__dm_col");
    e.abs(LDA, SC_G);
    e.imm(CMP, 64);
    e.br(BCS, "__dm_cols_done");
    e.jsr("__stream_col");
    e.abs(INC, SC_G);
    e.jmp("__dm_col");
    e.label("__dm_cols_done");
    e.abs(LDA, MAP_W); // groups = ceil(width / 4), at most 16 on screen
    e.imp(CLC);
    e.imm(ADC, 3);
    e.acc(ROR);
    e.acc(LSR);
    e.abs(STA, NGROUPS);
    e.imm(LDA, 0);
    e.abs(STA, SC_G);
    e.label("__dm_attr");
    e.abs(LDA, SC_G);
    e.abs(CMP, NGROUPS);
    e.br(BCS, "__dm_attr_done");
    e.imm(CMP, 16);
    e.br(BCS, "__dm_attr_done");
    e.jsr("__attr_group");
    e.abs(INC, SC_G);
    e.jmp("__dm_attr");
    e.label("__dm_attr_done");
    e.imm(LDA, 0);
    e.abs(STA, CAM_X);
    e.abs(STA, CAM_X + 1);
    e.abs(STA, CAM_COL);
    e.abs(STA, SCROLL_X);
    e.abs(LDA, CTRL_SHADOW);
    e.imm(AND, 0xFC);
    e.abs(STA, CTRL_SHADOW);
    e.jsr("__fix_scroll");
    e.imp(PLA);
    e.br(BEQ, "__dm_done");
    e.jmp("__ppu_on");
    e.label("__dm_done");
    e.imp(RTS);

    // ARG0:ARG1 = followed x, PTR / PTR2 / ARG2 = map. Moves the camera at most 8 px toward
    // x - 120 (clamped to the map), streams in the newly visible column, then updates scroll.
    e.label("__camera_follow");
    e.abs(INC, VQ_LOCK); // the NMI keeps the previous scroll until we are done
    e.jsr("__set_map");
    // target = x - 120, clamped to [0, max]; max = width * 8 - 256 (TMP4:TMP5)
    e.abs(LDA, ARG2);
    e.imm(CMP, 33);
    e.br(BCS, "__cf_wide");
    e.jmp("__cf_done"); // a single screen never scrolls
    e.label("__cf_wide");
    e.imm(LDA, 0);
    e.abs(STA, TMP5);
    e.abs(LDA, ARG2);
    for _ in 0..3 {
        e.acc(ASL);
        e.abs(ROL, TMP5);
    }
    e.abs(STA, TMP4);
    e.abs(DEC, TMP5); // - 256
    e.abs(LDA, ARG0);
    e.imp(SEC);
    e.imm(SBC, 120);
    e.abs(STA, TMP0);
    e.abs(LDA, ARG1);
    e.imm(SBC, 0);
    e.abs(STA, TMP1);
    e.br(BCS, "__cf_not_neg");
    e.imm(LDA, 0);
    e.abs(STA, TMP0);
    e.abs(STA, TMP1);
    e.label("__cf_not_neg");
    e.abs(LDA, TMP4); // if max < target: target = max
    e.abs(CMP, TMP0);
    e.abs(LDA, TMP5);
    e.abs(SBC, TMP1);
    e.br(BCS, "__cf_clamped");
    e.abs(LDA, TMP4);
    e.abs(STA, TMP0);
    e.abs(LDA, TMP5);
    e.abs(STA, TMP1);
    e.label("__cf_clamped");
    // diff = target - cam; step limited to +-8
    e.abs(LDA, TMP0);
    e.imp(SEC);
    e.abs(SBC, CAM_X);
    e.abs(STA, TMP2);
    e.abs(LDA, TMP1);
    e.abs(SBC, CAM_X + 1);
    e.br(BMI, "__cf_left");
    e.br(BNE, "__cf_right_max"); // diff >= 256
    e.abs(LDA, TMP2);
    e.imm(CMP, 9);
    e.br(BCC, "__cf_apply");
    e.label("__cf_right_max");
    e.imm(LDA, 8);
    e.abs(STA, TMP2);
    e.imm(LDA, 0);
    e.jmp("__cf_apply_hi");
    e.label("__cf_left");
    e.imm(CMP, 0xFF);
    e.br(BNE, "__cf_left_max"); // diff <= -256
    e.abs(LDA, TMP2);
    e.imm(CMP, 0xF8);
    e.br(BCS, "__cf_apply_neg");
    e.label("__cf_left_max");
    e.imm(LDA, 0xF8);
    e.abs(STA, TMP2);
    e.label("__cf_apply_neg");
    e.imm(LDA, 0xFF);
    e.jmp("__cf_apply_hi");
    e.label("__cf_apply");
    e.imm(LDA, 0);
    e.label("__cf_apply_hi");
    e.abs(STA, TMP3);
    e.abs(LDA, CAM_X);
    e.imp(CLC);
    e.abs(ADC, TMP2);
    e.abs(STA, CAM_X);
    e.abs(LDA, CAM_X + 1);
    e.abs(ADC, TMP3);
    e.abs(STA, CAM_X + 1);
    // new camera column = cam >> 3
    e.abs(LDA, CAM_X + 1);
    e.abs(STA, TMP1);
    e.abs(LDA, CAM_X);
    for _ in 0..3 {
        e.abs(LSR, TMP1);
        e.acc(ROR);
    }
    e.abs(CMP, CAM_COL);
    e.br(BEQ, "__cf_scroll");
    e.abs(LDX, CAM_COL);
    e.abs(STA, CAM_COL);
    e.imp(TXA);
    e.abs(CMP, CAM_COL);
    e.br(BCS, "__cf_went_left");
    // moved right: column cam_col + 32 just came into view
    e.abs(LDA, CAM_COL);
    e.imp(CLC);
    e.imm(ADC, 32);
    e.jmp("__cf_stream");
    e.label("__cf_went_left");
    e.abs(LDA, CAM_COL);
    e.label("__cf_stream");
    e.imp(PHA);
    e.jsr("__stream_col");
    e.imp(PLA);
    e.abs(STA, SC_G);
    e.imm(AND, 0x03);
    e.br(BNE, "__cf_scroll"); // attributes change only at 4-column group boundaries
    e.abs(LDA, SC_G);
    e.acc(LSR);
    e.acc(LSR);
    e.jsr("__attr_group");
    e.label("__cf_scroll");
    // scroll x = cam & 255, nametable select bit = (cam >> 8) & 1
    e.abs(LDA, CAM_X);
    e.abs(STA, SCROLL_X);
    e.abs(LDA, CAM_X + 1);
    e.imm(AND, 0x01);
    e.abs(STA, TMP0);
    e.abs(LDA, CTRL_SHADOW);
    e.imm(AND, 0xFE);
    e.abs(ORA, TMP0);
    e.abs(STA, CTRL_SHADOW);
    e.abs(LDA, MASK_SHADOW);
    e.br(BNE, "__cf_done");
    e.jsr("__fix_scroll"); // rendering off: apply now (no NMI does it for us)
    e.label("__cf_done");
    e.abs(DEC, VQ_LOCK);
    e.imp(RTS);

    // Remembers the map passed in PTR / PTR2 / ARG2 as the one being drawn or followed.
    e.label("__set_map");
    for (src, dst) in [(PTR, MAP_T), (PTR + 1, MAP_T + 1), (PTR2, MAP_A), (PTR2 + 1, MAP_A + 1), (ARG2, MAP_W), (ARG3, MAP_ID)] {
        e.abs(LDA, src);
        e.abs(STA, dst);
    }
    e.imp(RTS);

    // PTR = tiles, ARG0:ARG1 = tx, ARG2 = ty, ARG3 = width; returns the tile or 0 outside.
    e.label("__map_tile");
    e.abs(LDA, ARG1);
    e.br(BNE, "__mt_out");
    e.abs(LDA, ARG0);
    e.abs(CMP, ARG3);
    e.br(BCS, "__mt_out");
    e.abs(LDA, ARG2);
    e.imm(CMP, 30);
    e.br(BCS, "__mt_out");
    e.abs(LDA, OV_COUNT);
    e.br(BEQ, "__mt_rom");
    e.jsr("__ov_find");
    e.br(BCS, "__mt_rom");
    e.ins(LDA, Operand::abs_x(OV_TILE));
    e.imp(RTS);
    e.label("__mt_rom");
    e.abs(LDA, ARG0);
    e.jsr("__col_addr");
    e.abs(LDY, ARG2);
    e.ins(LDA, Operand::IndY(PTR as u8));
    e.imp(RTS);
    e.label("__mt_out");
    e.imm(LDA, 0);
    e.imp(RTS);

    // Finds change-list entry (map ARG4, column ARG0, row ARG2): X = index and C = 0,
    // or X = OV_COUNT and C = 1 when there is none.
    e.label("__ov_find");
    e.imm(LDX, 0);
    e.label("__ovf_loop");
    e.abs(CPX, OV_COUNT);
    e.br(BCS, "__ovf_done");
    for (arr, key) in [(OV_ID, ARG4), (OV_COL, ARG0), (OV_ROW, ARG2)] {
        e.ins(LDA, Operand::abs_x(arr));
        e.abs(CMP, key);
        e.br(BNE, "__ovf_next");
    }
    e.imp(CLC);
    e.imp(RTS);
    e.label("__ovf_next");
    e.imp(INX);
    e.jmp("__ovf_loop");
    e.label("__ovf_done");
    e.imp(RTS);

    // ARG0 = tx, ARG1 = width, ARG2 = ty, ARG3 = tile, ARG4 = map id.
    // Records the change (A = 1, or 0 when the list is full or the cell is outside the map)
    // and, if the cell is on screen, writes it there too.
    e.label("__set_map_tile");
    e.imm(LDA, 0);
    e.abs(STA, OV_RES);
    e.abs(LDA, ARG0);
    e.abs(CMP, ARG1);
    e.br(BCS, "__smt_done");
    e.abs(LDA, ARG2);
    e.imm(CMP, 30);
    e.br(BCS, "__smt_done");
    e.jsr("__ov_find");
    e.br(BCC, "__smt_store");
    e.imm(CPX, OV_MAX);
    e.br(BCS, "__smt_screen"); // full: not remembered, but still drawn
    e.abs(LDA, ARG4);
    e.ins(STA, Operand::abs_x(OV_ID));
    e.abs(LDA, ARG0);
    e.ins(STA, Operand::abs_x(OV_COL));
    e.abs(LDA, ARG2);
    e.ins(STA, Operand::abs_x(OV_ROW));
    e.abs(INC, OV_COUNT);
    e.label("__smt_store");
    e.abs(LDA, ARG3);
    e.ins(STA, Operand::abs_x(OV_TILE));
    e.imm(LDA, 1);
    e.abs(STA, OV_RES);
    // On screen: the map is the one drawn, the row is below the status bar and the
    // column is one of the 33 that can be visible (the ring holds exactly those).
    e.label("__smt_screen");
    e.abs(LDA, MAP_ID);
    e.abs(CMP, ARG4);
    e.br(BNE, "__smt_done");
    e.abs(LDA, ARG2);
    e.abs(CMP, STATUS_ROWS);
    e.br(BCC, "__smt_done");
    e.abs(LDA, ARG0);
    e.imp(SEC);
    e.abs(SBC, CAM_COL);
    e.br(BCC, "__smt_done");
    e.imm(CMP, 33);
    e.br(BCS, "__smt_done");
    e.abs(LDA, ARG2);
    e.acc(LSR);
    e.acc(LSR);
    e.acc(LSR);
    e.abs(STA, TMP1);
    e.abs(LDA, ARG0);
    e.imm(AND, 0x20);
    e.acc(LSR);
    e.acc(LSR);
    e.acc(LSR);
    e.imm(ORA, 0x20);
    e.imp(CLC);
    e.abs(ADC, TMP1);
    e.abs(STA, TMP1);
    e.abs(LDA, ARG2);
    for _ in 0..5 {
        e.acc(ASL);
    }
    e.abs(STA, TMP0);
    e.abs(LDA, ARG0);
    e.imm(AND, 0x1F);
    e.abs(ORA, TMP0);
    e.abs(STA, TMP0);
    e.abs(LDA, ARG3);
    e.abs(STA, PRINT_BUF);
    e.imm(LDA, PRINT_BUF as u8);
    e.abs(STA, PTR);
    e.imm(LDA, (PRINT_BUF >> 8) as u8);
    e.abs(STA, PTR + 1);
    e.imm(LDA, 1);
    e.abs(STA, ARG2);
    e.jsr("__vram_write");
    e.label("__smt_done");
    e.abs(LDA, OV_RES);
    e.imp(RTS);

    // ARG4 = map id: removes that map's entries, keeping the others in order.
    e.label("__reset_map");
    e.imm(LDX, 0);
    e.imm(LDY, 0);
    e.label("__rm_loop");
    e.abs(CPX, OV_COUNT);
    e.br(BCS, "__rm_done");
    e.ins(LDA, Operand::abs_x(OV_ID));
    e.abs(CMP, ARG4);
    e.br(BEQ, "__rm_skip");
    for arr in [OV_ID, OV_COL, OV_ROW, OV_TILE] {
        e.ins(LDA, Operand::abs_x(arr));
        e.ins(STA, Operand::abs_y(arr));
    }
    e.imp(INY);
    e.label("__rm_skip");
    e.imp(INX);
    e.jmp("__rm_loop");
    e.label("__rm_done");
    e.abs(STY, OV_COUNT);
    e.imp(RTS);

    e.comment("---- runtime: metasprites ----");
    // ARG0 = first slot, ARG1 = x, ARG2 = y, PTR = [count, (dx, dy, tile, attr)...].
    // Returns ARG0 + count. A piece whose position wraps past a screen edge is hidden.
    e.label("__metasprite");
    e.imm(LDY, 0);
    e.ins(LDA, Operand::IndY(PTR as u8));
    e.abs(STA, TMP3);
    e.imp(INY);
    e.abs(LDA, ARG0);
    e.acc(ASL);
    e.acc(ASL);
    e.imp(TAX);
    e.label("__ms_piece");
    e.imm(LDA, 0);
    e.abs(STA, TMP5); // hidden flag
    // For each axis: pos = base + d; it wrapped iff carry != sign(d).
    for (base, out) in [(ARG1, TMP1), (ARG2, TMP4)] {
        e.ins(LDA, Operand::IndY(PTR as u8));
        e.abs(STA, TMP0);
        e.imp(CLC);
        e.abs(ADC, base);
        e.abs(STA, out);
        e.imm(LDA, 0);
        e.acc(ROL); // A = carry
        e.abs(STA, TMP2);
        e.abs(LDA, TMP0);
        e.acc(ASL); // carry = sign of d
        e.imm(LDA, 0);
        e.acc(ROL);
        e.abs(EOR, TMP2);
        e.abs(ORA, TMP5);
        e.abs(STA, TMP5);
        e.imp(INY);
    }
    e.ins(LDA, Operand::IndY(PTR as u8));
    e.ins(STA, Operand::abs_x(OAM_SHADOW + 1));
    e.imp(INY);
    e.ins(LDA, Operand::IndY(PTR as u8));
    e.ins(STA, Operand::abs_x(OAM_SHADOW + 2));
    e.imp(INY);
    e.abs(LDA, TMP1);
    e.ins(STA, Operand::abs_x(OAM_SHADOW + 3));
    e.abs(LDA, TMP5);
    e.br(BNE, "__ms_hide");
    e.abs(LDA, TMP4);
    e.jmp("__ms_store");
    e.label("__ms_hide");
    e.imm(LDA, 0xFF);
    e.label("__ms_store");
    e.ins(STA, Operand::abs_x(OAM_SHADOW));
    for _ in 0..4 {
        e.imp(INX);
    }
    e.abs(DEC, TMP3);
    e.br(BNE, "__ms_piece");
    e.imm(LDY, 0);
    e.ins(LDA, Operand::IndY(PTR as u8));
    e.imp(CLC);
    e.abs(ADC, ARG0);
    e.imp(RTS);

    e.comment("---- runtime: sprites, input, random, sound ----");
    // ARG0 = index, ARG1 = x, ARG2 = y, ARG3 = tile, ARG4 = attributes
    e.label("__sprite");
    e.abs(LDA, ARG0);
    e.acc(ASL);
    e.acc(ASL);
    e.imp(TAX);
    for (slot, off) in [(ARG2, 0u16), (ARG3, 1), (ARG4, 2), (ARG1, 3)] {
        e.abs(LDA, slot);
        e.ins(STA, Operand::abs_x(OAM_SHADOW + off));
    }
    e.imp(RTS);

    // X = port; returns buttons in A (bit 7 = A ... bit 0 = Right)
    e.label("__joypad");
    e.imm(LDA, 1);
    e.abs(STA, JOY1);
    e.imm(LDA, 0);
    e.abs(STA, JOY1);
    e.imm(LDY, 8);
    e.label("__joypad_loop");
    e.ins(LDA, Operand::abs_x(JOY1));
    e.acc(LSR);
    e.abs(ROL, TMP4);
    e.imp(DEY);
    e.br(BNE, "__joypad_loop");
    e.abs(LDA, TMP4);
    e.imp(RTS);

    // 8-bit Galois LFSR (x^8 + x^4 + x^3 + x^2 + 1), period 255
    e.label("__rand");
    e.abs(LDA, RNG);
    e.acc(ASL);
    e.br(BCC, "__rand_done");
    e.imm(EOR, 0x1D);
    e.label("__rand_done");
    e.abs(STA, RNG);
    e.imp(RTS);

    // A:X = 11-bit timer period for pulse channel 1
    e.label("__play_tone");
    e.imp(PHA);
    e.abs(LDA, APU_EN);
    e.imm(ORA, 0x01);
    e.abs(STA, APU_EN);
    e.abs(STA, 0x4015);
    e.imm(LDA, 0xBF); // 50% duty, length halt, constant volume 15
    e.abs(STA, 0x4000);
    e.imm(LDA, 0x08); // sweep off
    e.abs(STA, 0x4001);
    e.imp(PLA);
    e.abs(STA, 0x4002);
    e.imp(TXA);
    e.imm(AND, 0x07);
    e.abs(STA, 0x4003);
    e.imp(RTS);

    e.comment("---- runtime: sound effects ----");
    // A = effect id (masked to the table). The timer is cleared first and set last so an
    // NMI arriving mid-setup never ticks a half-written effect.
    e.label("__sfx");
    e.imm(AND, 0x1F);
    e.ins(CMP, Operand::Abs(crate::asm::Val::Label("__sfx_count".into(), 0)));
    e.br(BCS, "__sfx_done");
    e.acc(ASL);
    e.acc(ASL);
    e.acc(ASL);
    e.imp(TAX);
    e.imm(LDA, 0);
    e.abs(STA, SFX_TIMER);
    for (off, dst) in [(0u16, SFX_CH), (1, SFX_PLO), (3, SFX_DELTA), (5, SFX_MAXVOL), (6, SFX_DUTY)] {
        e.ins(LDA, Operand::AbsX(crate::asm::Val::Label("__sfx_table".into(), off as i32)));
        e.abs(STA, dst);
    }
    e.abs(LDA, SFX_CH);
    e.br(BNE, "__sfx_noise");
    e.imm(LDA, 0x08); // sweep off
    e.abs(STA, 0x4005);
    e.ins(LDA, Operand::AbsX(crate::asm::Val::Label("__sfx_table".into(), 2)));
    e.abs(STA, 0x4007);
    e.imm(LDA, 0x02);
    e.jmp("__sfx_enable");
    e.label("__sfx_noise");
    e.imm(LDA, 0);
    e.abs(STA, 0x400F);
    e.imm(LDA, 0x08);
    e.label("__sfx_enable");
    e.abs(ORA, APU_EN);
    e.abs(STA, APU_EN);
    e.abs(STA, 0x4015);
    e.ins(LDA, Operand::AbsX(crate::asm::Val::Label("__sfx_table".into(), 4)));
    e.abs(STA, SFX_TIMER);
    e.label("__sfx_done");
    e.imp(RTS);

    // Called by the NMI every frame: volume = min(frames left, max), period += delta.
    e.label("__sfx_tick");
    e.abs(LDA, SFX_TIMER);
    e.br(BEQ, "__sfx_done");
    e.abs(DEC, SFX_TIMER);
    e.br(BEQ, "__sfx_off");
    e.abs(LDA, SFX_TIMER);
    e.abs(CMP, SFX_MAXVOL);
    e.br(BCC, "__sfx_vol");
    e.abs(LDA, SFX_MAXVOL);
    e.label("__sfx_vol");
    e.abs(ORA, SFX_DUTY);
    e.abs(LDX, SFX_CH);
    e.br(BNE, "__sfx_tick_noise");
    e.abs(STA, 0x4004);
    e.abs(LDA, SFX_PLO);
    e.imp(CLC);
    e.abs(ADC, SFX_DELTA);
    e.abs(STA, SFX_PLO);
    e.abs(STA, 0x4006);
    e.imp(RTS);
    e.label("__sfx_tick_noise");
    e.abs(STA, 0x400C);
    e.abs(LDA, SFX_PLO);
    e.imm(AND, 0x0F);
    e.abs(STA, 0x400E);
    e.imp(RTS);
    e.label("__sfx_off");
    e.abs(LDA, SFX_CH);
    e.br(BNE, "__sfx_off_noise");
    e.imm(LDA, !0x02u8);
    e.jmp("__sfx_off_write");
    e.label("__sfx_off_noise");
    e.imm(LDA, !0x08u8);
    e.label("__sfx_off_write");
    e.abs(AND, APU_EN);
    e.abs(STA, APU_EN);
    e.abs(STA, 0x4015);
    e.imp(RTS);

    emit_music(e);

}

/// The NMI-driven music engine: one generated routine per voice (absolute addressing).
fn emit_music(e: &mut Emitter) {
    use crate::asm::Val;
    e.comment("---- runtime: music ----");
    e.label("__music_tick");
    for v in 0..3 {
        e.jsr(&format!("__mu{v}"));
    }
    e.imp(RTS);

    for (v, &b) in VOICES.iter().enumerate() {
        let l = |s: &str| format!("__mu{v}_{s}");
        e.label(&format!("__mu{v}"));
        e.abs(LDA, MU_ON);
        e.imm(AND, 1 << v);
        e.br(BNE, &l("on"));
        e.imp(RTS);
        e.label(&l("on"));
        e.abs(DEC, b + 4);
        e.br(BEQ, &l("read"));
        e.imp(RTS);
        // Read events until one with a duration.
        e.label(&l("read"));
        e.abs(LDA, b);
        e.abs(STA, NMI_PTR);
        e.abs(LDA, b + 1);
        e.abs(STA, NMI_PTR + 1);
        e.imm(LDY, 0);
        e.ins(LDA, Operand::IndY(NMI_PTR as u8));
        e.abs(STA, MU_NOTE);
        e.imp(INY);
        e.ins(LDA, Operand::IndY(NMI_PTR as u8));
        e.abs(STA, MU_ARG);
        e.abs(LDA, b);
        e.imp(CLC);
        e.imm(ADC, 2);
        e.abs(STA, b);
        e.br(BCC, &l("nc"));
        e.abs(INC, b + 1);
        e.label(&l("nc"));
        e.abs(LDA, MU_NOTE);
        e.imm(CMP, crate::mml::EV_END);
        e.br(BNE, &l("not_end"));
        e.abs(LDA, b + 2);
        e.abs(STA, b);
        e.abs(LDA, b + 3);
        e.abs(STA, b + 1);
        e.jmp(&l("read"));
        e.label(&l("not_end"));
        e.imm(CMP, crate::mml::EV_REST);
        e.br(BEQ, &l("rest"));
        e.imm(CMP, crate::mml::EV_CONT);
        e.br(BEQ, &l("cont"));
        e.imm(CMP, 0x90);
        e.br(BCS, &l("duty"));
        e.imm(CMP, 0x80);
        e.br(BCS, &l("vol"));
        e.abs(LDA, MU_ARG);
        e.abs(STA, b + 4);
        e.jmp(&l("note"));
        e.label(&l("duty"));
        e.imm(AND, 0x03); // d -> d << 6
        e.acc(LSR);
        e.acc(ROR);
        e.acc(ROR);
        e.abs(STA, b + 6);
        e.jmp(&l("read"));
        e.label(&l("vol"));
        e.imm(AND, 0x0F);
        e.abs(STA, b + 5);
        e.jmp(&l("read"));
        e.label(&l("cont"));
        e.abs(LDA, MU_ARG);
        e.abs(STA, b + 4);
        e.imp(RTS);
        e.label(&l("rest"));
        e.abs(LDA, MU_ARG);
        e.abs(STA, b + 4);

        if v == 1 {
            // bass: triangle; a rest turns the channel off
            e.abs(LDA, APU_EN);
            e.imm(AND, !0x04u8);
            e.abs(STA, APU_EN);
            e.abs(STA, 0x4015);
            e.imp(RTS);
            e.label(&l("note"));
            e.abs(LDX, MU_NOTE);
            e.imm(LDA, 0xFF); // linear counter: hold
            e.abs(STA, 0x4008);
            // the triangle sounds an octave below a pulse with the same period
            e.ins(LDA, Operand::AbsX(Val::Label("__per_lo".into(), 12)));
            e.abs(STA, 0x400A);
            e.ins(LDA, Operand::AbsX(Val::Label("__per_hi".into(), 12)));
            e.abs(STA, 0x400B);
            e.abs(LDA, APU_EN);
            e.imm(ORA, 0x04);
            e.abs(STA, APU_EN);
            e.abs(STA, 0x4015);
            e.imp(RTS);
            continue;
        }

        let (reg, enable) = if v == 0 { (0x4000u16, 0x01u8) } else { (0x4004, 0x02) };
        // harmony shares pulse 2 with sound effects and yields while one plays
        let yield_to_sfx = |e: &mut Emitter, tag: &str| {
            if v == 2 {
                let free = format!("__mu{v}_free_{tag}");
                e.abs(LDA, SFX_TIMER);
                e.br(BEQ, &free);
                e.abs(LDA, SFX_CH);
                e.br(BNE, &free);
                e.imp(RTS);
                e.label(&free);
            }
        };
        yield_to_sfx(e, "rest");
        e.abs(LDA, b + 6);
        e.imm(ORA, 0x30); // length halt + constant volume, volume 0
        e.abs(STA, reg);
        e.imp(RTS);
        e.label(&l("note"));
        yield_to_sfx(e, "note");
        e.abs(LDX, MU_NOTE);
        e.abs(LDA, b + 6);
        e.imm(ORA, 0x30);
        e.abs(ORA, b + 5);
        e.abs(STA, reg);
        e.imm(LDA, 0x08); // sweep off
        e.abs(STA, reg + 1);
        e.ins(LDA, Operand::AbsX(Val::Label("__per_lo".into(), 0)));
        e.abs(STA, reg + 2);
        e.ins(LDA, Operand::AbsX(Val::Label("__per_hi".into(), 0)));
        e.abs(STA, reg + 3);
        e.abs(LDA, APU_EN);
        e.imm(ORA, enable);
        e.abs(STA, APU_EN);
        e.abs(STA, 0x4015);
        e.imp(RTS);
    }

    // Stops the engine and silences its channels (pulse 2 only if no effect is using it).
    e.label("__stop_music");
    e.imm(LDA, 0);
    e.abs(STA, MU_ON);
    e.imm(LDA, 0x30);
    e.abs(STA, 0x4000);
    e.abs(LDA, SFX_TIMER);
    e.br(BEQ, "__smu_p2");
    e.abs(LDA, SFX_CH);
    e.br(BEQ, "__smu_keep");
    e.label("__smu_p2");
    e.imm(LDA, 0x30);
    e.abs(STA, 0x4004);
    e.label("__smu_keep");
    e.abs(LDA, APU_EN);
    e.imm(AND, !0x04u8);
    e.abs(STA, APU_EN);
    e.abs(STA, 0x4015);
    e.imp(RTS);

    // Timer periods for notes 0-107 (pulse uses n, triangle n + 12).
    let periods: Vec<u16> = (0..108).map(crate::mml::pulse_period).collect();
    e.label("__per_lo");
    e.items.push(crate::asm::Item::Bytes(periods.iter().map(|p| *p as u8).collect()));
    e.label("__per_hi");
    e.items.push(crate::asm::Item::Bytes(periods.iter().map(|p| (p >> 8) as u8).collect()));
}

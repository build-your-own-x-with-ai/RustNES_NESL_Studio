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
        "map_tile" => sig!(Map, U8, U8 => U8),
        "metasprite" => sig!(U8, U8, U8, Meta => U8),
        "collide" => sig!(U8, U8, U8, U8, U8, U8, U8, U8 => U8),
        _ => return None,
    })
}

pub fn is_builtin_fn(name: &str) -> bool {
    signature(name).is_some()
}

impl Codegen {
    pub(crate) fn builtin_call(&mut self, name: &str, args: &[Expr], pos: Pos) -> R<Option<Type>> {
        let sig = signature(name).expect("caller checked");
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
            "stop_sound" => {
                self.e.imm(LDA, 0);
                self.e.abs(STA, 0x4015);
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
            "draw_map" => {
                let label = self.map_label(&args[0])?;
                self.pointer(&label);
                self.e.jsr("__draw_map");
            }
            "map_tile" => {
                let label = self.map_label(&args[0])?;
                self.slots(&[(&args[1], Type::U8, ARG0), (&args[2], Type::U8, ARG1)])?;
                self.pointer(&label);
                self.e.jsr("__map_tile");
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

    fn map_label(&self, e: &Expr) -> R<String> {
        if let ExprKind::Var(n) = &e.kind {
            if let Some(Sym::Map { label }) = self.lookup(n) {
                return Ok(label);
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
    e.label("__vram_write");
    e.abs(LDA, MASK_SHADOW);
    e.imm(AND, 0x18);
    e.br(BNE, "__vw_queue");
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
    for src in [TMP1, TMP0, TMP2] {
        e.abs(LDA, src);
        e.ins(STA, Operand::abs_x(VQ));
        e.imp(INX);
    }
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
    e.ins(LDY, Operand::abs_x(VQ));
    e.imp(INX);
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
    // PTR = map. Draws nametable 0 with rendering off, restoring rendering afterwards.
    e.label("__draw_map");
    e.abs(LDA, MASK_SHADOW);
    e.imp(PHA);
    e.br(BEQ, "__dm_off");
    e.jsr("__wait_vblank");
    e.jsr("__ppu_off");
    e.label("__dm_off");
    e.abs(BIT, PPUSTATUS);
    e.imm(LDA, 0x20);
    e.abs(STA, PPUADDR);
    e.imm(LDA, 0x00);
    e.abs(STA, PPUADDR);
    e.imm(LDX, 4); // 3 full pages + 192 bytes = 960
    e.imm(LDY, 0);
    e.label("__dm_loop");
    e.imm(CPX, 1);
    e.br(BNE, "__dm_byte");
    e.imm(CPY, 192);
    e.br(BEQ, "__dm_attr");
    e.label("__dm_byte");
    e.ins(LDA, Operand::IndY(PTR as u8));
    e.abs(STA, PPUDATA);
    e.imp(INY);
    e.br(BNE, "__dm_loop");
    e.abs(INC, PTR + 1);
    e.imp(DEX);
    e.jmp("__dm_loop");
    e.label("__dm_attr");
    e.imm(LDA, 0);
    e.imm(LDX, 64);
    e.label("__dm_attr_loop");
    e.abs(STA, PPUDATA);
    e.imp(DEX);
    e.br(BNE, "__dm_attr_loop");
    e.jsr("__fix_scroll");
    e.imp(PLA);
    e.br(BEQ, "__dm_done");
    e.jmp("__ppu_on");
    e.label("__dm_done");
    e.imp(RTS);

    // PTR = map, ARG0 = tx, ARG1 = ty; returns the tile, or 0 when out of bounds.
    e.label("__map_tile");
    e.abs(LDA, ARG0);
    e.imm(CMP, 32);
    e.br(BCS, "__mt_out");
    e.abs(LDA, ARG1);
    e.imm(CMP, 30);
    e.br(BCS, "__mt_out");
    // PTR += ty * 32
    e.imm(LDX, 0);
    e.abs(STX, TMP1);
    for _ in 0..5 {
        e.acc(ASL);
        e.abs(ROL, TMP1);
    }
    e.imp(CLC);
    e.abs(ADC, PTR);
    e.abs(STA, PTR);
    e.abs(LDA, TMP1);
    e.abs(ADC, PTR + 1);
    e.abs(STA, PTR + 1);
    e.abs(LDY, ARG0);
    e.ins(LDA, Operand::IndY(PTR as u8));
    e.imp(RTS);
    e.label("__mt_out");
    e.imm(LDA, 0);
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
    e.imm(LDA, 0x01);
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
}

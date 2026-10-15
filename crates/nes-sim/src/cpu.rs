//! NMOS 6502 core (official opcodes only; decimal mode disabled as on the NES 2A03).

/// Memory bus seen by the CPU.
pub trait Bus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

pub const FLAG_C: u8 = 0x01;
pub const FLAG_Z: u8 = 0x02;
pub const FLAG_I: u8 = 0x04;
pub const FLAG_D: u8 = 0x08;
pub const FLAG_B: u8 = 0x10;
pub const FLAG_U: u8 = 0x20;
pub const FLAG_V: u8 = 0x40;
pub const FLAG_N: u8 = 0x80;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpuError {
    IllegalOpcode { opcode: u8, pc: u16 },
}

impl std::fmt::Display for CpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuError::IllegalOpcode { opcode, pc } => {
                write!(f, "illegal opcode ${opcode:02X} at ${pc:04X}")
            }
        }
    }
}

impl std::error::Error for CpuError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub p: u8,
    pub pc: u16,
    pub cycles: u64,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Imp,
    Acc,
    Imm,
    Zp,
    Zpx,
    Zpy,
    Abs,
    Abx,
    Aby,
    Ind,
    Izx,
    Izy,
    Rel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Adc, And, Asl, Bcc, Bcs, Beq, Bit, Bmi, Bne, Bpl, Brk, Bvc, Bvs, Clc, Cld, Cli, Clv, Cmp,
    Cpx, Cpy, Dec, Dex, Dey, Eor, Inc, Inx, Iny, Jmp, Jsr, Lda, Ldx, Ldy, Lsr, Nop, Ora, Pha,
    Php, Pla, Plp, Rol, Ror, Rti, Rts, Sbc, Sec, Sed, Sei, Sta, Stx, Sty, Tax, Tay, Tsx, Txa,
    Txs, Tya,
}

/// Decodes an opcode into (operation, addressing mode, base cycles, page-cross penalty applies).
fn decode(opcode: u8) -> Option<(Op, Mode, u32, bool)> {
    use Mode::*;
    use Op::*;
    let d = match opcode {
        0x69 => (Adc, Imm, 2, false),
        0x65 => (Adc, Zp, 3, false),
        0x75 => (Adc, Zpx, 4, false),
        0x6D => (Adc, Abs, 4, false),
        0x7D => (Adc, Abx, 4, true),
        0x79 => (Adc, Aby, 4, true),
        0x61 => (Adc, Izx, 6, false),
        0x71 => (Adc, Izy, 5, true),

        0x29 => (And, Imm, 2, false),
        0x25 => (And, Zp, 3, false),
        0x35 => (And, Zpx, 4, false),
        0x2D => (And, Abs, 4, false),
        0x3D => (And, Abx, 4, true),
        0x39 => (And, Aby, 4, true),
        0x21 => (And, Izx, 6, false),
        0x31 => (And, Izy, 5, true),

        0x0A => (Asl, Acc, 2, false),
        0x06 => (Asl, Zp, 5, false),
        0x16 => (Asl, Zpx, 6, false),
        0x0E => (Asl, Abs, 6, false),
        0x1E => (Asl, Abx, 7, false),

        0x90 => (Bcc, Rel, 2, false),
        0xB0 => (Bcs, Rel, 2, false),
        0xF0 => (Beq, Rel, 2, false),
        0x30 => (Bmi, Rel, 2, false),
        0xD0 => (Bne, Rel, 2, false),
        0x10 => (Bpl, Rel, 2, false),
        0x50 => (Bvc, Rel, 2, false),
        0x70 => (Bvs, Rel, 2, false),

        0x24 => (Bit, Zp, 3, false),
        0x2C => (Bit, Abs, 4, false),

        0x00 => (Brk, Imp, 7, false),

        0x18 => (Clc, Imp, 2, false),
        0xD8 => (Cld, Imp, 2, false),
        0x58 => (Cli, Imp, 2, false),
        0xB8 => (Clv, Imp, 2, false),
        0x38 => (Sec, Imp, 2, false),
        0xF8 => (Sed, Imp, 2, false),
        0x78 => (Sei, Imp, 2, false),

        0xC9 => (Cmp, Imm, 2, false),
        0xC5 => (Cmp, Zp, 3, false),
        0xD5 => (Cmp, Zpx, 4, false),
        0xCD => (Cmp, Abs, 4, false),
        0xDD => (Cmp, Abx, 4, true),
        0xD9 => (Cmp, Aby, 4, true),
        0xC1 => (Cmp, Izx, 6, false),
        0xD1 => (Cmp, Izy, 5, true),

        0xE0 => (Cpx, Imm, 2, false),
        0xE4 => (Cpx, Zp, 3, false),
        0xEC => (Cpx, Abs, 4, false),
        0xC0 => (Cpy, Imm, 2, false),
        0xC4 => (Cpy, Zp, 3, false),
        0xCC => (Cpy, Abs, 4, false),

        0xC6 => (Dec, Zp, 5, false),
        0xD6 => (Dec, Zpx, 6, false),
        0xCE => (Dec, Abs, 6, false),
        0xDE => (Dec, Abx, 7, false),
        0xCA => (Dex, Imp, 2, false),
        0x88 => (Dey, Imp, 2, false),

        0x49 => (Eor, Imm, 2, false),
        0x45 => (Eor, Zp, 3, false),
        0x55 => (Eor, Zpx, 4, false),
        0x4D => (Eor, Abs, 4, false),
        0x5D => (Eor, Abx, 4, true),
        0x59 => (Eor, Aby, 4, true),
        0x41 => (Eor, Izx, 6, false),
        0x51 => (Eor, Izy, 5, true),

        0xE6 => (Inc, Zp, 5, false),
        0xF6 => (Inc, Zpx, 6, false),
        0xEE => (Inc, Abs, 6, false),
        0xFE => (Inc, Abx, 7, false),
        0xE8 => (Inx, Imp, 2, false),
        0xC8 => (Iny, Imp, 2, false),

        0x4C => (Jmp, Abs, 3, false),
        0x6C => (Jmp, Ind, 5, false),
        0x20 => (Jsr, Abs, 6, false),

        0xA9 => (Lda, Imm, 2, false),
        0xA5 => (Lda, Zp, 3, false),
        0xB5 => (Lda, Zpx, 4, false),
        0xAD => (Lda, Abs, 4, false),
        0xBD => (Lda, Abx, 4, true),
        0xB9 => (Lda, Aby, 4, true),
        0xA1 => (Lda, Izx, 6, false),
        0xB1 => (Lda, Izy, 5, true),

        0xA2 => (Ldx, Imm, 2, false),
        0xA6 => (Ldx, Zp, 3, false),
        0xB6 => (Ldx, Zpy, 4, false),
        0xAE => (Ldx, Abs, 4, false),
        0xBE => (Ldx, Aby, 4, true),

        0xA0 => (Ldy, Imm, 2, false),
        0xA4 => (Ldy, Zp, 3, false),
        0xB4 => (Ldy, Zpx, 4, false),
        0xAC => (Ldy, Abs, 4, false),
        0xBC => (Ldy, Abx, 4, true),

        0x4A => (Lsr, Acc, 2, false),
        0x46 => (Lsr, Zp, 5, false),
        0x56 => (Lsr, Zpx, 6, false),
        0x4E => (Lsr, Abs, 6, false),
        0x5E => (Lsr, Abx, 7, false),

        0xEA => (Nop, Imp, 2, false),

        0x09 => (Ora, Imm, 2, false),
        0x05 => (Ora, Zp, 3, false),
        0x15 => (Ora, Zpx, 4, false),
        0x0D => (Ora, Abs, 4, false),
        0x1D => (Ora, Abx, 4, true),
        0x19 => (Ora, Aby, 4, true),
        0x01 => (Ora, Izx, 6, false),
        0x11 => (Ora, Izy, 5, true),

        0x48 => (Pha, Imp, 3, false),
        0x08 => (Php, Imp, 3, false),
        0x68 => (Pla, Imp, 4, false),
        0x28 => (Plp, Imp, 4, false),

        0x2A => (Rol, Acc, 2, false),
        0x26 => (Rol, Zp, 5, false),
        0x36 => (Rol, Zpx, 6, false),
        0x2E => (Rol, Abs, 6, false),
        0x3E => (Rol, Abx, 7, false),

        0x6A => (Ror, Acc, 2, false),
        0x66 => (Ror, Zp, 5, false),
        0x76 => (Ror, Zpx, 6, false),
        0x6E => (Ror, Abs, 6, false),
        0x7E => (Ror, Abx, 7, false),

        0x40 => (Rti, Imp, 6, false),
        0x60 => (Rts, Imp, 6, false),

        0xE9 => (Sbc, Imm, 2, false),
        0xE5 => (Sbc, Zp, 3, false),
        0xF5 => (Sbc, Zpx, 4, false),
        0xED => (Sbc, Abs, 4, false),
        0xFD => (Sbc, Abx, 4, true),
        0xF9 => (Sbc, Aby, 4, true),
        0xE1 => (Sbc, Izx, 6, false),
        0xF1 => (Sbc, Izy, 5, true),

        0x85 => (Sta, Zp, 3, false),
        0x95 => (Sta, Zpx, 4, false),
        0x8D => (Sta, Abs, 4, false),
        0x9D => (Sta, Abx, 5, false),
        0x99 => (Sta, Aby, 5, false),
        0x81 => (Sta, Izx, 6, false),
        0x91 => (Sta, Izy, 6, false),

        0x86 => (Stx, Zp, 3, false),
        0x96 => (Stx, Zpy, 4, false),
        0x8E => (Stx, Abs, 4, false),
        0x84 => (Sty, Zp, 3, false),
        0x94 => (Sty, Zpx, 4, false),
        0x8C => (Sty, Abs, 4, false),

        0xAA => (Tax, Imp, 2, false),
        0xA8 => (Tay, Imp, 2, false),
        0xBA => (Tsx, Imp, 2, false),
        0x8A => (Txa, Imp, 2, false),
        0x9A => (Txs, Imp, 2, false),
        0x98 => (Tya, Imp, 2, false),

        _ => return None,
    };
    Some(d)
}

impl Cpu {
    pub fn new() -> Self {
        Cpu { a: 0, x: 0, y: 0, sp: 0xFD, p: FLAG_I | FLAG_U, pc: 0, cycles: 0 }
    }

    /// Reset: loads PC from $FFFC, SP=$FD, P=$24. Takes 7 cycles.
    pub fn reset<B: Bus>(&mut self, bus: &mut B) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.p = FLAG_I | FLAG_U;
        self.pc = read16(bus, 0xFFFC);
        self.cycles = 7;
    }

    /// Non-maskable interrupt: push PC and P (B clear), set I, jump via $FFFA. 7 cycles.
    pub fn nmi<B: Bus>(&mut self, bus: &mut B) {
        self.push16(bus, self.pc);
        self.push(bus, (self.p & !FLAG_B) | FLAG_U);
        self.p |= FLAG_I;
        self.pc = read16(bus, 0xFFFA);
        self.cycles += 7;
    }

    fn set_zn(&mut self, v: u8) {
        self.set_flag(FLAG_Z, v == 0);
        self.set_flag(FLAG_N, v & 0x80 != 0);
    }

    fn set_flag(&mut self, f: u8, on: bool) {
        if on {
            self.p |= f;
        } else {
            self.p &= !f;
        }
    }

    fn push<B: Bus>(&mut self, bus: &mut B, v: u8) {
        bus.write(0x100 | self.sp as u16, v);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pull<B: Bus>(&mut self, bus: &mut B) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        bus.read(0x100 | self.sp as u16)
    }

    fn push16<B: Bus>(&mut self, bus: &mut B, v: u16) {
        self.push(bus, (v >> 8) as u8);
        self.push(bus, v as u8);
    }

    fn pull16<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let lo = self.pull(bus) as u16;
        let hi = self.pull(bus) as u16;
        (hi << 8) | lo
    }

    fn fetch<B: Bus>(&mut self, bus: &mut B) -> u8 {
        let v = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }

    fn fetch16<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let lo = self.fetch(bus) as u16;
        let hi = self.fetch(bus) as u16;
        (hi << 8) | lo
    }

    /// Resolves the effective address for memory modes; returns (addr, page_crossed).
    fn operand_addr<B: Bus>(&mut self, bus: &mut B, mode: Mode) -> (u16, bool) {
        match mode {
            Mode::Imm => {
                let a = self.pc;
                self.pc = self.pc.wrapping_add(1);
                (a, false)
            }
            Mode::Zp => (self.fetch(bus) as u16, false),
            Mode::Zpx => (self.fetch(bus).wrapping_add(self.x) as u16, false),
            Mode::Zpy => (self.fetch(bus).wrapping_add(self.y) as u16, false),
            Mode::Abs => (self.fetch16(bus), false),
            Mode::Abx => {
                let base = self.fetch16(bus);
                let a = base.wrapping_add(self.x as u16);
                (a, (base & 0xFF00) != (a & 0xFF00))
            }
            Mode::Aby => {
                let base = self.fetch16(bus);
                let a = base.wrapping_add(self.y as u16);
                (a, (base & 0xFF00) != (a & 0xFF00))
            }
            Mode::Ind => {
                let ptr = self.fetch16(bus);
                // 6502 bug: the high byte is fetched without carrying into the page.
                let hi_ptr = (ptr & 0xFF00) | (ptr.wrapping_add(1) & 0x00FF);
                let lo = bus.read(ptr) as u16;
                let hi = bus.read(hi_ptr) as u16;
                ((hi << 8) | lo, false)
            }
            Mode::Izx => {
                let zp = self.fetch(bus).wrapping_add(self.x);
                (read16_zp(bus, zp), false)
            }
            Mode::Izy => {
                let zp = self.fetch(bus);
                let base = read16_zp(bus, zp);
                let a = base.wrapping_add(self.y as u16);
                (a, (base & 0xFF00) != (a & 0xFF00))
            }
            Mode::Imp | Mode::Acc | Mode::Rel => (0, false),
        }
    }

    fn adc(&mut self, m: u8) {
        let sum = self.a as u16 + m as u16 + (self.p & FLAG_C) as u16;
        let r = sum as u8;
        self.set_flag(FLAG_C, sum > 0xFF);
        self.set_flag(FLAG_V, (!(self.a ^ m) & (self.a ^ r) & 0x80) != 0);
        self.a = r;
        self.set_zn(r);
    }

    fn compare(&mut self, reg: u8, m: u8) {
        let r = reg.wrapping_sub(m);
        self.set_flag(FLAG_C, reg >= m);
        self.set_zn(r);
    }

    /// Executes one instruction and returns the number of cycles it took.
    pub fn step<B: Bus>(&mut self, bus: &mut B) -> Result<u32, CpuError> {
        let start_pc = self.pc;
        let opcode = bus.read(self.pc);
        let (op, mode, base, penalty) =
            decode(opcode).ok_or(CpuError::IllegalOpcode { opcode, pc: start_pc })?;
        self.pc = self.pc.wrapping_add(1);
        let mut cycles = base;

        let (addr, crossed) = self.operand_addr(bus, mode);
        if penalty && crossed {
            cycles += 1;
        }

        match op {
            // Loads / stores
            Op::Lda => {
                self.a = bus.read(addr);
                self.set_zn(self.a);
            }
            Op::Ldx => {
                self.x = bus.read(addr);
                self.set_zn(self.x);
            }
            Op::Ldy => {
                self.y = bus.read(addr);
                self.set_zn(self.y);
            }
            Op::Sta => bus.write(addr, self.a),
            Op::Stx => bus.write(addr, self.x),
            Op::Sty => bus.write(addr, self.y),

            // Arithmetic / logic
            Op::Adc => {
                let m = bus.read(addr);
                self.adc(m);
            }
            Op::Sbc => {
                let m = bus.read(addr);
                self.adc(!m);
            }
            Op::And => {
                self.a &= bus.read(addr);
                self.set_zn(self.a);
            }
            Op::Ora => {
                self.a |= bus.read(addr);
                self.set_zn(self.a);
            }
            Op::Eor => {
                self.a ^= bus.read(addr);
                self.set_zn(self.a);
            }
            Op::Cmp => {
                let m = bus.read(addr);
                self.compare(self.a, m);
            }
            Op::Cpx => {
                let m = bus.read(addr);
                self.compare(self.x, m);
            }
            Op::Cpy => {
                let m = bus.read(addr);
                self.compare(self.y, m);
            }
            Op::Bit => {
                let m = bus.read(addr);
                self.set_flag(FLAG_Z, self.a & m == 0);
                self.set_flag(FLAG_V, m & 0x40 != 0);
                self.set_flag(FLAG_N, m & 0x80 != 0);
            }

            // Shifts / rotates and memory inc/dec (read-modify-write)
            Op::Asl | Op::Lsr | Op::Rol | Op::Ror | Op::Inc | Op::Dec => {
                let v = if mode == Mode::Acc { self.a } else { bus.read(addr) };
                let carry_in = self.p & FLAG_C;
                let r = match op {
                    Op::Asl => {
                        self.set_flag(FLAG_C, v & 0x80 != 0);
                        v << 1
                    }
                    Op::Lsr => {
                        self.set_flag(FLAG_C, v & 0x01 != 0);
                        v >> 1
                    }
                    Op::Rol => {
                        self.set_flag(FLAG_C, v & 0x80 != 0);
                        (v << 1) | carry_in
                    }
                    Op::Ror => {
                        self.set_flag(FLAG_C, v & 0x01 != 0);
                        (v >> 1) | (carry_in << 7)
                    }
                    Op::Inc => v.wrapping_add(1),
                    _ => v.wrapping_sub(1),
                };
                self.set_zn(r);
                if mode == Mode::Acc {
                    self.a = r;
                } else {
                    bus.write(addr, r);
                }
            }

            // Register inc/dec and transfers
            Op::Inx => {
                self.x = self.x.wrapping_add(1);
                self.set_zn(self.x);
            }
            Op::Iny => {
                self.y = self.y.wrapping_add(1);
                self.set_zn(self.y);
            }
            Op::Dex => {
                self.x = self.x.wrapping_sub(1);
                self.set_zn(self.x);
            }
            Op::Dey => {
                self.y = self.y.wrapping_sub(1);
                self.set_zn(self.y);
            }
            Op::Tax => {
                self.x = self.a;
                self.set_zn(self.x);
            }
            Op::Tay => {
                self.y = self.a;
                self.set_zn(self.y);
            }
            Op::Txa => {
                self.a = self.x;
                self.set_zn(self.a);
            }
            Op::Tya => {
                self.a = self.y;
                self.set_zn(self.a);
            }
            Op::Tsx => {
                self.x = self.sp;
                self.set_zn(self.x);
            }
            Op::Txs => self.sp = self.x,

            // Flags
            Op::Clc => self.p &= !FLAG_C,
            Op::Cld => self.p &= !FLAG_D,
            Op::Cli => self.p &= !FLAG_I,
            Op::Clv => self.p &= !FLAG_V,
            Op::Sec => self.p |= FLAG_C,
            Op::Sed => self.p |= FLAG_D,
            Op::Sei => self.p |= FLAG_I,

            // Stack
            Op::Pha => self.push(bus, self.a),
            Op::Php => self.push(bus, self.p | FLAG_B | FLAG_U),
            Op::Pla => {
                self.a = self.pull(bus);
                self.set_zn(self.a);
            }
            Op::Plp => {
                let v = self.pull(bus);
                self.p = (v & !FLAG_B) | FLAG_U;
            }

            // Branches
            Op::Bcc | Op::Bcs | Op::Beq | Op::Bne | Op::Bmi | Op::Bpl | Op::Bvc | Op::Bvs => {
                let offset = self.fetch(bus) as i8;
                let taken = match op {
                    Op::Bcc => self.p & FLAG_C == 0,
                    Op::Bcs => self.p & FLAG_C != 0,
                    Op::Bne => self.p & FLAG_Z == 0,
                    Op::Beq => self.p & FLAG_Z != 0,
                    Op::Bpl => self.p & FLAG_N == 0,
                    Op::Bmi => self.p & FLAG_N != 0,
                    Op::Bvc => self.p & FLAG_V == 0,
                    _ => self.p & FLAG_V != 0,
                };
                if taken {
                    let target = self.pc.wrapping_add(offset as i16 as u16);
                    cycles += 1;
                    if target & 0xFF00 != self.pc & 0xFF00 {
                        cycles += 1;
                    }
                    self.pc = target;
                }
            }

            // Jumps / subroutines / interrupts
            Op::Jmp => self.pc = addr,
            Op::Jsr => {
                self.push16(bus, self.pc.wrapping_sub(1));
                self.pc = addr;
            }
            Op::Rts => self.pc = self.pull16(bus).wrapping_add(1),
            Op::Rti => {
                let v = self.pull(bus);
                self.p = (v & !FLAG_B) | FLAG_U;
                self.pc = self.pull16(bus);
            }
            Op::Brk => {
                // BRK skips a padding byte.
                self.push16(bus, self.pc.wrapping_add(1));
                self.push(bus, self.p | FLAG_B | FLAG_U);
                self.p |= FLAG_I;
                self.pc = read16(bus, 0xFFFE);
            }
            Op::Nop => {}
        }

        self.cycles += cycles as u64;
        Ok(cycles)
    }
}

fn read16<B: Bus>(bus: &mut B, addr: u16) -> u16 {
    let lo = bus.read(addr) as u16;
    let hi = bus.read(addr.wrapping_add(1)) as u16;
    (hi << 8) | lo
}

fn read16_zp<B: Bus>(bus: &mut B, zp: u8) -> u16 {
    let lo = bus.read(zp as u16) as u16;
    let hi = bus.read(zp.wrapping_add(1) as u16) as u16;
    (hi << 8) | lo
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_151_official_opcodes() {
        let n = (0..=255u8).filter(|&o| decode(o).is_some()).count();
        assert_eq!(n, 151);
    }
}

#![allow(dead_code)]
//! End-to-end harness: compile NESL → load the ROM into the nes-sim test machine → run.

use nes_sim::TestNes;
use nesl::{compile, CompileOutput, Diagnostic};

pub struct Run {
    pub nes: TestNes,
    pub out: CompileOutput,
}

pub fn build(src: &str) -> CompileOutput {
    match compile(src) {
        Ok(o) => o,
        Err(errs) => {
            let msgs: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
            panic!("compile failed:\n{}", msgs.join("\n"));
        }
    }
}

fn boot(src: &str) -> Run {
    let out = build(src);
    let nes = TestNes::from_ines(&out.rom).expect("valid iNES");
    Run { nes, out }
}

/// Compiles and runs until `main` returns.
pub fn run(src: &str) -> Run {
    let mut r = boot(src);
    let halt = r.out.symbols["__halt"];
    if let Err(e) = r.nes.run_until_pc(halt, 20_000_000) {
        panic!("program did not finish: {e}\n{}", r.out.listing);
    }
    r
}

/// Compiles, lets `setup` tweak the machine, then runs `frames` frames.
pub fn run_frames(src: &str, frames: u64, setup: impl FnOnce(&mut TestNes)) -> Run {
    let mut r = boot(src);
    setup(&mut r.nes);
    r.nes.run_frames(frames).expect("run frames");
    r
}

pub fn errors(src: &str) -> Vec<Diagnostic> {
    match compile(src) {
        Ok(_) => panic!("expected compile errors"),
        Err(e) => e,
    }
}

impl Run {
    pub fn addr(&self, name: &str) -> u16 {
        self.out.vars.get(name).unwrap_or_else(|| panic!("no global `{name}`")).addr
    }
    pub fn u8(&self, name: &str) -> u8 {
        self.nes.peek(self.addr(name))
    }
    pub fn u16(&self, name: &str) -> u16 {
        let a = self.addr(name);
        self.nes.peek(a) as u16 | (self.nes.peek(a + 1) as u16) << 8
    }
    pub fn i8(&self, name: &str) -> i8 {
        self.u8(name) as i8
    }
    pub fn bytes(&self, name: &str) -> Vec<u8> {
        let v = &self.out.vars[name];
        (0..v.size).map(|i| self.nes.peek(v.addr + i)).collect()
    }
    pub fn vram(&self, addr: u16, len: usize) -> Vec<u8> {
        self.nes.bus.vram[addr as usize..addr as usize + len].to_vec()
    }
}

// ---------------------------------------------------------------- wide maps (004+)

/// A map `width` columns wide whose cell (c, r) is a printable character derived from
/// both coordinates, so any misplaced column or row is detected.
pub fn cell(c: usize, r: usize) -> u8 {
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"[(c * 7 + r * 3) % 36]
}

pub fn wide_map(name: &str, width: usize) -> String {
    let rows: Vec<String> = (0..30)
        .map(|r| format!("    \"{}\"", (0..width).map(|c| cell(c, r) as char).collect::<String>()))
        .collect();
    format!("map {name} = [\n{}\n];\n", rows.join(",\n"))
}

/// Nametable byte for ring column `slot` (0-63) and row `r`.
pub fn nt(r: &Run, slot: usize, row: usize) -> u8 {
    let base = if slot < 32 { 0x2000 } else { 0x2400 };
    r.nes.bus.vram[base + row * 32 + slot % 32]
}


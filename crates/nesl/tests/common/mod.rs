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
    pub fn bytes(&self, name: &str) -> Vec<u8> {
        let v = &self.out.vars[name];
        (0..v.size).map(|i| self.nes.peek(v.addr + i)).collect()
    }
    pub fn vram(&self, addr: u16, len: usize) -> Vec<u8> {
        self.nes.bus.vram[addr as usize..addr as usize + len].to_vec()
    }
}

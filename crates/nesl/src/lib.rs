//! NESL compiler library: compiles NESL source code into NES (iNES) ROM images.

pub mod asm;
pub mod ast;
pub mod builtins;
pub mod chr;
pub mod codegen;
pub mod lexer;
pub mod mml;
pub mod parser;
pub mod rom;
pub mod runtime;

use std::fmt;

/// A compile error with a 1-based source position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub line: u32,
    pub col: u32,
    pub message: String,
}

impl Diagnostic {
    pub fn new(pos: ast::Pos, message: impl Into<String>) -> Self {
        Diagnostic { line: pos.line, col: pos.col, message: message.into() }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}: error: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for Diagnostic {}

#[cfg(test)]
mod tests;

/// A global variable's location, exposed for tooling and tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarInfo {
    pub addr: u16,
    pub size: u16,
}

#[derive(Clone, Debug)]
pub struct CompileOutput {
    pub rom: Vec<u8>,
    pub symbols: std::collections::BTreeMap<String, u16>,
    pub vars: std::collections::BTreeMap<String, VarInfo>,
    pub listing: String,
    pub prg_used: usize,
    /// 0 (NROM) or 2 (UNROM).
    pub mapper: u8,
}

/// Compiles NESL source into an iNES ROM image.
pub fn compile(src: &str) -> Result<CompileOutput, Vec<Diagnostic>> {
    let prog = parser::parse(src).map_err(|d| vec![d])?;
    let top = ast::Pos { line: 1, col: 1 };
    
    // Build tiles/CHR
    let mut cg_temp = codegen::Codegen::new();
    cg_temp.program(&prog);
    if !cg_temp.errors.is_empty() {
        return Err(cg_temp.errors);
    }
    
    let mut tiles = Vec::new();
    let mut errors = Vec::new();
    for t in &cg_temp.tiles {
        let rows: Vec<&str> = t.rows.iter().map(|(r, _)| r.as_str()).collect();
        match chr::encode_tile(&rows) {
            Ok(data) => tiles.push((t.index, data)),
            Err(m) => errors.push(Diagnostic::new(t.pos, m)),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let chr = chr::build_chr(&tiles);
    
    // Try NROM assembly at $8000
    let asm_nrom = asm::assemble(&cg_temp.e.items, 0x8000);
    if cg_temp.mmc3 {
        let asm = asm_nrom.map_err(|m| vec![Diagnostic::new(top, m)])?;
        let sym = |n: &str| asm.symbols[n];
        let rom = rom::build_mmc3(&asm.code, sym("__nmi"), sym("__reset"), sym("__irq"), &chr)
            .map_err(|m| vec![Diagnostic::new(top, m)])?;
        return Ok(CompileOutput {
            rom,
            prg_used: asm.code.len(),
            mapper: 4,
            symbols: asm.symbols,
            vars: cg_temp.vars,
            listing: asm.listing,
        });
    }
    let use_unrom = if let Ok(asm) = &asm_nrom {
        asm.code.len() > 32762 // CHR is a separate 8KB chip in NROM
    } else {
        true // assembly failed, might be due to size
    };
    
    if use_unrom {
        return compile_unrom(&prog, &chr);
    }
    
    // NROM path
    let asm = asm_nrom.map_err(|m| vec![Diagnostic::new(top, m)])?;
    let sym = |n: &str| asm.symbols[n];
    let rom = rom::build_ines(&asm.code, sym("__nmi"), sym("__reset"), sym("__irq"), &chr)
        .map_err(|m| vec![Diagnostic::new(top, m)])?;
    
    Ok(CompileOutput {
        rom,
        prg_used: asm.code.len(),
        mapper: 0,
        symbols: asm.symbols,
        vars: cg_temp.vars,
        listing: asm.listing,
    })
}


fn compile_unrom(prog: &ast::Program, chr: &[u8]) -> Result<CompileOutput, Vec<Diagnostic>> {
    let top = ast::Pos { line: 1, col: 1 };
    let mut cg = codegen::Codegen::new();
    cg.banked = true;
    cg.program(prog);
    if !cg.errors.is_empty() {
        return Err(cg.errors);
    }
    
    // Emit Equ items for banked maps and build bank data
    let (banks, _) = cg.emit_banked_maps(chr);
    
    // Now assemble the fixed bank (code + runtime) at $C000
    let fixed_asm = asm::assemble(&cg.e.items, 0xC000)
        .map_err(|m| vec![Diagnostic::new(top, m)])?;
    
    if fixed_asm.code.len() > 0x4000 - 6 {
        return Err(vec![Diagnostic::new(top, format!(
            "fixed bank too large: {} bytes (max {})",
            fixed_asm.code.len(), 0x4000 - 6
        ))]);
    }
    
    let sym = |n: &str| fixed_asm.symbols[n];
    let rom = rom::build_unrom(banks, &fixed_asm.code, sym("__nmi"), sym("__reset"), sym("__irq"))
        .map_err(|m| vec![Diagnostic::new(top, m)])?;
    
    Ok(CompileOutput {
        rom,
        prg_used: fixed_asm.code.len(),
        mapper: 2,
        symbols: fixed_asm.symbols,
        vars: cg.vars,
        listing: fixed_asm.listing,
    })
}


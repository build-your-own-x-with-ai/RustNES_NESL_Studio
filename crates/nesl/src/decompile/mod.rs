//! NESL decompiler: converts .nes ROMs with debug symbols back to NESL source code.

pub mod rom_parser;
pub mod symbol_loader;
pub mod ast_builder;
pub mod codegen;
pub mod chr_decode;
pub mod advanced;  // Advanced decompiler for ROMs without debug symbols
pub mod nesl_codegen;  // NESL code generator (experimental)
pub mod control_flow;  // Control flow reconstruction
pub mod mario;  // Super Mario Bros enhanced decompiler
pub mod enhanced_pseudocode;  // Phase 2: Enhanced pseudocode with control flow
pub mod register_elimination;  // Phase 3: Register elimination and SSA
pub mod dataflow;  // Phase 3: Data flow analysis and optimization
pub mod high_level_codegen;  // Phase 4: High-level statement generation
pub mod complete;  // Phase 5: Complete integrated decompiler
pub mod final_complete;  // Final: Truly compilable NESL output
pub mod chr_extract;  // CHR ROM extraction
pub mod data_analysis;  // Phase 6: Data segment analysis
pub mod chr_integration;  // Phase 7: CHR ROM integration
pub mod complete_generator;  // Phase 8: Complete code generation
pub mod ultimate;  // Ultimate: Fully runnable with all resources
pub mod c_codegen;  // Feature 020: C 静态重编译（帧级一致可验证）

use std::path::Path;
use std::io;
use std::fmt;

/// Errors that can occur during decompilation.
#[derive(Debug)]
pub enum DecompileError {
    /// ROM file not found at the given path.
    RomNotFound { path: String },
    /// Debug symbols file not found.
    SymbolsNotFound { path: String },
    /// ROM file is invalid or corrupted.
    InvalidRom { reason: String },
    /// Debug symbols file is invalid or corrupted.
    InvalidSymbols { reason: String },
    /// Unsupported symbol table version.
    UnsupportedVersion { found: String, supported: String },
    /// Unsupported mapper type.
    UnsupportedMapper { mapper: u8 },
    /// Symbols don't match the ROM (checksum/size mismatch).
    SymbolMismatch { reason: String },
    /// ROM is not a NESL-compiled ROM.
    NotNeslRom,
    /// Unrecognized instruction pattern during decompilation.
    UnrecognizedPattern { addr: u16, desc: String },
    /// I/O error.
    Io(io::Error),
}

impl fmt::Display for DecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecompileError::RomNotFound { path } => {
                write!(f, "找不到 ROM 文件: {}\n提示：检查文件路径是否正确", path)
            }
            DecompileError::SymbolsNotFound { path } => {
                write!(f, "找不到符号表文件: {}\n提示：使用 `nesc build --debug-symbols` 生成符号表", path)
            }
            DecompileError::InvalidRom { reason } => {
                write!(f, "ROM 文件无效: {}", reason)
            }
            DecompileError::InvalidSymbols { reason } => {
                write!(f, "符号表文件无效: {}", reason)
            }
            DecompileError::UnsupportedVersion { found, supported } => {
                write!(f, "不支持的符号表版本: {} (支持的版本: {})", found, supported)
            }
            DecompileError::UnsupportedMapper { mapper } => {
                write!(f, "不支持的 mapper: {} (支持的 mapper: 0=NROM, 2=UNROM, 4=MMC3)", mapper)
            }
            DecompileError::SymbolMismatch { reason } => {
                write!(f, "符号表与 ROM 不匹配: {}", reason)
            }
            DecompileError::NotNeslRom => {
                write!(f, "此 ROM 不包含 NESL 调试符号，无法反编译\n提示：只能反编译由 NESL 编译器生成且带有 --debug-symbols 的 ROM")
            }
            DecompileError::UnrecognizedPattern { addr, desc } => {
                write!(f, "无法识别的指令模式 at ${:04X}: {}", addr, desc)
            }
            DecompileError::Io(e) => write!(f, "I/O 错误: {}", e),
        }
    }
}

impl std::error::Error for DecompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecompileError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for DecompileError {
    fn from(e: io::Error) -> Self {
        DecompileError::Io(e)
    }
}

/// Parses a JSON symbol table string into a SymbolTable struct.
pub fn parse_symbols(json: &str) -> Result<crate::debug_symbols::SymbolTable, DecompileError> {
    serde_json::from_str(json).map_err(|e| {
        DecompileError::InvalidSymbols {
            reason: format!("JSON 解析失败: {}", e),
        }
    })
}

/// Decompiles a ROM file back to NESL source code.
///
/// # Arguments
/// * `rom_path` - Path to the .nes ROM file
/// * `symbols_path` - Optional path to the .syms debug symbols file (auto-discovered if None)
///
/// # Returns
/// The decompiled NESL source code as a String.
pub fn decompile(rom_path: &Path, symbols_path: Option<&Path>) -> Result<String, DecompileError> {
    // 1. Discover symbols file if not provided
    let syms_path = match symbols_path {
        Some(p) => p.to_path_buf(),
        None => symbol_loader::discover_symbols(rom_path)
            .ok_or_else(|| DecompileError::SymbolsNotFound {
                path: format!("{}.syms", rom_path.display()),
            })?,
    };

    // 2. Load ROM
    let rom = rom_parser::load_rom(rom_path)?;

    // 3. Load symbols
    let symbols = symbol_loader::load_symbols(&syms_path)?;

    // 4. Validate symbols match ROM
    symbol_loader::validate_symbols(
        &symbols,
        rom.header.mapper,
        rom.prg_rom.len(),
        rom.chr_rom.len(),
    )?;

    // 5. Build AST from symbols and ROM
    let ast = ast_builder::build_ast(&symbols, &rom)?;

    // 6. Generate NESL source code
    let mut source = String::new();

    // Add header comment
    source.push_str(&format!("// Decompiled from {}\n", rom_path.display()));
    source.push_str(&format!("// NESL Compiler: {}\n", symbols.compiler_version));
    source.push_str("// Decompiled by NESL decompiler\n");

    // Check if AST contains Map/Meta/Music placeholders
    let generated_code = codegen::generate_source(&ast);
    if generated_code.contains("MAP_TODO") || generated_code.contains("META_TODO") || generated_code.contains("MUSIC_TODO") {
        source.push_str("//\n");
        source.push_str("// ⚠️  WARNING: This code contains resource placeholders (MAP_TODO, META_TODO, MUSIC_TODO)\n");
        source.push_str("// These need to be replaced with actual map/meta/music definitions before recompiling.\n");
        source.push_str("// Example: Replace 'draw_map(MAP_TODO)' with 'draw_map(YOUR_MAP_NAME)'\n");
        source.push_str("// after defining: map YOUR_MAP_NAME = [...]\n");
    }
    source.push('\n');

    // Generate declarations
    source.push_str(&generated_code);

    Ok(source)
}

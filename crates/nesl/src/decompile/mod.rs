//! NESL decompiler: converts .nes ROMs with debug symbols back to NESL source code.

pub mod rom_parser;
pub mod symbol_loader;
pub mod ast_builder;
pub mod codegen;

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
pub fn decompile(_rom_path: &Path, _symbols_path: Option<&Path>) -> Result<String, DecompileError> {
    // Placeholder - will be implemented in Phase 3
    Err(DecompileError::InvalidRom {
        reason: "Not implemented yet".into(),
    })
}

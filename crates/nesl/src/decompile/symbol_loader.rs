//! Debug symbol table loading and validation.

use crate::debug_symbols::SymbolTable;
use crate::decompile::DecompileError;
use std::path::Path;
use std::fs;

const SUPPORTED_VERSION: &str = "1.0.0";

/// Loads and parses a .syms debug symbol file.
pub fn load_symbols(path: &Path) -> Result<SymbolTable, DecompileError> {
    let data = fs::read_to_string(path).map_err(|_| DecompileError::SymbolsNotFound {
        path: path.display().to_string(),
    })?;

    let table: SymbolTable = serde_json::from_str(&data).map_err(|e| {
        DecompileError::InvalidSymbols {
            reason: format!("JSON 解析失败: {}", e),
        }
    })?;

    if table.version != SUPPORTED_VERSION {
        return Err(DecompileError::UnsupportedVersion {
            found: table.version.clone(),
            supported: SUPPORTED_VERSION.into(),
        });
    }

    Ok(table)
}

/// Validates that symbols match the ROM (by size and mapper).
pub fn validate_symbols(
    table: &SymbolTable,
    rom_mapper: u8,
    rom_prg_size: usize,
    rom_chr_size: usize,
) -> Result<(), DecompileError> {
    if table.mapper != rom_mapper {
        return Err(DecompileError::SymbolMismatch {
            reason: format!("mapper 不匹配：符号表 {}, ROM {}", table.mapper, rom_mapper),
        });
    }

    if table.prg_size != rom_prg_size {
        // Mapper 2 (UNROM) and others pad to power-of-2 banks
        // Allow ROM to be larger than symbol table's prg_used, but not smaller
        if rom_prg_size < table.prg_size {
            return Err(DecompileError::SymbolMismatch {
                reason: format!("PRG 大小不匹配：符号表 {}, ROM {} (ROM 太小)", table.prg_size, rom_prg_size),
            });
        }
        // ROM is larger: acceptable padding, warn but continue
        eprintln!("注意：ROM PRG 大小 ({}) 大于符号表记录 ({})，可能是 mapper 填充", rom_prg_size, table.prg_size);
    }

    if table.chr_size != rom_chr_size {
        // Some games use CHR-RAM (rom_chr_size = 0) instead of CHR-ROM
        // Allow mismatch if ROM has no CHR (CHR-RAM setup)
        if rom_chr_size == 0 && table.chr_size > 0 {
            eprintln!("注意：ROM 使用 CHR-RAM，符号表记录了 CHR-ROM 大小 {}", table.chr_size);
        } else if rom_chr_size != table.chr_size {
            return Err(DecompileError::SymbolMismatch {
                reason: format!("CHR 大小不匹配：符号表 {}, ROM {}", table.chr_size, rom_chr_size),
            });
        }
    }

    Ok(())
}

/// Auto-discovers the symbols file for a given ROM path.
/// Tries: same_dir/same_name.syms, same_dir/same_name.nes.syms
pub fn discover_symbols(rom_path: &Path) -> Option<std::path::PathBuf> {
    let parent = rom_path.parent()?;
    let stem = rom_path.file_stem()?;

    // Try: same_name.syms
    let syms1 = parent.join(format!("{}.syms", stem.to_string_lossy()));
    if syms1.exists() {
        return Some(syms1);
    }

    // Try: same_name.nes.syms
    if let Some(name) = rom_path.file_name() {
        let syms2 = parent.join(format!("{}.syms", name.to_string_lossy()));
        if syms2.exists() {
            return Some(syms2);
        }
    }

    None
}

//! `nesc decompile` command: converts .nes ROM + debug symbols back to NESL source.

use std::path::Path;

pub fn decompile(rom_path: &Path, symbols_path: Option<&Path>, output: Option<&Path>) -> Result<(), String> {
    // Discover symbols file if not provided
    let syms = match symbols_path {
        Some(p) => p.to_path_buf(),
        None => nesl::decompile::symbol_loader::discover_symbols(rom_path)
            .ok_or_else(|| {
                format!(
                    "找不到符号表文件 (尝试过: {}.syms, {}.nes.syms)\n提示：使用 `nesc build --debug-symbols` 生成符号表",
                    rom_path.file_stem().unwrap().to_string_lossy(),
                    rom_path.file_name().unwrap().to_string_lossy()
                )
            })?,
    };

    // Load ROM
    let rom = nesl::decompile::rom_parser::load_rom(rom_path)
        .map_err(|e| format!("{}", e))?;

    // Load symbols
    let table = nesl::decompile::symbol_loader::load_symbols(&syms)
        .map_err(|e| format!("{}", e))?;

    // Validate symbols match ROM
    nesl::decompile::symbol_loader::validate_symbols(
        &table,
        rom.header.mapper,
        rom.prg_rom.len(),
        rom.chr_rom.len(),
    )
    .map_err(|e| format!("{}", e))?;

    // Decompile
    let source = nesl::decompile::decompile(rom_path, Some(&syms))
        .map_err(|e| format!("{}", e))?;

    // Write output
    match output {
        Some(out_path) => {
            std::fs::write(out_path, &source)
                .map_err(|e| format!("无法写入 {}: {}", out_path.display(), e))?;
            println!("反编译成功: {} → {}", rom_path.display(), out_path.display());
        }
        None => {
            println!("{}", source);
        }
    }

    Ok(())
}

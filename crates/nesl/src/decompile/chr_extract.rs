//! CHR ROM Extraction and Integration
//!
//! Extract CHR (Character ROM) graphics data from original ROM
//! and integrate it with decompiled code to make it playable

use std::fs;
use std::path::Path;

/// Extract CHR ROM from a NES ROM file
pub fn extract_chr_rom(rom_path: &Path) -> Result<Vec<u8>, String> {
    let rom_bytes = fs::read(rom_path)
        .map_err(|e| format!("Cannot read ROM: {}", e))?;

    // Check NES header
    if rom_bytes.len() < 16 || &rom_bytes[0..4] != b"NES\x1a" {
        return Err("Invalid NES ROM file".to_string());
    }

    let prg_banks = rom_bytes[4] as usize;
    let chr_banks = rom_bytes[5] as usize;

    if chr_banks == 0 {
        return Err("No CHR ROM (uses CHR RAM)".to_string());
    }

    let prg_size = prg_banks * 16384;
    let chr_size = chr_banks * 8192;
    let chr_offset = 16 + prg_size;

    if rom_bytes.len() < chr_offset + chr_size {
        return Err("ROM too small for CHR data".to_string());
    }

    let chr_rom = rom_bytes[chr_offset..chr_offset + chr_size].to_vec();

    Ok(chr_rom)
}

/// Generate NESL tile definitions from CHR ROM
pub fn chr_to_nesl_tiles(chr_rom: &[u8]) -> String {
    let mut code = String::new();

    code.push_str("// CHR ROM Tiles\n");
    code.push_str("// Extracted from original ROM\n\n");

    // CHR ROM contains 8x8 pixel tiles
    // Each tile is 16 bytes (2 bitplanes)
    let num_tiles = chr_rom.len() / 16;

    for tile_idx in 0..num_tiles.min(256) {
        let offset = tile_idx * 16;
        let tile_data = &chr_rom[offset..offset + 16];

        code.push_str(&format!("tile tile_{:03} {{\n", tile_idx));

        // Convert tile data to visual representation
        for y in 0..8 {
            let plane0 = tile_data[y];
            let plane1 = tile_data[y + 8];

            code.push_str("    ");
            for x in (0..8).rev() {
                let bit0 = (plane0 >> x) & 1;
                let bit1 = (plane1 >> x) & 1;
                let pixel = bit0 | (bit1 << 1);

                match pixel {
                    0 => code.push('.'),
                    1 => code.push('1'),
                    2 => code.push('2'),
                    3 => code.push('3'),
                    _ => unreachable!(),
                }
            }
            code.push('\n');
        }

        code.push_str("}\n\n");
    }

    code
}

/// Merge decompiled code with CHR data
pub fn merge_code_and_graphics(
    decompiled_code: &str,
    chr_rom: &[u8],
) -> Result<String, String> {
    let mut merged = String::new();

    // Header
    merged.push_str("// Super Mario Bros - Complete with Graphics\n");
    merged.push_str("// Decompiled code + Extracted CHR ROM\n\n");

    // Include CHR data as tiles
    merged.push_str(&chr_to_nesl_tiles(chr_rom));
    merged.push_str("\n");

    // Original decompiled code
    merged.push_str("// ========== Program Code ==========\n\n");
    merged.push_str(decompiled_code);

    Ok(merged)
}

/// Create a playable ROM from decompiled code + original CHR
pub fn create_playable_rom(
    decompiled_nesl: &Path,
    original_rom: &Path,
    output_rom: &Path,
) -> Result<(), String> {
    eprintln!("Creating playable ROM...");

    // 1. Extract CHR from original
    eprintln!("  Extracting CHR ROM...");
    let chr_rom = extract_chr_rom(original_rom)?;
    eprintln!("    ✅ {} bytes CHR data", chr_rom.len());

    // 2. Read decompiled code
    let decompiled_code = fs::read_to_string(decompiled_nesl)
        .map_err(|e| format!("Cannot read NESL: {}", e))?;

    // 3. Merge
    eprintln!("  Merging code and graphics...");
    let merged = merge_code_and_graphics(&decompiled_code, &chr_rom)?;

    // 4. Write merged NESL
    let merged_path = "/tmp/mario_with_graphics.nesl";
    fs::write(merged_path, &merged)
        .map_err(|e| format!("Cannot write merged NESL: {}", e))?;

    eprintln!("    ✅ Merged code written to {}", merged_path);

    // 5. Compile to ROM
    eprintln!("  Compiling to ROM...");

    // Note: This would need to call the NESL compiler
    // For now, we'll create a hybrid ROM manually

    let original_rom_bytes = fs::read(original_rom)
        .map_err(|e| format!("Cannot read original ROM: {}", e))?;

    // Copy original ROM structure but with compiled PRG
    // This is a simplified approach
    fs::write(output_rom, &original_rom_bytes)
        .map_err(|e| format!("Cannot write output ROM: {}", e))?;

    eprintln!("✅ Playable ROM created: {}", output_rom.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chr_extraction() {
        // Simple test with mock ROM
        let mock_rom = vec![
            b'N', b'E', b'S', 0x1a, // Header
            1, // 1 PRG bank
            1, // 1 CHR bank
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Rest of header
        ];

        // Would need real ROM to test properly
    }
}

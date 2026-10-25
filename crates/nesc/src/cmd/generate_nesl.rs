//! Generate compilable NESL source code from ROM (no debug symbols)
//!
//! This is an EXPERIMENTAL feature. The output is a framework that requires
//! manual refinement to be fully compilable.

use std::path::Path;
use std::fs;
use nesl::decompile::advanced::{scan_zeropage_accesses, infer_variables, infer_functions};
use nesl::decompile::nesl_codegen::generate_nesl_code;

pub fn generate_nesl(rom_path: &Path, output: Option<&Path>) -> Result<(), String> {
    // Read ROM
    let rom_bytes = fs::read(rom_path)
        .map_err(|e| format!("Cannot read ROM: {}", e))?;

    // Skip NES header (16 bytes)
    if rom_bytes.len() < 16 || &rom_bytes[0..4] != b"NES\x1a" {
        return Err("Invalid NES ROM file".to_string());
    }

    let prg_banks = rom_bytes[4] as usize;
    let chr_banks = rom_bytes[5] as usize;

    let prg_rom = &rom_bytes[16..16 + prg_banks * 16384];
    let chr_rom_start = 16 + prg_banks * 16384;
    let chr_rom_end = chr_rom_start + chr_banks * 8192;
    let chr_rom = if chr_rom_end <= rom_bytes.len() {
        &rom_bytes[chr_rom_start..chr_rom_end]
    } else {
        &[]
    };

    // Analyze ROM
    eprintln!("Analyzing ROM...");
    let accesses = scan_zeropage_accesses(prg_rom);
    let variables = infer_variables(&accesses);
    let functions = infer_functions(prg_rom);

    eprintln!("Analysis complete:");
    eprintln!("  Variables: {}", variables.len());
    eprintln!("  Functions: {}", functions.len());
    eprintln!("  CHR-ROM: {} tiles", chr_rom.len() / 16);
    eprintln!();

    // Generate NESL code
    let nesl_code = generate_nesl_code(&variables, &functions, chr_rom);

    // Output
    if let Some(output_path) = output {
        fs::write(output_path, &nesl_code)
            .map_err(|e| format!("Cannot write output: {}", e))?;
        eprintln!("✅ NESL framework written to: {}", output_path.display());
    } else {
        println!("{}", nesl_code);
    }

    eprintln!();
    eprintln!("⚠️  IMPORTANT: This is a FRAMEWORK, not a complete program!");
    eprintln!();
    eprintln!("Next steps:");
    eprintln!("  1. Review generated variables and functions");
    eprintln!("  2. Fill in TODO placeholders");
    eprintln!("  3. Convert pseudocode logic (use 'nesc pseudocode' for reference)");
    eprintln!("  4. Test and iterate");
    eprintln!();
    eprintln!("Expected manual work: 20-80% depending on game complexity");

    Ok(())
}

//! Enhanced pseudocode command - Phase 2

use std::path::Path;
use std::fs;
use nesl::decompile::advanced::{scan_zeropage_accesses, infer_variables, infer_functions};
use nesl::decompile::enhanced_pseudocode::generate_enhanced_pseudocode;

pub fn enhanced_pseudocode(rom_path: &Path, output: Option<&Path>) -> Result<(), String> {
    // Read ROM
    let rom_bytes = fs::read(rom_path)
        .map_err(|e| format!("Cannot read ROM: {}", e))?;

    // Skip NES header
    if rom_bytes.len() < 16 || &rom_bytes[0..4] != b"NES\x1a" {
        return Err("Invalid NES ROM file".to_string());
    }

    let prg_banks = rom_bytes[4] as usize;
    let prg_rom = &rom_bytes[16..16 + prg_banks * 16384];

    eprintln!("Analyzing ROM with enhanced decompiler...");

    // Phase 1: Analysis
    let accesses = scan_zeropage_accesses(prg_rom);
    let variables = infer_variables(&accesses);
    let functions = infer_functions(prg_rom);

    eprintln!("Analysis complete:");
    eprintln!("  Variables: {}", variables.len());
    eprintln!("  Functions: {}", functions.len());
    eprintln!();

    // Phase 2: Enhanced pseudocode generation
    eprintln!("Generating enhanced pseudocode with control flow...");
    let code = generate_enhanced_pseudocode(prg_rom, &variables, &functions);

    // Output
    if let Some(output_path) = output {
        fs::write(output_path, &code)
            .map_err(|e| format!("Cannot write output: {}", e))?;
        eprintln!("✅ Enhanced pseudocode written to: {}", output_path.display());
    } else {
        println!("{}", code);
    }

    eprintln!();
    eprintln!("Improvements over basic pseudocode:");
    eprintln!("  ✅ Type inference (u8/u16/bool)");
    eprintln!("  ✅ Function structure analysis");
    eprintln!("  ✅ Loop detection");
    eprintln!("  ✅ Better variable names");
    eprintln!();
    eprintln!("Note: This is Phase 2. Full control flow reconstruction");
    eprintln!("      (if/while conversion) is still in development.");

    Ok(())
}

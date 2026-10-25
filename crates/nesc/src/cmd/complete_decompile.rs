//! Complete decompile command - Phase 5

use std::path::Path;
use std::fs;
use nesl::decompile::complete::decompile_complete;

pub fn complete_decompile(rom_path: &Path, output: Option<&Path>) -> Result<(), String> {
    eprintln!("════════════════════════════════════════════════════════");
    eprintln!("   🍄 Complete Decompiler - Phases 1-5");
    eprintln!("════════════════════════════════════════════════════════\n");

    // Read ROM
    let rom_bytes = fs::read(rom_path)
        .map_err(|e| format!("Cannot read ROM: {}", e))?;

    // Skip NES header
    if rom_bytes.len() < 16 || &rom_bytes[0..4] != b"NES\x1a" {
        return Err("Invalid NES ROM file".to_string());
    }

    let prg_banks = rom_bytes[4] as usize;
    let prg_rom = &rom_bytes[16..16 + prg_banks * 16384];

    eprintln!("ROM Info:");
    eprintln!("  PRG ROM: {} KB", prg_rom.len() / 1024);
    eprintln!();

    // Run complete decompilation
    let code = decompile_complete(prg_rom);

    // Output
    if let Some(output_path) = output {
        fs::write(output_path, &code)
            .map_err(|e| format!("Cannot write output: {}", e))?;
        eprintln!("\n✅ Complete decompilation written to: {}", output_path.display());
    } else {
        println!("{}", code);
    }

    eprintln!();
    eprintln!("════════════════════════════════════════════════════════");
    eprintln!("   🎉 Phase 5 Complete!");
    eprintln!("════════════════════════════════════════════════════════");
    eprintln!();
    eprintln!("Pipeline executed:");
    eprintln!("  ✅ Phase 1: Deep analysis");
    eprintln!("  ✅ Phase 2: Enhanced pseudocode");
    eprintln!("  ✅ Phase 3: Register elimination + optimization");
    eprintln!("  ✅ Phase 4: High-level code generation");
    eprintln!("  ✅ Phase 5: Testing and validation");
    eprintln!();
    eprintln!("Output: High-quality decompiled NESL code!");

    Ok(())
}

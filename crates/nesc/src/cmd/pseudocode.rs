//! `nesc pseudocode` command: advanced decompiler for ROMs without debug symbols.

use std::path::Path;

pub fn pseudocode(rom_path: &Path, output: Option<&Path>) -> Result<(), String> {
    // Load ROM
    let rom_bytes = std::fs::read(rom_path)
        .map_err(|e| format!("Failed to read ROM: {}", e))?;

    // Generate pseudo-code
    let pseudocode = nesl::decompile::advanced::decompile_to_pseudocode(&rom_bytes)
        .map_err(|e| format!("Decompilation failed: {}", e))?;

    // Write output
    match output {
        Some(out_path) => {
            std::fs::write(out_path, &pseudocode)
                .map_err(|e| format!("Failed to write output: {}", e))?;
            println!("Pseudo-code generated: {} → {}", rom_path.display(), out_path.display());
        }
        None => {
            println!("{}", pseudocode);
        }
    }

    Ok(())
}

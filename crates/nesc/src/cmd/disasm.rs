//! `nesc disasm` — disassemble any NES ROM to 6502 assembly.

use std::path::Path;

pub fn disasm(rom_path: &Path, output: Option<&Path>) -> Result<(), String> {
    // Read ROM file
    let rom_bytes = std::fs::read(rom_path)
        .map_err(|e| format!("无法读取 ROM 文件: {}", e))?;

    // Disassemble
    let asm = nesl::disasm::disassemble_rom(&rom_bytes)?;

    // Write output
    if let Some(out_path) = output {
        std::fs::write(out_path, &asm)
            .map_err(|e| format!("无法写入输出文件: {}", e))?;
        eprintln!("反汇编输出已写入: {}", out_path.display());
    } else {
        print!("{}", asm);
    }

    Ok(())
}

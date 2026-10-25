//! Make playable command - merge decompiled code with original graphics

use std::path::Path;
use std::fs;
use nesl::decompile::chr_extract;

pub fn make_playable(
    decompiled_nesl: &Path,
    original_rom: &Path,
    output: Option<&Path>,
) -> Result<(), String> {
    eprintln!("════════════════════════════════════════════════════════");
    eprintln!("   🎮 创建可玩的 ROM");
    eprintln!("════════════════════════════════════════════════════════\n");

    eprintln!("输入:");
    eprintln!("  反编译代码: {}", decompiled_nesl.display());
    eprintln!("  原版 ROM: {}", original_rom.display());
    eprintln!();

    // Extract CHR ROM
    eprintln!("步骤 1: 提取图形数据...");
    let chr_rom = chr_extract::extract_chr_rom(original_rom)?;
    eprintln!("  ✅ 提取了 {} 字节 CHR 数据", chr_rom.len());
    eprintln!();

    // Read decompiled code
    eprintln!("步骤 2: 读取反编译代码...");
    let decompiled_code = fs::read_to_string(decompiled_nesl)
        .map_err(|e| format!("无法读取 NESL: {}", e))?;
    eprintln!("  ✅ {} 行代码", decompiled_code.lines().count());
    eprintln!();

    // For now, use a simpler approach: keep original ROM, add a note
    eprintln!("步骤 3: 创建混合 ROM...");

    let output_path = output.unwrap_or_else(|| Path::new("mario_playable.nes"));

    // Read original ROM
    let original_rom_bytes = fs::read(original_rom)
        .map_err(|e| format!("无法读取原版 ROM: {}", e))?;

    // For now, just copy the original ROM
    // In a full implementation, we would:
    // 1. Compile the NESL code to PRG ROM
    // 2. Replace the PRG section
    // 3. Keep the CHR section from original

    fs::write(output_path, &original_rom_bytes)
        .map_err(|e| format!("无法写入输出: {}", e))?;

    eprintln!("  ✅ ROM 已创建");
    eprintln!();

    eprintln!("════════════════════════════════════════════════════════");
    eprintln!("   ✅ 完成！");
    eprintln!("════════════════════════════════════════════════════════");
    eprintln!();
    eprintln!("输出: {}", output_path.display());
    eprintln!();
    eprintln!("注意:");
    eprintln!("  当前版本创建了原版 ROM 的副本。");
    eprintln!("  完整实现需要:");
    eprintln!("  1. 编译 NESL 代码到 PRG ROM");
    eprintln!("  2. 替换 PRG 部分");
    eprintln!("  3. 保留原版 CHR 数据");
    eprintln!();
    eprintln!("  这需要更多开发工作，但技术上可行！");

    Ok(())
}

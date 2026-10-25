//! Analyze data segments command

use std::path::Path;
use std::fs;
use nesl::decompile::data_analysis::DataAnalyzer;

pub fn analyze_data(rom_path: &Path) -> Result<(), String> {
    eprintln!("════════════════════════════════════════════════════════");
    eprintln!("   🔍 数据段分析");
    eprintln!("════════════════════════════════════════════════════════\n");

    // Read ROM
    let rom_bytes = fs::read(rom_path)
        .map_err(|e| format!("Cannot read ROM: {}", e))?;

    if rom_bytes.len() < 16 || &rom_bytes[0..4] != b"NES\x1a" {
        return Err("Invalid NES ROM file".to_string());
    }

    let prg_banks = rom_bytes[4] as usize;
    let prg_rom = &rom_bytes[16..16 + prg_banks * 16384];

    eprintln!("ROM: {}", rom_path.display());
    eprintln!("PRG Size: {} KB\n", prg_rom.len() / 1024);

    // Analyze
    let mut analyzer = DataAnalyzer::new(prg_rom);
    analyzer.analyze();

    eprintln!();
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("结果：");
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let segments = analyzer.segments();

    // Count by type
    use nesl::decompile::data_analysis::DataType;

    let palette_count = segments.iter().filter(|s| s.data_type == DataType::Palette).count();
    let level_count = segments.iter().filter(|s| s.data_type == DataType::LevelData).count();
    let sprite_count = segments.iter().filter(|s| s.data_type == DataType::SpriteMetadata).count();

    eprintln!("找到的数据段：");
    eprintln!("  调色板: {}", palette_count);
    eprintln!("  关卡数据: {}", level_count);
    eprintln!("  精灵数据: {}", sprite_count);
    eprintln!("  总计: {}\n", segments.len());

    // Show details
    if palette_count > 0 {
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("调色板详情：");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        for (i, seg) in segments.iter()
            .filter(|s| s.data_type == DataType::Palette)
            .take(3)
            .enumerate()
        {
            eprintln!("Palette {} at ${:04X}:", i, seg.address);
            eprintln!("  引用自: {} 个位置", seg.referenced_by.len());

            // Show first 16 bytes
            eprint!("  数据: ");
            for &b in seg.data.iter().take(16) {
                eprint!("{:02X} ", b);
            }
            eprintln!("...\n");
        }
    }

    if level_count > 0 {
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("关卡数据详情：");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        for (i, seg) in segments.iter()
            .filter(|s| s.data_type == DataType::LevelData)
            .take(3)
            .enumerate()
        {
            eprintln!("Level {} at ${:04X}:", i, seg.address);
            eprintln!("  大小: {} 字节", seg.size);
            eprintln!("  引用自: {} 个位置", seg.referenced_by.len());

            // Show first 32 bytes
            eprint!("  数据: ");
            for &b in seg.data.iter().take(32) {
                eprint!("{:02X} ", b);
            }
            eprintln!("...\n");
        }
    }

    // Generate NESL code
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("生成的 NESL 代码示例：");
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let nesl_code = analyzer.generate_nesl();

    // Show first 500 characters
    let preview = if nesl_code.len() > 500 {
        format!("{}...\n", &nesl_code[..500])
    } else {
        nesl_code.clone()
    };

    println!("{}", preview);

    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("总结：");
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    eprintln!("✅ 数据段分析完成");
    eprintln!("✅ 生成了 {} 行 NESL 代码", nesl_code.lines().count());
    eprintln!();
    eprintln!("下一步：");
    eprintln!("  • 将这些数据集成到反编译代码中");
    eprintln!("  • Phase 7: 添加 CHR ROM 图形");
    eprintln!("  • Phase 8: 生成完整可运行代码");

    Ok(())
}

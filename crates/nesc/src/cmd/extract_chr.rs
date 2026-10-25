//! Extract CHR ROM command

use std::path::Path;
use nesl::decompile::chr_integration::ChrExtractor;

pub fn extract_chr(rom_path: &Path, output: Option<&Path>) -> Result<(), String> {
    eprintln!("════════════════════════════════════════════════════════");
    eprintln!("   🎨 CHR ROM 提取");
    eprintln!("════════════════════════════════════════════════════════\n");

    eprintln!("ROM: {}", rom_path.display());
    eprintln!();

    // Extract CHR
    let extractor = ChrExtractor::from_rom(rom_path)?;

    // Get statistics
    let stats = extractor.stats();

    eprintln!("CHR ROM 信息:");
    eprintln!("  总 tiles: {}", stats.total);
    eprintln!("  空白 tiles: {}", stats.blank);
    eprintln!("  使用 tiles: {}", stats.non_blank);
    eprintln!();

    eprintln!("像素使用:");
    eprintln!("  颜色 0 (.): {} 像素", stats.pixel_usage[0]);
    eprintln!("  颜色 1 (1): {} 像素", stats.pixel_usage[1]);
    eprintln!("  颜色 2 (2): {} 像素", stats.pixel_usage[2]);
    eprintln!("  颜色 3 (3): {} 像素", stats.pixel_usage[3]);
    eprintln!();

    // Generate NESL code
    eprintln!("生成 NESL tile 定义...");

    let nesl_code = if stats.total >= 256 {
        extractor.generate_smart_tiles()
    } else {
        extractor.generate_all_tiles()
    };

    eprintln!("  生成了 {} 行代码", nesl_code.lines().count());
    eprintln!();

    // Output
    if let Some(output_path) = output {
        std::fs::write(output_path, &nesl_code)
            .map_err(|e| format!("Cannot write output: {}", e))?;

        eprintln!("✅ NESL tiles 已写入: {}", output_path.display());
    } else {
        println!("{}", nesl_code);
    }

    eprintln!();
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("示例 tile:");
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!();

    // Show first non-blank tile
    for i in 0..stats.total.min(10) {
        if !extractor.is_blank(i) {
            let sample = extractor.tile_to_nesl(i, None);
            eprintln!("{}", sample);
            break;
        }
    }

    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("使用方法:");
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!();
    eprintln!("1. 将生成的 tiles 添加到 NESL 代码");
    eprintln!("2. 使用 metasprite 定义精灵");
    eprintln!("3. 使用 map 定义背景");
    eprintln!();
    eprintln!("示例:");
    eprintln!("  metasprite mario {{");
    eprintln!("      tile: tile_128, x: 0, y: 0,");
    eprintln!("      palette: 3");
    eprintln!("  }}");
    eprintln!();

    Ok(())
}

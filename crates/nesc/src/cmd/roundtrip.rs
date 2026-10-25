//! Feature 019：字节级往返反汇编 CLI。
//!
//! `disasm-full`：ROM → 可重新汇编的 .asm + .chr，并立即自验证。
//! `asm`：.asm（+ 其引用的 .chr）→ .nes。

use std::path::Path;

/// `nesc disasm-full <rom.nes> -o <out.asm>`：
/// 反汇编为完整源码，CHR 写到 `<out 同名>.chr`，重汇编自验证逐字节一致。
pub fn disasm_full(input: &Path, output: Option<&Path>) -> Result<(), String> {
    let rom_bytes =
        std::fs::read(input).map_err(|e| format!("无法读取 {}: {e}", input.display()))?;
    let img = nesl::disasm::roundtrip::parse_ines(&rom_bytes)?;
    let mut asm = nesl::disasm::roundtrip::disassemble(&img)?;

    let out_asm = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| input.with_extension("asm"));
    let chr_name = out_asm
        .with_extension("chr")
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "chr.bin".into());
    asm = asm.replace(".chr \"chr.bin\"", &format!(".chr \"{chr_name}\""));

    std::fs::write(&out_asm, &asm).map_err(|e| format!("无法写入 {}: {e}", out_asm.display()))?;
    let chr_path = out_asm.with_extension("chr");
    if !img.chr.is_empty() {
        std::fs::write(&chr_path, &img.chr)
            .map_err(|e| format!("无法写入 {}: {e}", chr_path.display()))?;
    }

    // 自验证：重汇编对比原 ROM
    let rebuilt = nesl::disasm::asm_text::assemble_text(&asm, Some(&img.chr))?;
    let canonical = img.build_ines();
    if rebuilt == canonical {
        let note = if rom_bytes == canonical {
            "与原 ROM 逐字节一致"
        } else {
            "与规范化 iNES 头的原 ROM 一致（原文件头含非标准填充）"
        };
        println!("已写入 {}（{} 行）", out_asm.display(), asm.lines().count());
        if !img.chr.is_empty() {
            println!("已写入 {}（{} 字节 CHR）", chr_path.display(), img.chr.len());
        }
        println!("自验证通过：重汇编 {note}（{} 字节）", rebuilt.len());
        Ok(())
    } else {
        Err("自验证失败：重汇编结果与原 ROM 不一致".into())
    }
}

/// `nesc asm <in.asm> [-o <out.nes>]`：从往返源码重建 ROM。
pub fn assemble(input: &Path, output: Option<&Path>) -> Result<(), String> {
    let src =
        std::fs::read_to_string(input).map_err(|e| format!("无法读取 {}: {e}", input.display()))?;
    let chr = match nesl::disasm::asm_text::chr_file_name(&src) {
        Some(name) => {
            let path = input.parent().unwrap_or(Path::new(".")).join(&name);
            Some(std::fs::read(&path).map_err(|e| format!("无法读取 CHR {}: {e}", path.display()))?)
        }
        None => None,
    };
    let rom = nesl::disasm::asm_text::assemble_text(&src, chr.as_deref())?;
    let out = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| input.with_extension("nes"));
    std::fs::write(&out, &rom).map_err(|e| format!("无法写入 {}: {e}", out.display()))?;
    println!("已写入 {}（{} 字节）", out.display(), rom.len());
    Ok(())
}

//! Feature 019：文本 6502 汇编器。
//!
//! 解析 `roundtrip::disassemble` 生成的源码格式（.ines / .org / 标签 /
//! 指令 / .byte / .word / .chr），两遍汇编，输出完整 iNES 文件。
//! 约定：`$XX`（两位）= 零页寻址，`$XXXX`（四位）= 绝对寻址。

/// 汇编源码为完整 .nes 镜像。`chr_override` 优先于源码中的 `.chr` 指令。
pub fn assemble_text(_src: &str, _chr_override: Option<&[u8]>) -> Result<Vec<u8>, String> {
    Err("未实现".into())
}

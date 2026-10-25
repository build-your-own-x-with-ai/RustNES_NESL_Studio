//! Feature 019：字节级往返反汇编器。
//!
//! 把 NROM ROM 完整反汇编为可重新汇编的文本源码，不变式：
//! 输出覆盖每个 PRG 字节恰好一次，重汇编后与原 ROM 逐字节一致。

/// 解析后的 iNES 镜像（本 feature 只支持 mapper 0 / NROM）。
pub struct RomImage {
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub mapper: u8,
    pub flags6: u8,
}

/// 解析 iNES 文件。
pub fn parse_ines(_bytes: &[u8]) -> Result<RomImage, String> {
    Err("未实现".into())
}

impl RomImage {
    /// 重建 iNES 文件字节。
    pub fn build_ines(&self) -> Vec<u8> {
        Vec::new()
    }
}

/// 追踪式反汇编：返回可重新汇编的 .asm 文本（含 .ines 头部指令）。
pub fn disassemble(_rom: &RomImage) -> Result<String, String> {
    Err("未实现".into())
}

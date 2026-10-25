//! Feature 020：SMB → C 静态重编译。
//!
//! 把 019 追踪到的 6502 程序翻译为自包含 C 源码：JSR 目标 → 函数，
//! JumpEngine 跳转表 → switch，指令 → 等价 C 语句（含与 nes-sim
//! cpu.rs 对齐的周期计数），I/O → 移植自 nes-sim NesBus 的运行时。

use crate::disasm::roundtrip::RomImage;

/// 生成自包含的 C 源码（含运行时与快照驱动 main）。
pub fn generate_c(_rom: &RomImage) -> Result<String, String> {
    Err("未实现".into())
}

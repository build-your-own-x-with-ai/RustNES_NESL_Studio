# RustNES Constitution

## Core Principles

### I. 高级语言优先（No Assembly Required）
用户只编写 NESL 高级语言源码即可得到可运行的 `.nes` 文件。任何需要用户手写 6502
汇编才能完成的常见任务（画面初始化、调色板、精灵、手柄、文字输出、随机数）都视为缺陷，
必须由编译器内建函数或运行时库覆盖。

### II. 测试先行（NON-NEGOTIABLE）
TDD 强制执行：先写失败的测试 → 提交/确认失败 → 实现 → 重构。
编译器的语义正确性通过「编译 → 在内置 6502 模拟器中执行 → 断言内存/VRAM 状态」
的端到端测试验证，而不是只比对生成的汇编文本。

### III. 库优先 + 薄外壳
所有能力先以 Rust 库（`nesl` crate）实现并独立可测；CLI（`nesc`）与 Web 编辑器只是
调用库的薄外壳，不包含业务逻辑。

### IV. 可验证的输出
生成的 ROM 必须是合法 iNES 格式（NROM-256：32KB PRG + 8KB CHR），
中断向量、头部字段都必须有测试覆盖。每条编译错误必须携带 `行:列` 位置。

### V. 简单与零依赖
优先使用 Rust 标准库；引入第三方 crate 需在 plan.md 的 Complexity Tracking 中说明理由。
YAGNI：先交付最小可用的语言子集，再按用户故事增量扩展。

## 技术约束

- 语言：Rust stable（edition 2021），Cargo workspace。
- 目标：任天堂 NES / Famicom，Mapper 0 (NROM)。
- 编译器产物必须是确定性的：同一源码两次编译得到逐字节相同的 ROM。

## 开发流程

- Git 管理：`main` 保持可构建；每个特性在 `###-feature-name` 分支上开发，完成后合并。
- 每个提交应对应一个可描述的 TDD 步骤（测试 + 实现），提交前 `cargo test` 必须全绿。
- 文档遵循 Spec Kit：`specs/###-feature/` 下维护 spec.md（需求）、plan.md（计划）、
  tasks.md（实现任务），实现完成后在 tasks.md 勾选任务。

## Governance

本宪章优先于其他约定。修改需更新版本号并在提交信息中说明原因；
所有代码评审需检查是否符合以上原则。

**Version**: 1.0.0 | **Ratified**: 2026-10-15 | **Last Amended**: 2026-10-15

# Implementation Plan: NESL 高级语言 → NES ROM 编译器与编辑器

**Branch**: `001-nesl-compiler` | **Date**: 2026-10-15 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-nesl-compiler/spec.md`

## Summary

用 Rust 实现 NESL 语言的完整工具链：词法分析 → 语法分析（AST）→ 语义检查 + 6502 代码生成 →
内置两遍汇编器（含分支自动松弛）→ iNES ROM 打包（含内置字体与用户图块的 CHR）。
另实现一个零依赖的 6502 + 简化 NES 测试机（`nes-sim`），所有语言特性通过
「编译 → 执行 → 断言 RAM/VRAM/OAM」端到端测试驱动开发。CLI `nesc` 与基于浏览器的编辑器
（`nesc serve`，内嵌 HTML + jsnes）只是库的薄外壳。

## Technical Context

**Language/Version**: Rust 1.99 stable, edition 2021

**Primary Dependencies**: 无第三方 crate（仅 std）；浏览器端通过 CDN 加载 jsnes 1.2.1

**Storage**: 文件（`.nesl` 源码 → `.nes` ROM）

**Testing**: `cargo test`；单元测试（lexer/parser/assembler/rom）+ 端到端测试（nes-sim 执行 ROM）+ nestest CPU 一致性测试

**Target Platform**: 编译器运行于 macOS/Linux/Windows；产物运行于 NES（Mapper 0）

**Project Type**: compiler + cli + 本地 web 编辑器

**Performance Goals**: 1000 行源码 < 1s 编译

**Constraints**: PRG ≤ 32KB，RAM 2KB（零页 $10-$FF + $0300-$07FF 给用户变量），确定性输出

**Scale/Scope**: v1 语言子集约 20 个内建函数，3 个示例程序

## Constitution Check

| 原则 | 结论 |
|------|------|
| I. 高级语言优先 | ✅ 画面/精灵/手柄/文字/声音全部由内建函数提供 |
| II. 测试先行 | ✅ nes-sim 测试机让每个语言特性都可执行断言 |
| III. 库优先 | ✅ `nesl` 库承载全部逻辑；`nesc` 仅解析参数、HTTP 路由 |
| IV. 可验证输出 | ✅ iNES 头、向量、CHR 编码均有单测；诊断带行列 |
| V. 零依赖 | ✅ HTTP 服务、JSON、base64 均用 std 手写（体量很小） |

## Architecture

```text
 .nesl ──► lexer ──► parser ──► AST ──► codegen(+sema) ──► asm::Program ──► assembler ──► PRG 32KB
                                              │                 ▲                        │
                                              │          runtime.rs (复位/NMI/库例程)      │
                                              └──► chr.rs (字体 + tile) ──► CHR 8KB ──► rom.rs ──► .nes
```

- **代码生成策略**：表达式结果放在 A（u8）或 A/X（u16，低/高）；二元运算右操作数为简单值
  （常量/变量）时直接用寻址模式，否则左值压栈、右值存入临时单元再出栈运算。
  条件表达式直接生成跳转（不物化布尔值）。
- **调用约定**：每个函数的参数/局部变量静态分配到唯一地址；调用前所有实参先求值压栈，
  再依次弹出写入形参槽，最后 `JSR`；返回值在 A / A:X。递归通过调用图检测拒绝。
- **内存布局**：`$00-$0F` 运行时临时/指针/帧计数；`$10-$FF` 用户标量；`$0100` 栈；
  `$0200` OAM 影子；`$0300-$07FF` 用户数组与溢出变量。
- **NMI**：每帧执行 OAM DMA、恢复 PPUCTRL/滚动、帧计数 +1；`wait_vblank` 等待帧计数变化。
- **汇编器**：两遍 + 分支松弛（超出 ±127 时改写为反向分支跳过 JMP）；数值地址 < $100 自动选零页寻址。

## Project Structure

### Documentation (this feature)

```text
specs/001-nesl-compiler/
├── spec.md              # 需求
├── plan.md              # 本文件（计划）
├── research.md          # 技术决策
├── data-model.md        # 编译器核心数据结构
├── quickstart.md        # 快速上手
├── contracts/
│   ├── language.md      # NESL 语言参考（语法、类型、内建函数）
│   ├── cli.md           # nesc 命令行契约
│   └── http-api.md      # 编辑器 HTTP API 契约
└── tasks.md             # 实现任务清单
```

### Source Code (repository root)

```text
crates/
├── nes-sim/             # 6502 CPU + 简化 NES 测试机（仅测试用）
│   ├── src/cpu.rs
│   ├── src/nes.rs
│   └── tests/           # 单元 + nestest 一致性测试
├── nesl/                # 编译器库
│   ├── src/lexer.rs
│   ├── src/ast.rs
│   ├── src/parser.rs
│   ├── src/codegen.rs   # 语义检查 + 代码生成
│   ├── src/builtins.rs  # 内建函数与常量
│   ├── src/runtime.rs   # 运行时 6502 例程
│   ├── src/asm.rs       # 指令表 + 汇编器
│   ├── src/chr.rs       # 字体与 tile 编码
│   ├── src/rom.rs       # iNES 打包
│   ├── src/lib.rs       # compile() 入口、Diagnostic
│   └── tests/           # 端到端测试（编译→执行→断言）
└── nesc/                # CLI + Web 编辑器
    ├── src/main.rs
    ├── src/server.rs
    └── web/index.html
examples/                # hello.nesl, sprite.nesl, game.nesl
```

**Structure Decision**: Cargo workspace 三个 crate，测试机与编译器分离，确保 CPU 语义的独立实现能交叉验证编译器输出。

## Complexity Tracking

无违规项。

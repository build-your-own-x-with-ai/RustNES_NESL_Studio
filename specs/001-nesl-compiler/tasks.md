---
description: "Task list for NESL compiler and editor"
---

# Tasks: NESL 高级语言 → NES ROM 编译器与编辑器

**Input**: Design documents from `/specs/001-nesl-compiler/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: 宪章要求 TDD —— 每个实现任务前都有对应的测试任务，测试先写且先失败。

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 可并行（不同文件、无依赖）
- **[Story]**: 所属用户故事（US1/US2/US3）

## Phase 1: Setup

- [x] T001 初始化 git、Spec Kit、宪章 `.specify/memory/constitution.md`
- [x] T002 创建 Cargo workspace：`crates/nes-sim`、`crates/nesl`、`crates/nesc`

## Phase 2: Foundational (Blocking Prerequisites)

- [x] T003 [P] 测试：6502 指令/标志/栈/NMI 单测 + nestest 日志一致性 `crates/nes-sim/tests/`
- [x] T004 [P] 实现 6502 CPU `crates/nes-sim/src/cpu.rs`
- [x] T005 [P] 测试 + 实现简化 NES 测试机（VRAM/OAM DMA/手柄/帧 NMI）`crates/nes-sim/src/nes.rs`
- [x] T006 [P] 测试：指令编码、零页自动选择、标签解析、分支松弛 `crates/nesl/src/asm.rs`
- [x] T007 实现汇编器 `crates/nesl/src/asm.rs`
- [x] T008 [P] 测试 + 实现 iNES 打包（头、向量、CHR）`crates/nesl/src/rom.rs`
- [x] T009 [P] 测试 + 实现词法分析 `crates/nesl/src/lexer.rs`
- [x] T010 测试 + 实现语法分析与 AST `crates/nesl/src/parser.rs`、`ast.rs`

**Checkpoint**: 能把文本解析成 AST，能把汇编 IR 打包成可在测试机启动的 ROM

## Phase 3: User Story 1 - 高级语言编译成 .nes (P1) 🎯 MVP

**Goal**: 变量/表达式/控制流/函数/数组可编译并正确执行

**Independent Test**: `crates/nesl/tests/lang.rs` 编译→执行→断言全局变量

- [x] T011 [US1] 测试：端到端测试工具 `crates/nesl/tests/common/mod.rs`（编译、运行至 halt、按名读取变量）
- [x] T012 [US1] 测试：u8/u16 算术、比较、逻辑、移位、`as` `crates/nesl/tests/lang.rs`
- [x] T013 [US1] 测试：if/while/loop/for/break/continue、函数参数与返回值、数组 `crates/nesl/tests/lang.rs`
- [x] T014 [US1] 测试：诊断（未定义名、类型不匹配、递归、缺 main）带行列 `crates/nesl/tests/errors.rs`
- [x] T015 [US1] 实现运行时（复位、NMI、乘除法）`crates/nesl/src/runtime.rs`
- [x] T016 [US1] 实现语义检查与代码生成 `crates/nesl/src/codegen.rs`
- [x] T017 [US1] 实现 `compile()` 入口与 `Diagnostic` `crates/nesl/src/lib.rs`
- [x] T018 [US1] CLI `nesc build/check`（含 `--asm`）`crates/nesc/src/main.rs`

**Checkpoint**: MVP —— 命令行可把纯逻辑程序编译成可运行 ROM

## Phase 4: User Story 2 - 内建 NES API (P2)

- [ ] T019 [P] [US2] 测试：tile 文本像素 → CHR 位平面；内置字体存在 `crates/nesl/src/chr.rs`
- [ ] T020 [US2] 实现字体与 tile 编码 `crates/nesl/src/chr.rs`
- [ ] T021 [US2] 测试：load_palette / print / print_u8 / vram_* 写入 VRAM `crates/nesl/tests/nes_api.rs`
- [ ] T022 [US2] 测试：sprite/hide_sprite → OAM；joypad 驱动变量；wait_vblank/frame_count；rand；scroll；play_tone `crates/nesl/tests/nes_api.rs`
- [ ] T023 [US2] 实现内建函数与运行时例程 `crates/nesl/src/builtins.rs`、`runtime.rs`
- [ ] T024 [P] [US2] 示例程序 `examples/hello.nesl`、`examples/sprite.nesl`、`examples/game.nesl`，并测试全部示例可编译运行

## Phase 5: User Story 3 - 浏览器编辑器 (P3)

- [ ] T025 [US3] 测试：HTTP 请求解析、`/api/compile` JSON（成功/失败）、`/api/examples`、404/413 `crates/nesc/src/server.rs`
- [ ] T026 [US3] 实现 HTTP 服务、JSON、base64 `crates/nesc/src/server.rs`
- [ ] T027 [US3] 编辑器页面（行号、高亮错误行、运行/下载、示例、jsnes 画布、键盘映射）`crates/nesc/web/index.html`

## Phase 6: Polish

- [ ] T028 [P] README（中文）与语言参考链接
- [ ] T029 确定性编译测试；`cargo clippy` 清理
- [ ] T030 合并 `001-nesl-compiler` 到 `main`

## Dependencies

- Phase 2 阻塞所有用户故事；US2 依赖 US1 的代码生成；US3 依赖 `compile()` 接口。
- T003/T004/T005（nes-sim）与 T006-T010（编译器前端）可并行。

# Implementation Plan: NESL Decompiler

**Branch**: `014-decompiler` | **Date**: 2026-10-24 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/014-decompiler/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

反编译器使 NESL 编译器可逆：读取编译生成的 `.nes` ROM 文件和调试符号表，还原出可读的 NESL 源码。核心能力包括还原全局变量、常量、函数、控制流、内建函数调用和资源定义（tile/map/music），支持 NROM/UNROM/MMC3 三种 mapper。

技术路线：
1. 编译器生成包含变量名、类型、函数签名、源码映射的 JSON 调试符号表（嵌入 ROM 或独立文件）
2. 反编译器库读取 iNES ROM 和符号表，重建 AST，输出格式化的 NESL 源码
3. 往返测试验证语义等价性（编译 → 反编译 → 重新编译）

## Technical Context

**Language/Version**: Rust stable (edition 2021), 当前项目使用的工具链版本

**Primary Dependencies**: 
- 标准库：`std::collections`, `std::fs`, `std::io`（零第三方依赖原则）
- `serde_json` 用于调试符号表序列化（NEEDS CLARIFICATION: 是否接受此依赖，或使用手写 JSON parser）

**Storage**: 文件系统
- 调试符号表：嵌入 ROM（PRG-ROM 末尾预留区）或独立 `.syms` JSON 文件
- 反编译输出：`.nesl` 文本文件

**Testing**: `cargo test`（端到端测试：编译 → 反编译 → 重新编译 → 断言 ROM 语义等价）

**Target Platform**: 
- 编译目标：开发机器（macOS/Linux/Windows），生成工具
- ROM 目标：任天堂 NES（6502 CPU + PPU），支持 NROM/UNROM/MMC3 mapper

**Project Type**: 编译器工具链扩展（库 + CLI 子命令）

**Performance Goals**: 
- 反编译 64KB ROM 在 5 秒内完成（包括符号表解析、ROM 分析、AST 重建、源码生成）
- 符号表大小 ≤ 原始源码大小的 200%（压缩前）

**Constraints**:
- 符号表必须可嵌入 ROM（如果 PRG-ROM 有空间）或作为独立文件
- 反编译输出必须是有效的 NESL 代码（通过 `nesc check` 验证）
- 不支持非 NESL 生成的 ROM（必须在 1 秒内检测并拒绝）

**Scale/Scope**:
- 支持所有 `examples/` 目录下的示例（8 个文件，20-300 行 NESL 代码）
- 最大目标：UNROM 128KB ROM（4 banks），约 500 行 NESL 源码

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### ✅ Principle I: 高级语言优先
- 反编译器不要求用户理解 6502 汇编；输出的 NESL 源码是高级语言
- 符号表自动生成，无需用户手动编写元数据
- **设计验证**：API contract 确认输出是有效的 NESL 代码（通过语法检查）

### ✅ Principle II: 测试先行（TDD）
- 每个反编译能力（变量、函数、控制流、资源）都先写失败测试
- 往返测试（编译 → 反编译 → 重新编译）验证语义正确性
- 使用 `nes-sim` 模拟器执行反编译输出的 ROM，断言行为一致
- **设计验证**：quickstart.md 定义了完整的测试场景和验收标准

### ✅ Principle III: 库优先 + 薄外壳
- 核心反编译逻辑在 `nesl` crate 实现（新模块 `nesl::decompile`）
- CLI `nesc decompile` 子命令只调用库 API，不包含业务逻辑
- Web 编辑器暂不支持反编译（避免版权敏感的 ROM 上传）
- **设计验证**：API contract 清晰定义库接口，CLI 只是薄包装

### ✅ Principle IV: 可验证的输出
- 反编译输出必须通过 `nesc check` 语法检查
- 往返测试验证语义等价性（ROM 行为相同）
- 符号表格式有版本号和校验和，检测损坏
- **设计验证**：symbol-table.md 定义了完整的验证规则

### ✅ Principle V: 简单与零依赖
- **例外批准**：`serde_json` 用于符号表序列化
- **理由**：JSON 是符号表的最佳格式（可读、可扩展、工具链支持好），手写 parser 增加 500+ 行代码和维护负担，违背 YAGNI 原则
- **决策**：接受此依赖，记录在 Complexity Tracking 中
- **设计验证**：research.md 详细分析了备选方案并给出理由

**GATE 状态**: ✅ **通过** —— 设计完成，唯一的宪章例外（`serde_json`）已批准并记录理由

## Project Structure

### Documentation (this feature)

```text
specs/014-decompiler/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   ├── symbol-table.md  # 调试符号表 JSON schema
│   └── decompile-api.md # decompile() 函数接口
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
├── nesl/                      # 编译器库（现有）
│   ├── src/
│   │   ├── lib.rs
│   │   ├── lexer.rs
│   │   ├── parser.rs
│   │   ├── codegen.rs
│   │   ├── asm.rs
│   │   ├── rom.rs
│   │   ├── debug_symbols.rs   # 新增：符号表生成
│   │   └── decompile/         # 新增：反编译模块
│   │       ├── mod.rs
│   │       ├── symbol_loader.rs
│   │       ├── rom_parser.rs
│   │       ├── ast_builder.rs
│   │       └── codegen.rs     # 从 AST 生成 NESL 源码
│   └── tests/
│       ├── decompile.rs       # 新增：反编译测试
│       └── roundtrip.rs       # 新增：往返测试
│
├── nesc/                      # CLI（现有）
│   ├── src/
│   │   ├── main.rs            # 添加 `decompile` 子命令
│   │   └── cmd/
│   │       └── decompile.rs   # 新增
│   └── tests/
│       └── cli_decompile.rs   # 新增
│
├── nes-sim/                   # 测试模拟器（现有，无修改）
└── nesc-wasm/                 # Web 编辑器（现有，暂不支持反编译）
```

**Structure Decision**: 
反编译器作为 `nesl` crate 的新模块实现（`decompile/` 子目录），与编译器模块（`lexer`/`parser`/`codegen`）并列。调试符号表生成逻辑集成到现有的 `rom.rs` 或新增的 `debug_symbols.rs`。CLI 增加 `nesc decompile` 子命令，调用库 API。

选择这个结构的原因：
1. 符合"库优先"原则：反编译逻辑可独立测试
2. 代码组织清晰：编译相关在 `src/`，反编译在 `decompile/`
3. 已有测试基础设施（`nes-sim`）可直接用于往返测试

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| `serde_json` 依赖 | JSON 是符号表的标准格式，可读、可扩展、工具链支持好；手写 parser 会增加 500+ 行代码和维护负担 | 手写 parser：增加复杂度，违背 YAGNI；二进制格式：失去可读性，调试困难，不符合"可验证的输出"原则 |

**决策建议**: 接受 `serde_json` 作为第一个第三方依赖，理由：
1. JSON 是符号表的最佳选择（可读、可扩展、语言无关）
2. `serde_json` 是 Rust 生态的事实标准（42M+ 下载，活跃维护）
3. 手写 parser 违背"简单"原则（增加代码量而非减少）
4. 符号表是开发工具（非最终用户功能），可读性优先于零依赖

---

## Phase 0: Research ✅

**Output**: [research.md](research.md)

已完成的调研决策：
1. **JSON 库选择**：接受 `serde_json` 依赖，理由：行业标准、零实现成本、可读性
2. **符号表位置**：优先生成独立 `.syms` 文件（简单、无大小限制）
3. **控制流重建**：模式匹配 + 回退到等价形式
4. **内建函数识别**：符号表映射表（精确，无需启发式）
5. **资源还原**：符号表标记 + 专门处理
6. **往返测试**：语义等价性（ROM 行为），而非逐字节相同
7. **错误处理**：优雅拒绝，清晰错误信息

所有 NEEDS CLARIFICATION 已解决。

---

## Phase 1: Design & Contracts ✅

**Outputs**: 
- [data-model.md](data-model.md) - 核心数据结构（SymbolTable、DecompileContext、AST、错误类型）
- [contracts/symbol-table.md](contracts/symbol-table.md) - 符号表 JSON schema 和验证规则
- [contracts/decompile-api.md](contracts/decompile-api.md) - 反编译器库 API 和 CLI 接口
- [quickstart.md](quickstart.md) - 端到端验证场景（往返测试、错误处理、性能）

### 关键设计决策

1. **数据模型**：
   - `SymbolTable`：编译器输出的 JSON 元数据（包含变量、函数、资源、内建函数映射、源码映射）
   - `DecompileContext`：反编译器内部状态（符号表 + ROM 镜像 + AST + 错误列表）
   - AST：与编译器 AST 结构兼容的语法树

2. **符号表格式**：
   - JSON（`serde_json` 序列化）
   - 版本化（v1.0.0），向后兼容
   - 包含所有反编译所需的元数据（变量地址、函数签名、资源位置、内建函数映射）

3. **反编译流程**：
   - Load ROM + Symbols → Build AST → Generate Source → Validate
   - 模式匹配识别常见控制流（if/while/for）
   - 遇到未知模式时输出占位符 + 警告
   - 内建函数调用通过地址映射表还原

4. **错误处理**：
   - 分类错误类型（RomNotFound、InvalidSymbols、UnsupportedMapper、SymbolMismatch 等）
   - 清晰的错误信息（指出问题和解决方法）
   - 优雅拒绝非 NESL ROM（不输出错误代码）

5. **CLI 接口**：
   ```bash
   nesc build hello.nesl --debug-symbols  # 生成 .nes + .syms
   nesc decompile hello.nes -o output.nesl
   ```

### 验收标准（来自 quickstart.md）

- ✅ 所有 `examples/*.nesl` 往返成功（语义等价）
- ✅ UNROM/MMC3 正确处理
- ✅ 资源（tile/map/music）完整还原
- ✅ 非 NESL ROM 被正确拒绝
- ✅ 性能 <5 秒处理 64KB ROM
- ✅ 所有测试通过

---

## Next Step

运行 `/speckit-tasks` 生成实现任务列表（tasks.md），然后开始 TDD 实现。

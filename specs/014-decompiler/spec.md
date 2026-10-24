# Feature Specification: NESL Decompiler

**Feature Branch**: `014-decompiler`

**Created**: 2026-10-24

**Status**: Draft

**Input**: 反编译器：将 NESL 编译器生成的 .nes ROM 文件反编译回 NESL 源代码

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Round-trip Compilation (Priority: P1)

开发者将一个简单的 NESL 程序（如 `hello.nesl`）编译成 `.nes`，然后用反编译器还原回源码，验证往返一致性。

**Why this priority**: 这是反编译器的核心价值 —— 证明 NESL 编译是可逆的。如果连简单程序都无法往返，反编译器无法交付任何实际价值。

**Independent Test**: 使用 `examples/hello.nesl`（约 16 行代码），编译 → 反编译 → 重新编译，断言两次生成的 ROM 逐字节相同。

**Acceptance Scenarios**:

1. **Given** `hello.nesl` 包含全局变量、常量数组、函数和内建函数调用，**When** 执行 `nesc build hello.nesl --debug-symbols` 生成 `hello.nes` 和 `hello.syms`，然后执行 `nesc decompile hello.nes --symbols hello.syms -o hello.dec.nesl`，**Then** `hello.dec.nesl` 是合法的 NESL 源码，包含原始的变量名、类型声明和函数签名。

2. **Given** `hello.dec.nesl` 是反编译输出，**When** 执行 `nesc build hello.dec.nesl -o hello2.nes`，**Then** `hello2.nes` 与原始的 `hello.nes` 的代码段（PRG-ROM 中的指令序列）在语义上等价（允许变量地址重排，但逻辑相同）。

---

### User Story 2 - NROM Resource Reconstruction (Priority: P2)

开发者反编译包含图块（tile）、地图（map）、调色板的 NROM 游戏，还原出所有资源定义。

**Why this priority**: NESL 的核心卖点是"无需汇编的游戏开发"，资源（tile/map）是游戏的视觉部分。能还原资源才能让反编译输出成为可编辑的源码。

**Independent Test**: 使用 `examples/sprite.nesl` 或 `examples/game.nesl`（包含 `tile` 定义和调色板），反编译后检查输出是否包含完整的 `tile` 声明和 `const PAL` 数组。

**Acceptance Scenarios**:

1. **Given** 一个包含 `tile 1 = ["..3333..", ...]` 定义的 NESL 程序，**When** 反编译器读取 ROM 中的 CHR 数据（$0000-$1FFF），**Then** 输出文件包含对应的 `tile N = [...]` 声明，每个图块的 8 行像素字符串与原始源码一致。

2. **Given** 一个包含 `const PAL: [u8] = [0x0F, 0x30, ...]` 的程序，**When** 反编译器检测到符号表中的常量数组标记和 PRG-ROM 中的数据段，**Then** 输出包含同样的 `const PAL: [u8] = [...]` 声明。

---

### User Story 3 - UNROM Multi-Bank Decompilation (Priority: P3)

开发者反编译一个 UNROM（mapper 2）游戏（如 6-world 的 `quest.nesl`），还原出所有分 bank 的地图和代码。

**Why this priority**: UNROM 是 NESL 支持的第一个 banked mapper，验证反编译器能处理多 bank 结构是扩展性的关键。但这是增量功能 —— 即使没有它，P1/P2 仍能交付可用的反编译器。

**Independent Test**: 使用 `examples/quest.nesl`（UNROM 4 banks, 6 worlds），反编译后检查是否还原出 `const WORLDS: [map] = [W1, W2, W3, W4, W5, W6]` 和每个地图的定义。

**Acceptance Scenarios**:

1. **Given** 一个 UNROM ROM，其 iNES 头部声明 mapper 2 和 4 个 16KB PRG banks，**When** 反编译器读取每个 bank 的内容和固定 bank（最后一个）中的主代码，**Then** 输出文件正确还原出分 bank 的地图定义（每个地图标注所在 bank）和主函数。

2. **Given** `quest.nesl` 的反编译输出，**When** 重新编译该输出，**Then** 生成的 ROM 具有相同的 bank 数量和地图分布（bank 分配算法可能不同，但每个地图仍在一个完整的 bank 中）。

---

### User Story 4 - MMC3 IRQ Handler Decompilation (Priority: P3)

开发者反编译一个 MMC3（mapper 4）游戏（如 `split.nesl`），还原出扫描线中断回调函数。

**Why this priority**: MMC3 的 IRQ 是高级特性，验证反编译器能识别特殊的中断向量和 MMC3 寄存器配置。这是 P3 因为它只影响少数使用 MMC3 的项目。

**Independent Test**: 使用 `examples/split.nesl`（MMC3 分屏），反编译后检查是否还原出 `fn irq_handler() { ... }` 和其中的 MMC3 寄存器写入。

**Acceptance Scenarios**:

1. **Given** 一个 MMC3 ROM，其 IRQ 向量指向一个自定义处理函数，**When** 反编译器检测到 iNES 头部的 mapper 4 标记和 IRQ 向量非默认值，**Then** 输出包含 `fn irq_handler() { ... }` 声明和其函数体（可能简化为内建函数调用或汇编内联块）。

---

### Edge Cases

- **符号表丢失**：如果用户只提供 `.nes` 文件而没有 `.syms` 文件，反编译器能否推断出基本结构（如函数边界、常量数据段），但变量名用占位符（`var_0300`）？
- **不完整的调试符号**：如果符号表只覆盖部分代码（如用户手动修改了 ROM），反编译器如何处理未标记的区域？
- **非 NESL 生成的 ROM**：如果用户尝试反编译一个用 CC65 或手写汇编生成的 ROM，反编译器应报错"此 ROM 不包含 NESL 调试符号，无法反编译"，而不是输出错误的 NESL 代码。
- **混淆或加密的 ROM**：如果 ROM 数据被加密或打包（如某些商业游戏），反编译器应检测并拒绝处理。

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: 编译器 MUST 在编译时生成调试符号表文件（`.syms` 或嵌入 ROM），包含所有全局变量、常量、函数的名称、类型、地址映射。
- **FR-002**: 符号表 MUST 包含每条 6502 指令到 NESL 源码行号的映射（源码映射，source map）。
- **FR-003**: 符号表格式 MUST 可扩展（使用结构化格式如 JSON），以便未来版本添加更多元数据（如注释、优化提示）。
- **FR-004**: 反编译器 MUST 能读取 iNES 格式的 ROM 文件，解析头部（mapper 类型、PRG/CHR 大小），并根据 mapper 类型选择不同的反编译策略。
- **FR-005**: 反编译器 MUST 能读取符号表文件，将 ROM 中的地址映射回 NESL 符号（变量名、函数名）。
- **FR-006**: 反编译器 MUST 还原全局变量声明（`var x: u8 = 120;`），包括类型（u8/u16/i8）和初始值（从符号表或 ROM 中的初始化代码推断）。
- **FR-007**: 反编译器 MUST 还原常量数组声明（`const PAL: [u8] = [...]`），通过符号表定位数据段并读取字节序列。
- **FR-008**: 反编译器 MUST 还原函数签名（`fn add(a: u8, b: u8) -> u8`）和函数体，将 6502 指令序列转换回 NESL 语句（变量赋值、算术运算、控制流）。
- **FR-009**: 反编译器 MUST 识别内建函数调用模式（如 `JSR __sprite` 对应 `sprite(i, x, y, tile, attr)`），还原出高级 API 调用而非底层 JSR 指令。
- **FR-010**: 反编译器 MUST 还原控制流结构（`if`/`while`/`for`/`loop`），通过分析 6502 分支指令（BEQ/BNE/BCC 等）和跳转目标重建高级控制流。
- **FR-011**: 反编译器 MUST 还原图块定义（`tile N = [...]`），从 CHR-ROM 中读取 8x8 像素数据并转换为 8 行字符串（`.` 和 `1`/`2`/`3`）。
- **FR-012**: 反编译器 MUST 还原地图定义（`map LEVEL = [...]`），从符号表定位地图数据段并转换为字符串数组。
- **FR-013**: 反编译器 MUST 支持 NROM（mapper 0）、UNROM（mapper 2）和 MMC3（mapper 4）三种 mapper 的反编译。
- **FR-014**: 对于 UNROM，反编译器 MUST 能识别每个 bank 的边界，还原出分 bank 的地图定义和固定 bank 中的主代码。
- **FR-015**: 对于 MMC3，反编译器 MUST 能识别 IRQ 中断向量，还原出 `fn irq_handler() { ... }` 函数（如果符号表标记了该函数）。
- **FR-016**: 反编译器 MUST 验证 ROM 是否由 NESL 生成（检查符号表存在性或 ROM 中的特定魔数），如果不是则报错并拒绝处理。
- **FR-017**: 反编译器输出的 NESL 源码 MUST 是合法的、可重新编译的代码（通过 `nesc check` 验证无语法错误）。
- **FR-018**: 反编译器 MUST 在输出文件顶部添加注释，说明这是反编译生成的代码、原始 ROM 的文件名和反编译时间。

### Key Entities

- **调试符号表（Debug Symbol Table）**：编译器输出的元数据文件，包含：
  - 变量表：`{ name: "x", type: "u8", addr: 0x0300, init_value: 120 }`
  - 常量表：`{ name: "PAL", type: "[u8]", addr: 0x8100, length: 32 }`
  - 函数表：`{ name: "main", addr: 0x8000, params: [], return_type: "void" }`
  - 源码映射：`{ nes_addr: 0x8000, source_line: 10, source_file: "hello.nesl" }`
  - 资源表：`{ name: "tile_1", type: "tile", chr_offset: 0x10 }`

- **反编译器输出（Decompiled Source）**：结构化的 NESL 源码，包含：
  - 顶层声明：常量、变量、函数、资源（按原始源码顺序排列，或按地址升序）
  - 函数体：高级语句（赋值、控制流、内建函数调用），而非裸的 6502 指令

- **ROM 元数据（ROM Metadata）**：从 iNES 头部和 ROM 内容提取的信息：
  - Mapper 类型、PRG/CHR 大小、镜像模式
  - 中断向量（NMI/RESET/IRQ 地址）
  - 特殊标记（如 NESL 编译器在固定位置写入的魔数，用于快速识别 NESL ROM）

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 对于所有 `examples/` 目录下的 NESL 示例程序（hello/sprite/game/shooter/maze/runner/quest/split），编译 → 反编译 → 重新编译的往返成功率达到 100%（生成的第二个 ROM 与第一个 ROM 在代码段语义上等价）。

- **SC-002**: 反编译器能在 5 秒内处理一个 64KB 的 UNROM ROM（包括读取符号表、解析 ROM、生成输出文件），在普通开发机器上（8GB RAM, 现代 CPU）。

- **SC-003**: 反编译输出的 NESL 源码可读性达到"人类可编辑"标准：变量名保留原始名称（不是 `var_0300`），控制流缩进正确，注释标注了关键的地址映射（如 `// 原地址: $8000`）。

- **SC-004**: 反编译器对非 NESL ROM 的拒绝率达到 100%：尝试反编译任意商业游戏 ROM（如 Super Mario Bros）或 CC65 生成的 ROM 时，反编译器应在 1 秒内检测并报错，而不是输出错误的 NESL 代码。

## Assumptions

- **符号表格式**：采用 JSON 格式存储调试符号，嵌入 ROM 的方式是在 PRG-ROM 末尾预留一个固定大小的区域（如最后 4KB），头部写入魔数 `NESLDBG\0` 和符号表长度，后续字节为 JSON 数据（压缩或明文）。如果 ROM 空间不足，符号表作为独立的 `.syms` 文件输出。

- **反编译策略**：优先使用符号表驱动的反编译（有符号 → 直接映射），对于缺失符号的部分才回退到启发式分析（如识别常见的 JSR 模式）。这意味着反编译器的准确性高度依赖符号表的完整性。

- **控制流重建限制**：NESL 的控制流（if/while/for）编译后可能被优化或内联，反编译时无法 100% 还原原始的嵌套结构，可能输出等价但略有差异的控制流（如 `while (cond) { ... }` 反编译成 `loop { if !cond { break; } ... }`）。

- **不支持手动修改的 ROM**：如果用户编译后用十六进制编辑器修改了 ROM（如修改常量值或插入自定义汇编），反编译器只保证还原符号表覆盖的部分，修改的区域可能输出为占位符或注释块（`/* 未知代码段 $8A00-$8A1F */`）。

- **资源还原的精度**：图块（tile）的反编译依赖 CHR-ROM 数据，但 NESL 编译器可能对图块进行重排（如按使用频率排序），反编译输出的 `tile` 定义顺序可能与原始源码不同，但像素内容相同。

- **Mapper 特定逻辑**：UNROM 的 bank 切换代码（`JSR __bank`）和 MMC3 的寄存器配置（写入 $8000/$8001）会被识别并转换为高级注释（`// Bank switch to 2`），而非生成用户可编辑的 NESL 语句（因为 NESL 没有显式的 bank 切换 API）。

- **CLI 接口**：反编译器作为 `nesc decompile <rom> [--symbols <syms>] -o <output>` 子命令实现，符号表路径默认为 `<rom>.syms`（如果存在）。Web 编辑器在 v1 中不提供反编译功能（因为需要上传 ROM 文件，涉及版权敏感问题）。

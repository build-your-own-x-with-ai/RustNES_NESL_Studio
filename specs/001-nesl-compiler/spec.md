# Feature Specification: NESL 高级语言 → NES ROM 编译器与编辑器

**Feature Branch**: `001-nesl-compiler`

**Created**: 2026-10-15

**Status**: Implemented

**Input**: User description: "Git 管理，TDD，实现 NES 编辑器，SpecKit 那样有需求/计划/实现文档，不用编写汇编语言，使用高级语言就能编译出 NES 文件"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 用高级语言写游戏逻辑并编译成 .nes (Priority: P1) 🎯 MVP

开发者用类 Rust 的高级语言 NESL 编写变量、函数、条件、循环、数组等逻辑，
执行一条命令即可得到可在任意 NES 模拟器（FCEUX、Mesen、jsnes…）中运行的 `.nes` 文件，
全程不需要接触 6502 汇编。

**Why this priority**: 这是整个产品存在的意义；没有编译器其他一切无从谈起。

**Independent Test**: 编译一段计算程序（如累加、函数调用、数组），在内置 6502 模拟器中执行
生成的 ROM，断言内存中变量的值与预期一致。

**Acceptance Scenarios**:

1. **Given** 一个只包含 `fn main()` 的 NESL 文件，**When** 运行 `nesc build game.nesl`，**Then** 生成合法的 iNES 文件（16 字节头 + 32KB PRG + 8KB CHR），复位向量指向运行时入口。
2. **Given** 使用 `u8`/`u16` 变量、算术、比较、`if/while/for/loop`、函数和数组的程序，**When** 编译并执行，**Then** 全局变量的最终值与按语言语义手工计算的结果一致。
3. **Given** 源码有语法或类型错误，**When** 编译，**Then** 不生成 ROM，并输出带 `文件:行:列` 的错误信息。

---

### User Story 2 - 使用内建 NES API 画图、读手柄、显示文字 (Priority: P2)

开发者通过内建函数完成 NES 常见操作：加载调色板、定义图块（tile）、写背景、显示文字和数字、
控制精灵、读取手柄、等待垂直同步、产生随机数、播放简单音调。

**Why this priority**: 只有计算没有画面不能算游戏；这些 API 替代了初学者最难的汇编部分。

**Independent Test**: 编译调用 `load_palette`/`print`/`sprite`/`joypad` 的程序，在测试机中运行若干帧，
断言 VRAM 调色板/命名表内容、OAM 精灵数据、手柄输入驱动的变量变化。

**Acceptance Scenarios**:

1. **Given** `load_palette(PAL)`，**When** 运行，**Then** VRAM `$3F00-$3F1F` 等于 `PAL` 的 32 字节。
2. **Given** `print(2, 3, "HELLO")`，**When** 运行，**Then** 命名表 `$2000 + 3*32 + 2` 起依次是 `H E L L O` 的 ASCII 码，且 CHR 中内置了对应字形。
3. **Given** 主循环中 `sprite(0, x, y, 1, 0)` 与 `wait_vblank()`，**When** 运行一帧，**Then** OAM 第 0 个精灵为 `(y, 1, 0, x)`。
4. **Given** 按住手柄右键，**When** 程序 `if joypad(0) & BTN_RIGHT { x += 1; }` 运行 N 帧，**Then** `x` 增加 N。
5. **Given** 源码中 `tile 1 = ["..3333..", ...]` 的 8 行像素定义，**When** 编译，**Then** CHR 中第 1 个图块的两个位平面按像素值编码。

---

### User Story 3 - 浏览器内的 NES 编辑器 (Priority: P3)

开发者运行 `nesc serve`，打开浏览器即可在编辑器中写 NESL 代码，点击「运行」即时编译并在页面内的
模拟器中游玩，编译错误定位到行，可下载 `.nes` 文件，可载入内置示例。

**Why this priority**: 降低使用门槛，形成「编辑 → 编译 → 运行」闭环；但 CLI 已能独立交付价值。

**Independent Test**: 启动服务后对 `POST /api/compile` 发送源码，断言返回的 JSON 包含 base64 ROM
或带行列号的错误；`GET /` 返回编辑器页面；`GET /api/examples` 返回示例列表。

**Acceptance Scenarios**:

1. **Given** 服务已启动，**When** 浏览器访问 `/`，**Then** 显示带行号的代码编辑区、运行/下载按钮、示例选择和模拟器画布。
2. **Given** 代码有错误，**When** 点击运行，**Then** 错误列表显示消息，并高亮对应行。
3. **Given** 代码正确，**When** 点击运行，**Then** ROM 在页面画布中运行，键盘可控制（方向键、Z=B、X=A、Enter=Start、Shift=Select）。

---

### Edge Cases

- 递归调用（直接或间接）：静态分配局部变量无法支持 → 编译错误 `recursion is not supported`。
- 未定义的变量/函数、重复定义、参数个数不符、`u16` 赋给 `u8` 未显式 `as u8` → 编译错误。
- 缺少 `fn main()` → 编译错误。
- 条件跳转超过 ±127 字节 → 汇编器自动改写为「反向短跳 + JMP」，对用户透明。
- 变量超出零页（$10-$FF）→ 自动溢出到 `$0300-$07FF`；RAM 用尽 → 编译错误。
- 程序代码 + 数据超过 32KB PRG → 编译错误。
- `u8` 运算溢出按 256 取模回绕；除以 0 结果定义为 `0xFF`（商）与被除数（余数），不崩溃。
- `load_palette` 参数不是 32 字节常量数组 → 编译错误。
- `tile` 行数不是 8、每行不是 8 个字符或含非法字符 → 编译错误。

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: 系统 MUST 将 NESL 源码编译为 iNES（Mapper 0/NROM-256，垂直镜像）ROM 文件。
- **FR-002**: 语言 MUST 支持 `u8`、`u16` 类型、全局 `var`、局部 `let`、`const` 常量与常量数组、全局 RAM 数组。
- **FR-003**: 语言 MUST 支持 `+ - * / % & | ^ << >>`、比较、`&& || !`、`~`、`as` 转换，以及复合赋值。
- **FR-004**: 语言 MUST 支持 `if/else if/else`、`while`、`loop`、`for i in a..b`、`break`、`continue`、`return`。
- **FR-005**: 语言 MUST 支持带参数和返回值的函数；检测并拒绝递归。
- **FR-006**: 系统 MUST 提供内建 NES API：`wait_vblank, ppu_on, ppu_off, load_palette, vram_addr, vram_put, vram_fill, print, print_u8, sprite, hide_sprite, joypad, scroll, rand, frame_count, peek, poke, play_tone, stop_sound`，以及按键常量 `BTN_*`。
- **FR-007**: 系统 MUST 内置 ASCII 字体（`0x20-0x5F`）到 CHR，使 `print` 无需额外资源即可显示文字。
- **FR-008**: 语言 MUST 支持 `tile N = [8 行字符串]` 以文本像素图定义 CHR 图块。
- **FR-009**: 所有编译错误 MUST 带行号、列号和可读消息；CLI 以非零码退出。
- **FR-010**: CLI MUST 提供 `nesc build <src> [-o out.nes] [--asm]`（`--asm` 输出汇编清单便于调试）、`nesc check <src>`、`nesc serve [--port N]`。
- **FR-011**: Web 编辑器 MUST 提供编辑、编译、错误定位、浏览器内运行、下载 ROM、加载示例功能。
- **FR-012**: 编译 MUST 确定性：同一输入两次编译逐字节相同。

### Key Entities

- **Source Program**：NESL 源码文本，由常量、全局变量、图块、函数组成。
- **ROM Image**：iNES 文件，包含头部、PRG（代码+只读数据+向量）、CHR（字体+用户图块）。
- **Diagnostic**：编译错误，含行、列、消息。
- **Runtime**：编译器自动链接的 6502 例程（复位、NMI、手柄、乘除法、打印等）。

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 用户无需编写任何汇编，即可用 ≤ 60 行 NESL 写出一个可用手柄移动精灵并显示分数的程序。
- **SC-002**: 所有语言特性和内建 API 均有「编译 → 模拟执行 → 断言」的自动化测试，`cargo test` 全绿。
- **SC-003**: 编译 1000 行源码耗时 < 1 秒。
- **SC-004**: 从 `nesc serve` 到浏览器内看到示例运行，操作步骤 ≤ 3 步。

## Assumptions

- 目标用户熟悉一门高级语言（C/Rust/JS/Python），但不熟悉 6502 汇编。
- v1 只支持 NROM（32KB PRG/8KB CHR），不做 bank 切换；不支持有符号数、浮点、结构体、指针。
- 局部变量静态分配（类似许多 8 位编译器），因此不支持递归。
- 浏览器内运行使用开源 JS 模拟器 jsnes（CDN 加载）；离线时仍可编译和下载 ROM。
- VRAM 写入（`print`/`vram_*`）应在 `ppu_off()` 期间或紧跟 `wait_vblank()` 之后少量进行，这是 NES 硬件限制，文档中说明。

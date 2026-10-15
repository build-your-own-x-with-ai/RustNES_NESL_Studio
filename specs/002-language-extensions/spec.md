# Feature Specification: NESL 语言增强（struct / i8 / u16 数组 / 局部数组 / match）

**Feature Branch**: `002-language-extensions`

**Created**: 2026-10-15

**Status**: Implemented

**Input**: User description: "语言增强：struct、i8/有符号数、u16 数组、局部数组、match；写游戏时最常碰到的缺口"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 用 struct 组织游戏对象 (Priority: P1) 🎯 MVP

开发者用 `struct Enemy { x: u8, y: u8, hp: u8 }` 描述游戏对象，声明单个对象 `var boss: Enemy;`
或对象数组 `var enemies: [Enemy; 8];`，用 `boss.hp`、`enemies[i].x += 1` 读写字段。

**Why this priority**: 没有 struct 时多个敌人只能写成多组平行数组，是写游戏最痛的地方。

**Independent Test**: 编译使用结构体变量、结构体数组、字段复合赋值的程序，执行后断言各字段值。

**Acceptance Scenarios**:

1. **Given** `var boss: Enemy;`，**When** `boss.hp = 9; boss.x = boss.hp + 1;`，**Then** 两字段分别为 9 和 10。
2. **Given** `var es: [Enemy; 8];`，**When** 循环 `es[i].y += i`，**Then** 每个元素的 y 等于下标。
3. **Given** 访问不存在的字段 `boss.z`，**When** 编译，**Then** 报错 `no field `z` in struct `Enemy``，带行列。
4. **Given** 把整个结构体当作值使用（`a = boss;`），**When** 编译，**Then** 报错提示访问字段。

---

### User Story 2 - 有符号数 i8 表示速度/方向 (Priority: P2)

开发者用 `var vx: i8 = -2;` 表示速度，`x += vx` 让坐标左右移动；`if vy < 0` 做有符号比较；
`as u16` 时符号扩展，`>>` 为算术右移。

**Why this priority**: 速度、加速度、相对偏移天然是有符号的；用 u8 模拟容易出错。

**Independent Test**: 执行有符号比较、混合加法、符号扩展、算术右移，断言结果。

**Acceptance Scenarios**:

1. **Given** `x: u8 = 10, vx: i8 = -3`，**When** `x += vx`，**Then** x = 7。
2. **Given** `a: i8 = -5, b: i8 = 3`，**When** 比较，**Then** `a < b` 为真、`a > b` 为假；`-128 < 127` 为真。
3. **Given** `w: u16 = 1000, v: i8 = -1`，**When** `w += v`，**Then** w = 999（符号扩展）。
4. **Given** `a: i8 = -8`，**When** `a >> 1`，**Then** 结果 -4。
5. **Given** 非常量 i8 与 u8 比较，**When** 编译，**Then** 报错要求显式转换。

---

### User Story 3 - u16 数组与局部数组 (Priority: P2)

开发者声明 `var hiscores: [u16; 5];`、`const NOTES: [u16] = [0x1AB, 0x17C];`、
函数内 `let buf: [u8; 8];`（每次进入置零）。

**Why this priority**: 分数表、音符周期表都是 u16；临时缓冲放全局会污染命名空间。

**Independent Test**: 读写 u16 数组元素（常量/变量下标）、读取 u16 常量表、两次调用函数验证局部数组被清零。

**Acceptance Scenarios**:

1. **Given** `var t: [u16; 4];`，**When** `t[i] = 300 * ...`/`t[2] += 1000`，**Then** 元素值正确。
2. **Given** `const N: [u16] = [0x1AB, 0x17C]`，**When** `play_tone(N[i])`，**Then** 写入 APU 的周期正确。
3. **Given** 函数里 `let buf: [u8; 4];`，**When** 调用两次且每次累加写入，**Then** 第二次开始时 buf 为全 0。

---

### User Story 4 - match 多分支 (Priority: P3)

开发者用 `match state { 0 => title(), 1 | 2 => play(), 3..=9 => { ... } _ => {} }` 替代长 `if-else`。

**Why this priority**: 游戏状态机、按值分发很常见；语法糖但显著提升可读性。

**Independent Test**: 对多个输入值执行 match，断言命中的分支；包括 `|`、`a..=b`、`_`、i8 负数模式。

**Acceptance Scenarios**:

1. **Given** 带值、或模式、闭区间、通配的 match，**When** 输入各种值，**Then** 命中第一个匹配的分支且只执行一次。
2. **Given** 被匹配表达式含函数调用，**When** 执行，**Then** 只求值一次。
3. **Given** 没有任何分支匹配且无 `_`，**When** 执行，**Then** 什么都不做。

### Edge Cases

- 结构体字段只能是 `u8`/`i8`/`u16`；结构体不能嵌套、不能作为参数/返回值（v2 限制，编译错误）。
- 数组长度 1-256；u16 数组按低/高字节两张表存储，因此长度上限同样 256。
- 常量放入目标类型：i8 接受 -128..=255（≥128 视为位模式），u8 接受 -256..=255。
- `i8` 的 `* / %` 仅支持乘以 2 的幂常量；其他报错。
- 无初始化的 `let` 只允许数组和结构体（自动置零）；标量 `let` 必须初始化。
- match 模式必须是常量；区间 `a..=b` 中 a > b 报错。

## Requirements *(mandatory)*

### Functional Requirements

- **FR-201**: 语言 MUST 支持 `struct Name { field: type, ... }` 声明，字段类型 u8/i8/u16。
- **FR-202**: 全局与局部 MUST 支持结构体变量与结构体数组；`a.f`、`a[i].f` 可读可写可复合赋值。
- **FR-203**: 语言 MUST 支持 `i8` 类型：有符号比较、算术右移、`as u16` 符号扩展、与 u8 混合加减得到 u8。
- **FR-204**: 语言 MUST 支持 `[u16; N]` RAM 数组与 `const X: [u16] / [i8]` 常量表。
- **FR-205**: 函数内 MUST 支持 `let name: [T; N];` 与 `let s: Struct;`，每次执行到声明时置零。
- **FR-206**: 语言 MUST 支持 `match` 语句：常量值、`|` 组合、`a..=b` 闭区间、`_` 通配；分支体为块或单条语句（逗号分隔）。
- **FR-207**: 001 的全部测试 MUST 继续通过（向后兼容）。

### Key Entities

- **StructDef**：名称 + 有序字段（名称、标量类型）。
- **结构体存储**：采用 SoA（Structure of Arrays）布局 —— `enemies[i].x` 编译为 `LDA enemies.x,X`，无需乘法。

## Success Criteria *(mandatory)*

- **SC-201**: 预览中的敌人循环示例可原样编译运行。
- **SC-202**: `enemies[i].x` 的读取生成 ≤ 2 条指令（下标为变量时 `LDX i; LDA base,X`）。
- **SC-203**: `cargo test --workspace` 全绿，001 测试无回归。

## Assumptions

- 不引入 i16；不支持结构体整体赋值/比较。
- SoA 布局对用户透明（`nesc build --asm` 中能看到 `enemies.x` 这样的符号）。

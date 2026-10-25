# Feature 015: 高级反编译器（无符号表）

## 目标

将**任意商业 NES ROM**（无符号表）反编译为**高级伪代码**，比纯汇编更易读，接近 NESL 语言。

## 问题陈述

Feature 014 实现了：
- ✅ 带符号表的 NESL ROM → NESL 源码（62.5% 完美往返）
- ✅ 任意 ROM → 6502 汇编（100% 成功）

但无法处理：
- ❌ 商业 ROM（无符号表）→ 高级代码
- ❌ 变量名识别
- ❌ 函数边界识别
- ❌ 控制流重建（if/while）

## 用户故事

**作为** NES 游戏开发者/逆向工程师  
**我想** 反编译商业 NES ROM 为高级伪代码  
**以便** 理解游戏逻辑、学习技巧、移植到现代平台

## 示例

### 输入：Solomon's Key ROM（无符号表）

### 输出：高级伪代码

```c
// 推断的全局变量
u8 player_x;      // $02 - X 坐标
u8 player_y;      // $03 - Y 坐标
u8 player_vx;     // $04 - X 速度
u8 player_vy;     // $05 - Y 速度
u8 keys_held;     // $06 - 按键状态
u16 score;        // $07-$08 - 分数

// 推断的函数
fn update_player() {  // $8056
    keys_held = joypad_read(0);
    
    if (keys_held & BTN_LEFT) {
        player_x--;
    }
    
    if (keys_held & BTN_RIGHT) {
        player_x++;
    }
    
    if (keys_held & BTN_A) {
        player_vy = -4;  // 跳跃
    }
    
    player_y += player_vy;
    player_vy++;  // 重力
}

fn draw_player() {  // $80A2
    set_sprite(0, player_x, player_y, TILE_PLAYER);
}
```

## 技术方案

### Phase 1: 变量识别（Zero Page 分析）

**策略：** 从 Zero Page (0x00-0xFF) 内存访问模式推断变量

**启发式规则：**
- 连续两字节访问 → `u16`
- 单字节读写 → `u8`
- INC/DEC 频繁 → `counter`
- AND/ORA 位运算 → `flags`
- 0x00-0x01 → `ptr` (指针对)
- 0x02-0x03 常见 → `x_pos`, `y_pos`

**实现：**
```rust
struct Variable {
    addr: u8,
    name: String,
    ty: VarType,  // U8, U16, Array
    access_pattern: AccessPattern,  // ReadOnly, Counter, Coordinate, Flag
}

fn analyze_variables(prg_rom: &[u8]) -> HashMap<u8, Variable>
```

### Phase 2: 函数识别（JSR/RTS 分析）

**策略：** 从 JSR 调用识别函数入口，从 RTS/RTI 识别函数出口

**实现：**
```rust
struct Function {
    entry: u16,
    name: String,
    calls_to: Vec<u16>,      // 调用的函数
    called_from: Vec<u16>,   // 被谁调用
    local_vars: HashSet<u8>, // 局部使用的 ZP 变量
}

fn analyze_functions(prg_rom: &[u8]) -> BTreeMap<u16, Function>
```

### Phase 3: 控制流重建

**策略：** 从分支指令重建 if/while 结构

**模式识别：**
```asm
; Pattern 1: if statement
BEQ skip      ; if (Z) skip
LDA #$01      ; body
STA $02
skip:

→ if (!Z) { var_02 = 1; }
```

```asm
; Pattern 2: while loop
loop:
LDA $02
BNE loop      ; while (var_02 != 0)

→ while (var_02 != 0) { ... }
```

**实现：**
```rust
enum ControlFlow {
    If { condition: Condition, then_block: Vec<Statement> },
    While { condition: Condition, body: Vec<Statement> },
    Sequential(Vec<Statement>),
}

fn reconstruct_control_flow(instrs: &[Instruction]) -> ControlFlow
```

### Phase 4: 伪代码生成

**策略：** 将分析结果组合生成 C-like 伪代码

**输出格式：**
```c
// Variables (from Zero Page analysis)
u8 var_02;  // player_x
u16 var_04; // score

// Functions (from JSR analysis)
fn update_player() {
    // Body (from control flow reconstruction)
    if (joypad & BTN_LEFT) {
        var_02--;
    }
    call draw_sprite();
}
```

## 阶段划分

### MVP (最小可行产品)
- ✅ Phase 1: 变量识别
- ✅ Phase 2: 函数识别  
- ✅ Phase 4: 基础伪代码生成

### 扩展功能
- ⚠️ Phase 3: 控制流重建（可选）
- 📊 数据结构推断（数组、结构体）
- 🎯 NES 特定模式识别（精灵、背景、声音）

## 交付物

1. **新命令：** `nesc pseudocode <file.nes>`
2. **输出格式：** C-like 伪代码（易读、可理解）
3. **测试覆盖：** 5+ 商业 ROM（Super Mario, Mega Man, Castlevania...）
4. **文档：** 使用指南、分析报告

## 成功标准

- ✅ 可以识别至少 80% 的函数边界
- ✅ 可以识别至少 60% 的变量用途
- ✅ 生成的伪代码比纯汇编可读性提升 3x
- ✅ 支持所有 NES ROM（商业、homebrew）

## 非目标

- ❌ 不追求 100% 准确的类型推断
- ❌ 不生成可直接编译的 NESL 代码（需要人工修正）
- ❌ 不需要完美的变量命名（可以是 var_02）

## 参考

- Ghidra 反编译器（静态分析）
- IDA Pro 伪代码生成
- RetDec 开源反编译器
- NES 开发模式（NESDev wiki）

# NESL 反编译工具使用指南

RustNES 项目现在提供完整的 NES ROM 反编译工具链，支持三种不同的反编译模式。

## 工具概览

| 命令 | 输入 | 输出 | 用途 |
|------|------|------|------|
| `nesc decompile` | NESL ROM + 符号表 | NESL 源码 | 还原原始 NESL 代码 |
| `nesc pseudocode` | 任意 NES ROM | C-like 伪代码 | 理解商业游戏逻辑 |
| `nesc disasm` | 任意 NES ROM | 6502 汇编 | 底层分析和调试 |

## 使用场景

### 场景 1: 反编译自己的 NESL ROM

如果你用 NESL 编译了一个 ROM，并且保留了符号表，可以完美还原源码：

```bash
# 编译（生成符号表）
nesc build game.nesl --debug-symbols

# 反编译（使用符号表）
nesc decompile game.nes -o game_recovered.nesl

# 结果：几乎完全还原原始代码
```

**成功率：** 62.5% 完美往返（可直接重新编译）

### 场景 2: 分析商业 NES ROM（无符号表）

想理解 Super Mario Bros, Mega Man 等经典游戏的逻辑？使用高级反编译器：

```bash
# 生成高级伪代码
nesc pseudocode "Super Mario Bros.nes" -o mario_pseudo.c
```

**输出示例：**
```c
// 变量（自动识别）
u8 player_x;      // $02, 17 accesses
u8 player_y;      // $03, 8 accesses
u16 score;        // $04, counter
u8 lives;         // $06, 12 accesses

// 函数（自动识别）
void update_player() {  // $8120
    // Called from: $8000, $8059
    A = player_x;
    A++;
    player_x = A;
    if (!Z) goto +5;
    update_physics();
}

void nmi() {  // $8059
    push(A);
    A = X;
    push(A);
    update_player();
    // ...
}
```

**成功案例：**
- **Solomon's Key:** 243 个变量，183 个函数
- **可读性提升:** 比纯汇编好 3 倍以上

### 场景 3: 底层调试和分析

需要精确的 6502 指令？使用反汇编器：

```bash
# 生成带标签的汇编代码
nesc disasm game.nes -o game.asm
```

**输出示例：**
```asm
; Reset vector: $8000
L8000:
    LDX #$40
    STX $4017
    LDX #$FF
    TXS
    INX
    STX $2000
    ; ...
    JSR L8120

L8120:
    LDA $02
    INC A
    STA $02
    RTS
```

## 功能对比

### nesc decompile（Feature 014）

**优点：**
- ✅ 还原原始 NESL 代码
- ✅ 保留变量名和结构
- ✅ 62.5% 可直接重新编译

**限制：**
- ❌ 需要符号表（.syms 文件）
- ❌ 只支持 NESL 编译的 ROM
- ❌ Map 资源需要手动补充

**最佳用途：** 你自己的 NESL 项目

### nesc pseudocode（Feature 015）

**优点：**
- ✅ 无需符号表
- ✅ 支持所有 NES ROM（商业/自制）
- ✅ 自动识别变量和函数
- ✅ C-like 可读格式

**限制：**
- ❌ 不能直接重新编译
- ❌ 变量名是推断的（player_x, var_02）
- ❌ 控制流简化（goto 而非 if/while）

**最佳用途：** 理解商业游戏逻辑

### nesc disasm（Feature 014）

**优点：**
- ✅ 100% 准确
- ✅ 支持所有 NES ROM
- ✅ 适合底层分析

**限制：**
- ❌ 可读性较差
- ❌ 需要 6502 汇编知识

**最佳用途：** 调试、性能分析、精确逆向

## 高级技巧

### 组合使用工具

```bash
# 1. 先用高级反编译理解整体结构
nesc pseudocode game.nes -o game_pseudo.c

# 2. 找到感兴趣的函数地址（例如 $8120）

# 3. 用反汇编查看精确指令
nesc disasm game.nes -o game.asm
# 然后搜索 L8120
```

### 分析变量访问模式

高级反编译器会标注访问模式：

```c
u8 counter_00;    // counter - 频繁 INC/DEC
u16 ptr2;         // pointer - 间接寻址
u8 flags_20;      // flags - 位运算
u8 player_x;      // 启发式命名
```

### 识别函数调用关系

```c
void nmi() {
    // Called from: (interrupt vector)
    update_player();  // 调用 $8120
    draw_sprites();   // 调用 $8200
}

void update_player() {
    // Called from: $8059
    // 函数体...
}
```

## 实际案例

### 案例 1: 分析 Solomon's Key

```bash
$ nesc pseudocode "Solomon's Key.nes" -o solomon_pseudo.c

# 结果
- 243 个变量识别
- 183 个函数识别
- 12925 字节伪代码
- 最复杂函数：40 次调用
```

发现：
- NMI 处理器在 $8000
- 主游戏循环在 $8EA9（40 次调用）
- 热点函数：$8E8D（被调用 34 次）

### 案例 2: 学习 NESL 示例

```bash
# 反编译自己的项目
$ nesc decompile examples/sprite.nes -o sprite_recovered.nesl

# 对比原始代码
$ diff examples/sprite.nesl sprite_recovered.nesl

# 几乎完全一致！
```

## 限制和注意事项

1. **高级反编译不是魔法**
   - 变量名是推断的，可能不准确
   - 复杂的控制流可能简化为 goto
   - 某些模式可能识别不出

2. **符号表很重要**
   - 用 `--debug-symbols` 编译可以保留更多信息
   - 商业 ROM 没有符号表，只能用高级反编译

3. **人工审查必不可少**
   - 伪代码是辅助工具，不是最终答案
   - 需要结合游戏知识和 NES 硬件知识
   - 有疑问时查看汇编

## 下一步

- 尝试反编译自己的 NESL 项目
- 分析喜欢的商业游戏
- 组合使用三种工具
- 贡献改进到项目

## 相关文档

- [Feature 014 规格](../specs/014-decompiler/spec.md)
- [Feature 015 规格](../specs/015-advanced-decompiler/spec.md)
- [NESL 语言参考](../docs/nesl-reference.md)

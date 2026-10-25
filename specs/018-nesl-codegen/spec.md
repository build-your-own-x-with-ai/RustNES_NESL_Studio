# Feature 018: ROM → 可编译 NESL（无符号表）

## 目标

实现真正的反编译：商业 ROM → 可编译的 NESL 源码

**用例**：移植经典 NES 游戏到其他平台

## 挑战

### 技术难点

1. **变量类型推断**
   - 伪代码 `u16 ptr0` → NESL `var ptr0_lo: u8; var ptr0_hi: u8`
   - 数组推断：`u8[] flags` → `var flags_0: u8; var flags_1: u8; ...`

2. **控制流重建**
   - `goto` → `if/while/loop`
   - 标签 → 结构化控制流

3. **函数调用转换**
   - `sub_8000()` → `fn init() { ... }`
   - 识别参数和返回值

4. **数据段转换**
   - CHR-ROM → NESL tile 定义
   - 查找表 → NESL 常量数组

5. **寄存器抽象**
   - `A = X` → 临时变量
   - `push(A)` → 栈操作

## 可行性分析

| 难度 | 功能 | 完成度 |
|------|------|--------|
| ⭐ | 简单函数 | 80% |
| ⭐⭐ | 变量声明 | 60% |
| ⭐⭐⭐ | 控制流 | 30% |
| ⭐⭐⭐⭐ | 完整程序 | 10% |

### 现实评估

**完全自动化反编译几乎不可能**，因为：

1. **信息丢失**：编译过程不可逆
2. **优化混淆**：编译器优化难以还原
3. **汇编技巧**：手写汇编难以理解
4. **上下文缺失**：没有注释、变量名

### 实用方案

**半自动反编译 + 人工辅助**

```
ROM → 伪代码 (已完成)
  ↓
人工分析和注释
  ↓
手动转换为 NESL
  ↓
编译测试
  ↓
迭代修复
```

## 推荐方案

### 方案 A：半自动工具链 ⭐ 推荐

**目标**：生成 80% 可用的 NESL 框架，人工完成剩余 20%

**步骤**：

1. **生成框架**
   ```nesl
   // 自动生成的变量声明
   var ptr0_lo: u8;
   var ptr0_hi: u8;
   var counter_06: u8;
   
   // 自动生成的函数签名
   fn init() {
       // TODO: 填充逻辑
   }
   
   fn nmi() {
       // TODO: 填充逻辑
   }
   ```

2. **关键函数实现**
   - 自动转换简单函数（< 10 行）
   - 复杂函数生成 TODO 注释

3. **数据转换**
   - CHR-ROM → tile 定义（自动）
   - 查找表 → 常量数组（自动）

4. **人工完善**
   - 填充 TODO
   - 优化控制流
   - 添加注释

### 方案 B：完全手工移植

使用伪代码作为参考，完全手写 NESL：

1. 阅读伪代码理解逻辑
2. 用 NESL 重新实现
3. 参考原 ROM 的行为
4. 迭代测试

**优点**：代码质量高，可维护
**缺点**：工作量大（几周到几个月）

### 方案 C：部分移植

只移植游戏的核心逻辑：

```nesl
// 移植核心玩法
fn game_logic() {
    // 玩家移动
    // 碰撞检测
    // 得分计算
}

// 使用新的图形和音效
fn render() {
    // 现代化的渲染
}
```

## 实现计划（方案 A）

### Phase 1: NESL 框架生成器

```rust
pub fn generate_nesl_framework(rom: &Rom, analysis: &Analysis) -> String {
    let mut output = String::new();
    
    // 1. 变量声明
    output += &generate_variable_declarations(&analysis.variables);
    
    // 2. 常量定义
    output += &generate_constants(&analysis.read_only_vars);
    
    // 3. 函数签名
    output += &generate_function_signatures(&analysis.functions);
    
    // 4. 简单函数实现
    output += &generate_simple_functions(&analysis.functions);
    
    // 5. CHR-ROM 数据
    output += &generate_tile_data(&rom.chr_rom);
    
    output
}
```

### Phase 2: 简单函数转换

只转换简单的、无分支的函数：

```c
// 伪代码
void sub_FACA() {
    A &= 0x07;
    Y = A;
    return;
}
```

→

```nesl
fn mask_to_y() -> u8 {
    var result = A & 0x07;
    return result;
}
```

### Phase 3: 数据转换

```rust
fn generate_tile_data(chr_rom: &[u8]) -> String {
    // CHR-ROM → NESL tile 定义
    let mut output = String::new();
    
    for (i, tile) in extract_tiles(chr_rom).iter().enumerate() {
        output += &format!("tile {} {{\n", i);
        output += &format_tile(tile);
        output += "}\n\n";
    }
    
    output
}
```

### Phase 4: 交互式转换工具

```javascript
// 网页工具：辅助人工转换
class ConversionAssistant {
    showPseudocode(func);
    suggestNESL(func);
    validateNESL(code);
    testFunction(nesl_code);
}
```

## 预期结果

### Super Mario Bros 示例

**自动生成**（框架）：
```nesl
// 变量（自动推断）
var mario_x: u8;
var mario_y: u8;
var mario_state: u8;

// 函数框架
fn init() {
    // TODO: 初始化逻辑
}

fn update_player() {
    // TODO: 玩家更新逻辑
    // 参考伪代码 sub_8000
}

// Tile 数据（自动转换）
tile 0 { ... }
tile 1 { ... }
```

**人工完善** → **可编译的完整游戏**

## 工作量估算

| 游戏 | 代码量 | 自动化 | 人工 | 总时间 |
|------|--------|--------|------|--------|
| 简单游戏 | 5KB | 4KB | 1KB | 2-5 天 |
| 中等游戏 | 20KB | 15KB | 5KB | 1-2 周 |
| 复杂游戏 | 50KB+ | 35KB | 15KB | 1-2 月 |

## 成功标准

- ✅ 生成可编译的 NESL 框架
- ✅ 80% 变量和函数正确识别
- ✅ 简单函数自动转换
- ✅ CHR-ROM 完全转换
- ⚠️ 复杂控制流需要人工

## 非目标

- ❌ 100% 自动反编译（不可能）
- ❌ 完美还原原始源码（信息已丢失）
- ❌ 一键移植（需要人工验证）

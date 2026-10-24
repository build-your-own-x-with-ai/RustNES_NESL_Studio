# Contract: Decompile API

**Date**: 2026-10-24

## Overview

反编译器库 API，供 CLI 和测试使用。库位于 `nesl` crate 的 `decompile` 模块。

---

## Public API

### 1. Main Entry Point

```rust
/// 反编译 NES ROM 到 NESL 源码
///
/// # Arguments
/// * `rom_path` - ROM 文件路径（.nes）
/// * `symbols_path` - 符号表路径（.syms），如果为 None 则自动查找同名 .syms 文件
///
/// # Returns
/// * `Ok(String)` - 反编译的 NESL 源码
/// * `Err(DecompileError)` - 反编译失败
///
/// # Example
/// ```rust
/// let source = nesl::decompile::decompile("hello.nes", None)?;
/// std::fs::write("hello.dec.nesl", source)?;
/// ```
pub fn decompile(
    rom_path: impl AsRef<Path>,
    symbols_path: Option<impl AsRef<Path>>,
) -> Result<String, DecompileError>;
```

### 2. Low-Level API（高级用户）

```rust
/// 从内存中的 ROM 和符号表反编译
pub fn decompile_from_bytes(
    rom: &[u8],
    symbols: &str,  // JSON 字符串
) -> Result<String, DecompileError>;

/// 只解析符号表（不反编译）
pub fn parse_symbols(json: &str) -> Result<SymbolTable, DecompileError>;

/// 只验证符号表与 ROM 的一致性
pub fn validate_symbols(rom: &[u8], symbols: &SymbolTable) -> Result<(), DecompileError>;
```

---

## Error Types

```rust
#[derive(Debug)]
pub enum DecompileError {
    /// ROM 文件不存在或无法读取
    RomNotFound { path: String },
    
    /// 符号表文件不存在或无法读取
    SymbolsNotFound { path: String },
    
    /// ROM 格式错误（不是有效的 iNES 文件）
    InvalidRom { reason: String },
    
    /// 符号表格式错误（JSON 解析失败或字段缺失）
    InvalidSymbols { reason: String },
    
    /// 符号表版本不支持
    UnsupportedVersion { found: String, supported: String },
    
    /// Mapper 类型不支持
    UnsupportedMapper { mapper: u8 },
    
    /// 符号表与 ROM 不一致（mapper/大小/地址范围）
    SymbolMismatch { reason: String },
    
    /// ROM 不是 NESL 编译器生成的（缺少符号表或魔数）
    NotNeslRom,
    
    /// 反编译过程中遇到无法识别的模式
    UnrecognizedPattern { addr: u16, desc: String },
    
    /// I/O 错误
    Io(std::io::Error),
}

impl std::fmt::Display for DecompileError { ... }
impl std::error::Error for DecompileError { ... }
impl From<std::io::Error> for DecompileError { ... }
```

---

## Data Types（公开）

### SymbolTable

```rust
/// 调试符号表（从 JSON 反序列化）
#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub version: String,
    pub compiler_version: String,
    pub mapper: u8,
    pub prg_size: usize,
    pub chr_size: usize,
    pub variables: Vec<Variable>,
    pub constants: Vec<Constant>,
    pub functions: Vec<Function>,
    pub resources: Vec<Resource>,
    pub builtins: Vec<Builtin>,
    pub source_map: Vec<SourceMapEntry>,
}
```

（其他类型见 `data-model.md`，这里只列出公开的顶层结构）

---

## CLI Interface

CLI 子命令 `nesc decompile`：

```bash
# 基本用法（自动查找 hello.syms）
nesc decompile hello.nes

# 指定输出文件
nesc decompile hello.nes -o hello.dec.nesl

# 手动指定符号表路径
nesc decompile hello.nes --symbols custom.syms -o output.nesl

# 验证模式（只检查符号表是否有效，不输出源码）
nesc decompile hello.nes --validate

# 详细模式（显示反编译过程中的警告）
nesc decompile hello.nes -v
```

**Exit Codes**:
- `0`: 成功
- `1`: ROM 或符号表文件不存在
- `2`: ROM 格式错误或不是 NESL 编译器生成的
- `3`: 符号表格式错误或版本不支持
- `4`: 符号表与 ROM 不一致
- `5`: 反编译失败（无法识别的模式或其他错误）

---

## Examples

### Example 1: CLI 基本用法

```bash
$ nesc build hello.nesl --debug-symbols
# 输出: hello.nes, hello.syms

$ nesc decompile hello.nes -o hello.dec.nesl
# 输出: hello.dec.nesl

$ diff hello.nesl hello.dec.nesl
# 可能有格式差异（空白、注释），但语义相同
```

### Example 2: 库 API 用法

```rust
use nesl::decompile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 反编译
    let source = decompile::decompile("hello.nes", None)?;
    
    // 写入文件
    std::fs::write("hello.dec.nesl", source)?;
    
    // 验证（重新编译）
    let recompiled = nesl::compile(&source)?;
    assert_eq!(recompiled.mapper, 0);
    
    Ok(())
}
```

### Example 3: 错误处理

```rust
use nesl::decompile::{self, DecompileError};

match decompile::decompile("game.nes", None) {
    Ok(source) => println!("反编译成功:\n{}", source),
    Err(DecompileError::NotNeslRom) => {
        eprintln!("错误：此 ROM 不是 NESL 编译器生成的，无法反编译");
    }
    Err(DecompileError::SymbolsNotFound { path }) => {
        eprintln!("错误：找不到符号表文件: {}", path);
        eprintln!("提示：使用 `nesc build --debug-symbols` 生成符号表");
    }
    Err(e) => {
        eprintln!("反编译失败: {}", e);
    }
}
```

---

## Behavior Specifications

### 1. Symbol Table Discovery

当 `symbols_path` 为 `None` 时：
1. 如果 `rom_path` 是 `path/to/game.nes`，尝试读取 `path/to/game.syms`
2. 如果 `.syms` 文件不存在，返回 `SymbolsNotFound` 错误
3. 不尝试从 ROM 中提取嵌入的符号表（v1 不支持嵌入）

### 2. Validation

反编译前必须验证：
1. **ROM 格式**：前 4 字节是 `NES\x1A`
2. **Mapper 支持**：mapper 必须是 0/2/4
3. **符号表格式**：JSON 有效，包含所有必填字段
4. **版本兼容**：符号表 major version = 1
5. **一致性**：符号表的 mapper/prg_size/chr_size 与 ROM 头部一致

### 3. Error Reporting

- 错误信息必须清晰且可操作（告诉用户如何修复）
- 包含具体的地址、字段名、期望值 vs 实际值
- 对于常见错误（如符号表缺失），提供解决方法的提示

### 4. Output Format

反编译输出的 NESL 代码必须：
1. 通过 `nesl::parser::parse()` 语法检查
2. 顶层声明按顺序排列：`const` → `var` → `tile` → `map` → `music` → `fn`
3. 函数内部代码正确缩进（4 空格）
4. 包含头部注释：
   ```nesl
   // Decompiled from hello.nes
   // NESL Compiler: v0.1.0
   // Decompiled at: 2026-10-24 15:30:00
   ```

### 5. Unrecognized Patterns

遇到无法识别的代码模式时：
- 输出注释占位符：`// UNKNOWN CODE at $8050`
- 记录警告（在 verbose 模式下显示）
- 继续反编译其他部分（不因局部失败而整体失败）

---

## Testing Contract

### Unit Tests

```rust
#[test]
fn decompile_hello_example() {
    // 编译 hello.nesl
    let source = std::fs::read_to_string("examples/hello.nesl").unwrap();
    let out = nesl::compile(&source).unwrap();
    
    // 生成符号表（模拟编译器行为）
    let symbols = generate_symbols(&out);
    
    // 反编译
    let dec = decompile::decompile_from_bytes(&out.rom, &symbols).unwrap();
    
    // 验证输出是有效的 NESL 代码
    let reparsed = nesl::parser::parse(&dec).unwrap();
    assert!(reparsed.functions.iter().any(|f| f.name == "main"));
}
```

### Integration Tests

```rust
#[test]
fn roundtrip_all_examples() {
    for example in &["hello", "sprite", "game", "quest"] {
        let original = std::fs::read_to_string(format!("examples/{}.nesl", example)).unwrap();
        let out1 = nesl::compile(&original).unwrap();
        
        let symbols = generate_symbols(&out1);
        let decompiled = decompile::decompile_from_bytes(&out1.rom, &symbols).unwrap();
        let out2 = nesl::compile(&decompiled).unwrap();
        
        // 验证语义等价（在模拟器中运行相同步数，断言内存状态）
        assert_roms_equivalent(&out1.rom, &out2.rom);
    }
}
```

---

## Future Extensions (v2)

计划在未来版本中支持：
1. **嵌入式符号表**：符号表可选地嵌入 ROM（PRG-ROM 末尾 + 魔数）
2. **增量反编译**：只反编译修改过的函数（diff 模式）
3. **交互式反编译**：CLI 交互模式，用户可以选择性地反编译函数
4. **注释保留**：符号表包含原始源码的注释，反编译时还原

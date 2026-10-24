# Data Model: NESL Decompiler

**Date**: 2026-10-24

**Purpose**: 定义反编译器的核心数据结构

---

## 1. 调试符号表（Debug Symbol Table）

符号表是编译器输出的元数据文件（JSON 格式），包含反编译所需的所有信息。

### 1.1 顶层结构

```rust
struct SymbolTable {
    version: String,           // 符号表格式版本（如 "1.0.0"）
    compiler_version: String,  // NESL 编译器版本（如 "0.1.0"）
    mapper: u8,                // Mapper 类型（0=NROM, 2=UNROM, 4=MMC3）
    prg_size: usize,           // PRG-ROM 大小（字节）
    chr_size: usize,           // CHR-ROM 大小（字节）
    
    variables: Vec<Variable>,  // 全局变量表
    constants: Vec<Constant>,  // 常量表
    functions: Vec<Function>,  // 函数表
    resources: Vec<Resource>,  // 资源表（tile/map/music）
    builtins: Vec<Builtin>,    // 内建函数地址映射
    source_map: Vec<SourceMapEntry>,  // 源码行号映射
}
```

### 1.2 Variable（全局变量）

```rust
struct Variable {
    name: String,         // 变量名（如 "x"）
    ty: Type,             // 类型（u8/u16/i8/数组）
    addr: u16,            // RAM 地址（如 0x0300）
    init_value: Option<Value>,  // 初始值（如果有）
}

enum Type {
    U8,
    U16,
    I8,
    Array { element_ty: Box<Type>, length: usize },
    Struct { name: String, fields: Vec<(String, Type)> },
}

enum Value {
    U8(u8),
    U16(u16),
    I8(i8),
    Array(Vec<Value>),
}
```

**示例**：
```json
{
  "name": "score",
  "ty": "u16",
  "addr": "0x0310",
  "init_value": null
}
```

### 1.3 Constant（常量/ROM 数据）

```rust
struct Constant {
    name: String,         // 常量名（如 "PAL"）
    ty: Type,             // 类型（通常是 [u8] 或 [u16]）
    addr: u16,            // PRG-ROM 地址（如 0x8100）
    data: Vec<u8>,        // 数据内容
}
```

**示例**：
```json
{
  "name": "PAL",
  "ty": "[u8]",
  "addr": "0x8100",
  "data": [15, 48, 22, 39, ...]
}
```

### 1.4 Function（函数）

```rust
struct Function {
    name: String,              // 函数名（如 "main"）
    addr: u16,                 // 函数入口地址（如 0x8000）
    params: Vec<(String, Type)>,  // 参数列表
    return_ty: Option<Type>,   // 返回类型（None = void）
    local_vars: Vec<Variable>, // 局部变量（如果有）
}
```

**示例**：
```json
{
  "name": "add",
  "addr": "0x8050",
  "params": [["a", "u8"], ["b", "u8"]],
  "return_ty": "u8",
  "local_vars": []
}
```

### 1.5 Resource（资源：tile/map/music/metasprite）

```rust
struct Resource {
    name: String,         // 资源名（如 "tile_1"）
    resource_ty: ResourceType,
    location: ResourceLocation,
}

enum ResourceType {
    Tile,
    Map { width: u8, height: u8 },
    Music { channel: MusicChannel },
    Metasprite,
    Sound,
}

enum ResourceLocation {
    ChrRom { offset: usize, length: usize },  // CHR-ROM 中的位置
    PrgRom { addr: u16, length: usize },      // PRG-ROM 中的位置（map/music）
    Bank { bank: u8, addr: u16, length: usize },  // UNROM/MMC3 的 bank
}
```

**示例（tile）**：
```json
{
  "name": "tile_1",
  "resource_ty": "Tile",
  "location": { "ChrRom": { "offset": 16, "length": 16 } }
}
```

**示例（map，UNROM）**：
```json
{
  "name": "W1",
  "resource_ty": { "Map": { "width": 128, "height": 14 } },
  "location": { "Bank": { "bank": 0, "addr": "0x8000", "length": 1792 } }
}
```

### 1.6 Builtin（内建函数映射）

```rust
struct Builtin {
    name: String,         // 内建函数名（如 "__sprite"）
    addr: u16,            // 运行时地址（如 0xC000）
    signature: String,    // 函数签名（用于生成调用）
}
```

**示例**：
```json
{
  "name": "__sprite",
  "addr": "0xC000",
  "signature": "sprite(i: u8, x: u8, y: u8, tile: u8, attr: u8)"
}
```

### 1.7 SourceMapEntry（源码行号映射）

```rust
struct SourceMapEntry {
    nes_addr: u16,        // 6502 代码地址（如 0x8010）
    source_line: usize,   // 源码行号（从 1 开始）
    source_file: String,  // 源文件名（通常是主文件）
}
```

**示例**：
```json
{
  "nes_addr": "0x8010",
  "source_line": 15,
  "source_file": "hello.nesl"
}
```

---

## 2. 反编译器内部数据结构

### 2.1 DecompileContext（反编译上下文）

```rust
struct DecompileContext {
    symbols: SymbolTable,     // 符号表
    rom: RomImage,            // ROM 镜像
    ast: Vec<TopLevel>,       // 重建的 AST
    errors: Vec<DecompileError>,  // 错误列表
}

struct RomImage {
    header: InesHeader,       // iNES 头部
    prg_rom: Vec<u8>,         // PRG-ROM 内容
    chr_rom: Vec<u8>,         // CHR-ROM 内容
}
```

### 2.2 AST（抽象语法树，反编译输出）

反编译器重建的 AST 与编译器的 AST 结构相同（或简化版本）。

```rust
enum TopLevel {
    Const { name: String, ty: Type, value: ConstValue },
    Var { name: String, ty: Type, init: Option<Expr> },
    Function { name: String, params: Vec<(String, Type)>, return_ty: Option<Type>, body: Block },
    Tile { index: u8, pixels: [[u8; 8]; 8] },
    Map { name: String, width: u8, rows: Vec<String> },
    Music { name: String, channel: MusicChannel, mml: String },
}

enum ConstValue {
    Int(i64),
    Array(Vec<u8>),
    String(String),
}

struct Block {
    stmts: Vec<Stmt>,
}

enum Stmt {
    Let { name: String, ty: Option<Type>, init: Expr },
    Assign { target: Expr, value: Expr },
    If { cond: Expr, then: Block, else_: Option<Block> },
    While { cond: Expr, body: Block },
    For { var: String, start: Expr, end: Expr, body: Block },
    Loop { body: Block },
    Break,
    Continue,
    Return { value: Option<Expr> },
    Expr(Expr),
}

enum Expr {
    Literal(Value),
    Var(String),
    BinOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Call { func: String, args: Vec<Expr> },
    Index { base: Box<Expr>, index: Box<Expr> },
    Cast { expr: Box<Expr>, ty: Type },
}

enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or, BitAnd, BitOr, BitXor,
    Shl, Shr,
}
```

### 2.3 DecompileError（反编译错误）

```rust
enum DecompileError {
    MissingSymbols { reason: String },
    CorruptedRom { addr: u16, reason: String },
    UnsupportedMapper { mapper: u8 },
    UnrecognizedPattern { addr: u16, desc: String },
    VersionMismatch { expected: String, found: String },
}
```

---

## 3. 状态转换

反编译器的工作流程：

```text
1. Load ROM + Symbols
   ├─ Parse iNES header (mapper, PRG/CHR size)
   ├─ Load PRG-ROM and CHR-ROM into memory
   ├─ Read .syms file (or extract from ROM if embedded)
   └─ Validate symbol table version

2. Build AST
   ├─ Restore global variables (from symbols.variables)
   ├─ Restore constants (from symbols.constants)
   ├─ Restore resources (tile/map/music from symbols.resources)
   ├─ For each function in symbols.functions:
   │   ├─ Disassemble 6502 instructions (from addr to RTS/JMP)
   │   ├─ Recognize builtin calls (JSR addr → check symbols.builtins)
   │   ├─ Rebuild control flow (if/while/for from branch patterns)
   │   └─ Rebuild expressions (from register operations)
   └─ Handle errors (unrecognized patterns → placeholder/comment)

3. Generate Source
   ├─ Format AST as NESL source code
   ├─ Add header comment (decompiled from X, timestamp)
   ├─ Sort top-level declarations (const → var → tile → map → fn)
   └─ Pretty-print (indentation, spacing)

4. Validate Output
   ├─ Check syntax (run through parser)
   └─ Report warnings (unrecognized code, missing symbols)
```

---

## 4. 数据流图

```text
[ROM + .syms]
     ↓
[SymbolLoader]
     ↓
[SymbolTable]
     ↓
[RomParser] → [6502 Disassembler]
     ↓
[AstBuilder] ← [Pattern Matchers: if/while/for, builtin calls]
     ↓
[AST]
     ↓
[SourceCodeGenerator]
     ↓
[NESL Source Code]
```

---

## 5. 验证规则

### 5.1 符号表完整性

- 每个符号表必须包含 `version`、`mapper`、`prg_size` 字段
- 变量地址不重叠：`addr` + `sizeof(ty)` 不与其他变量冲突
- 函数地址在 PRG-ROM 范围内
- 资源位置在 ROM 边界内

### 5.2 ROM 一致性

- iNES 头部的 mapper 与符号表的 `mapper` 匹配
- PRG/CHR 大小与头部声明一致
- 中断向量（NMI/RESET/IRQ）指向有效地址

### 5.3 反编译输出

- 生成的 NESL 代码通过语法检查（`nesl::parser::parse()`）
- 无未定义的变量或函数引用
- 类型一致（不能把 u16 赋值给 u8 变量）

---

## Summary

核心数据结构：
- **SymbolTable**：编译器生成的元数据（JSON 序列化）
- **DecompileContext**：反编译器的内部状态
- **AST**：重建的语法树（与编译器 AST 兼容）
- **DecompileError**：错误分类和报告

关键约束：
- 符号表必须完整且与 ROM 一致
- 反编译输出必须是有效的 NESL 代码
- 错误优雅处理（不输出错误代码）

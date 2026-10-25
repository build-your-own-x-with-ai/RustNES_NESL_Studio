# Contract: Debug Symbol Table Format

**Version**: 1.0.0  
**Date**: 2026-10-24

## Overview

调试符号表是 NESL 编译器生成的 JSON 格式元数据文件，包含反编译所需的所有信息。文件扩展名为 `.syms`。

---

## JSON Schema

### Top-Level Structure

```json
{
  "version": "1.0.0",
  "compiler_version": "0.1.0",
  "mapper": 0,
  "prg_size": 32768,
  "chr_size": 8192,
  "variables": [...],
  "constants": [...],
  "functions": [...],
  "resources": [...],
  "builtins": [...],
  "source_map": [...]
}
```

**Fields**:
- `version` (string, required): 符号表格式版本，遵循 SemVer（如 "1.0.0"）
- `compiler_version` (string, required): NESL 编译器版本（如 "0.1.0"）
- `mapper` (integer, required): Mapper 类型（0=NROM, 2=UNROM, 4=MMC3）
- `prg_size` (integer, required): PRG-ROM 大小（字节），必须与 iNES 头部一致
- `chr_size` (integer, required): CHR-ROM 大小（字节）
- `variables` (array, required): 全局变量列表
- `constants` (array, required): 常量列表
- `functions` (array, required): 函数列表
- `resources` (array, required): 资源列表（tile/map/music）
- `builtins` (array, required): 内建函数地址映射
- `source_map` (array, optional): 源码行号映射

---

### Variable Object

```json
{
  "name": "score",
  "ty": "u16",
  "addr": "0x0310",
  "init_value": null
}
```

**Fields**:
- `name` (string, required): 变量名，必须是有效的 NESL 标识符
- `ty` (string or object, required): 类型，可以是 `"u8"` / `"u16"` / `"i8"` 或数组/结构体
- `addr` (string, required): RAM 地址（十六进制字符串，如 `"0x0300"`）
- `init_value` (value or null, optional): 初始值，null 表示默认初始化为 0

**Type Format**:
- 简单类型：`"u8"` / `"u16"` / `"i8"`
- 数组：`{ "Array": { "element_ty": "u8", "length": 16 } }`
- 结构体：`{ "Struct": { "name": "Enemy", "fields": [["x", "u8"], ["y", "u8"]] } }`

**Value Format**:
- 整数：`123`
- 数组：`[15, 48, 22, 39]`

---

### Constant Object

```json
{
  "name": "PAL",
  "ty": "[u8]",
  "addr": "0x8100",
  "data": [15, 48, 22, 39, 15, 48, 22, 39, ...]
}
```

**Fields**:
- `name` (string, required): 常量名
- `ty` (string, required): 类型，通常是 `"[u8]"` 或 `"[u16]"`
- `addr` (string, required): PRG-ROM 地址（十六进制）
- `data` (array of integers, required): 数据内容

---

### Function Object

```json
{
  "name": "main",
  "addr": "0x8000",
  "params": [],
  "return_ty": null,
  "local_vars": []
}
```

**Fields**:
- `name` (string, required): 函数名
- `addr` (string, required): 函数入口地址（十六进制）
- `params` (array, required): 参数列表，每项为 `[name, type]` 对
- `return_ty` (string or null, required): 返回类型，null 表示 void
- `local_vars` (array, optional): 局部变量列表（格式同 Variable）

**Example with parameters**:
```json
{
  "name": "add",
  "addr": "0x8050",
  "params": [["a", "u8"], ["b", "u8"]],
  "return_ty": "u8",
  "local_vars": []
}
```

---

### Resource Object

```json
{
  "name": "tile_1",
  "resource_ty": "Tile",
  "location": {
    "ChrRom": { "offset": 16, "length": 16 }
  }
}
```

**Fields**:
- `name` (string, required): 资源名
- `resource_ty` (string or object, required): 资源类型
- `location` (object, required): 资源在 ROM 中的位置

**Resource Types**:
- Tile: `"Tile"`
- Map: `{ "Map": { "width": 128, "height": 14 } }`
- Music: `{ "Music": { "channel": "Pulse1" } }`（Pulse1/Pulse2/Triangle）
- Metasprite: `"Metasprite"`
- Sound: `"Sound"`

**Location Types**:
- CHR-ROM: `{ "ChrRom": { "offset": 16, "length": 16 } }`
- PRG-ROM: `{ "PrgRom": { "addr": "0x8100", "length": 1792 } }`
- Banked ROM (UNROM/MMC3): `{ "Bank": { "bank": 0, "addr": "0x8000", "length": 1792 } }`

---

### Builtin Object

```json
{
  "name": "__sprite",
  "addr": "0xC000",
  "signature": "sprite(i: u8, x: u8, y: u8, tile: u8, attr: u8)"
}
```

**Fields**:
- `name` (string, required): 内建函数内部名（以 `__` 开头）
- `addr` (string, required): 运行时地址（十六进制）
- `signature` (string, required): 函数签名，用于生成调用代码

---

### SourceMapEntry Object

```json
{
  "nes_addr": "0x8010",
  "source_line": 15,
  "source_file": "hello.nesl"
}
```

**Fields**:
- `nes_addr` (string, required): 6502 代码地址（十六进制）
- `source_line` (integer, required): 源码行号（从 1 开始）
- `source_file` (string, required): 源文件名

---

## Example: Complete Symbol Table

```json
{
  "version": "1.0.0",
  "compiler_version": "0.1.0",
  "mapper": 0,
  "prg_size": 32768,
  "chr_size": 8192,
  
  "variables": [
    { "name": "x", "ty": "u8", "addr": "0x0300", "init_value": 120 },
    { "name": "score", "ty": "u16", "addr": "0x0310", "init_value": null }
  ],
  
  "constants": [
    {
      "name": "PAL",
      "ty": "[u8]",
      "addr": "0x8100",
      "data": [15, 48, 22, 39, 15, 48, 22, 39, 15, 48, 22, 39, 15, 48, 22, 39,
               15, 22, 39, 48, 15, 22, 39, 48, 15, 22, 39, 48, 15, 22, 39, 48]
    }
  ],
  
  "functions": [
    {
      "name": "main",
      "addr": "0x8000",
      "params": [],
      "return_ty": null,
      "local_vars": []
    }
  ],
  
  "resources": [
    {
      "name": "tile_1",
      "resource_ty": "Tile",
      "location": { "ChrRom": { "offset": 16, "length": 16 } }
    }
  ],
  
  "builtins": [
    { "name": "__sprite", "addr": "0xC000", "signature": "sprite(i: u8, x: u8, y: u8, tile: u8, attr: u8)" },
    { "name": "__print", "addr": "0xC030", "signature": "print(x: u8, y: u8, text: &[u8])" },
    { "name": "__load_palette", "addr": "0xC060", "signature": "load_palette(data: &[u8])" }
  ],
  
  "source_map": [
    { "nes_addr": "0x8000", "source_line": 1, "source_file": "hello.nesl" },
    { "nes_addr": "0x8010", "source_line": 15, "source_file": "hello.nesl" }
  ]
}
```

---

## Validation Rules

### 1. Version Compatibility
- 反编译器只支持符号表版本 `1.x.x`（major version = 1）
- Minor/patch 版本向后兼容

### 2. Address Ranges
- 变量地址必须在 RAM 范围内（`0x0000-0x07FF`，或扩展 RAM 区域）
- 函数地址必须在 PRG-ROM 范围内（NROM: `0x8000-0xFFFF`）
- 资源位置必须在 PRG/CHR-ROM 边界内

### 3. No Address Overlap
- 变量地址不重叠：`addr + sizeof(ty)` 不与其他变量冲突
- 常量/资源在 ROM 中不重叠（同一地址只有一个定义）

### 4. Type Consistency
- 数组长度 > 0
- 结构体字段名不重复

### 5. Mapper Consistency
- 符号表的 `mapper` 字段必须与 ROM 的 iNES 头部一致
- Banked 资源（`location.Bank`）只在 UNROM/MMC3 中出现

---

## Error Handling

反编译器遇到以下情况时必须报错：

1. **Missing Required Field**: 缺少 `version`、`mapper` 等必填字段
2. **Version Mismatch**: 符号表 major version ≠ 1
3. **Invalid Address**: 地址超出 ROM/RAM 范围
4. **Address Overlap**: 变量或资源地址冲突
5. **Mapper Mismatch**: 符号表的 mapper 与 ROM 头部不一致
6. **Corrupted Data**: JSON 解析失败

警告（不阻止反编译）：
1. **Missing Source Map**: `source_map` 字段缺失（影响调试体验）
2. **Unknown Builtin**: 符号表中的内建函数不在反编译器的已知列表中
3. **Incomplete Symbols**: 某些函数或变量缺少符号信息（用占位符）

---

## File Naming Convention

- 符号表文件名与 ROM 文件同名，扩展名为 `.syms`
- 例如：`hello.nes` → `hello.syms`

---

## Backwards Compatibility

- 新增字段（如未来的 `debug_info`）时，旧版本反编译器忽略它们
- 修改现有字段格式时，增加 major version

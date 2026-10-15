# Implementation Plan: NESL 语言增强

**Branch**: `002-language-extensions` | **Date**: 2026-10-15 | **Spec**: [spec.md](./spec.md)

## Summary

在 001 的编译器上增量扩展：类型系统加入 `I8` 与带元素类型的数组；结构体在**解析阶段降级**为带点号的
扁平符号（`boss.x` 标量、`enemies.x` 数组，SoA 布局），codegen 几乎复用现有变量/数组路径；
`match` 在 codegen 中降级为「临时变量 + 条件跳转链」，复用现有 `cond()`（因此自动支持有符号比较）。

## Technical Context

与 001 相同（Rust 1.99、零依赖、cargo test + nes-sim 端到端）。

## Constitution Check

| 原则 | 结论 |
|---|---|
| I 高级语言优先 | ✅ 消除平行数组与手工符号处理 |
| II 测试先行 | ✅ 每个用户故事先写失败的端到端测试 |
| III 库优先 | ✅ 仅改 `nesl` crate（编辑器高亮加关键字） |
| V 简单 | ✅ SoA 降级避免在 6502 上做下标乘法 |

## Design

### 类型
- `Type = U8 | I8 | U16`；`VarType = Scalar(T) | Array(T, len) | Struct(name) | StructArray(name, len)`。
- 自然类型合并：有 U16 → U16；U8 与 I8 混合算术 → U8（位等价）；I8 与 I8 → I8。
- 比较：任一侧为 I8（另一侧为 I8 或常量）→ 有符号比较；I8 与非常量 U8/U16 → 错误。
- 加宽：U8→U16 `LDX #0`；I8→U16 符号扩展 `LDX #0; CMP #$80; BCC +; DEX`。
- 有符号比较：`SEC; SBC r; BVC +2; EOR #$80` → N 标志 = (l < r)；`>`/`<=` 交换操作数。
- 常量适配目标类型：check_fits 对常量只做范围检查。

### 数组
- `Sym::Array { addr, len, elem }`、`Sym::Data { label, len, elem }`；u16 元素拆成两张表：低字节在 base，高字节在 base+len。
- u16 元素读取用 Y 下标：`LDY i; LDA lo,Y; LDX hi,Y`。

### 结构体（SoA 降级）
- 解析器把 `a.f` 产生为 `Var("a.f")`，`a[i].f` 产生为 `Index("a.f", i)`；`Place` 同理。
- 声明 `var boss: Enemy` 时注册 `boss` = `Sym::Struct`（用于报错）与每个字段 `boss.f`；
  `var es: [Enemy; 8]` 注册 `es` = `Sym::StructArray` 与每个字段数组 `es.f`（长度 8）。
- 未知字段：查找失败且名字含点时，报 `no field ...`。

### 局部数组/结构体
- `let x: [T; N];` / `let s: S;` 静态分配，声明处生成清零代码（`LDA #0; LDX #n; STA base-1,X; DEX; BNE`）。

### match
- 被匹配值求值一次存入隐藏局部 `$m<N>`（名字不可被源码写出），每个分支生成 `cond(pat1 || pat2 ...)`，
  区间降级为 `$m >= a && $m <= b`。

## Project Structure

```text
crates/nesl/src/ast.rs        Type::I8、VarType、StructDef、Stmt::Match、Pattern
crates/nesl/src/lexer.rs      `.` 符号、struct/match 关键字
crates/nesl/src/parser.rs     struct 声明、字段访问降级、数组元素类型、let 无初始化、match
crates/nesl/src/codegen.rs    I8/数组元素类型/结构体注册/局部数组/match
crates/nesl/tests/lang2.rs    002 端到端测试
crates/nesc/web/index.html    高亮新增关键字
examples/shooter.nesl         使用 struct + i8 + match 的示例
```

## Complexity Tracking

无。

# Data Model: 编译器核心数据结构

| 实体 | 字段 | 说明 |
|---|---|---|
| `Token` | kind, line, col | 词法单元 |
| `Program` (AST) | consts, globals, tiles, functions | 顶层声明集合 |
| `Function` | name, params[(name, Type)], ret: Option<Type>, body: Block, pos | |
| `Stmt` | Let / Assign / IndexAssign / If / While / Loop / For / Break / Continue / Return / Expr | |
| `Expr` | Num / Str / Var / Index / Call / Unary / Binary / Cast，均带 pos | |
| `Type` | U8 / U16 / Array(len) / ConstArray(len) | |
| `Symbol` | Global{addr,ty} / Local{addr,ty} / Const(value) / Data{label,len} / Func{...} | 名称解析结果 |
| `asm::Item` | Label / Ins(op, operand) / Branch(cond, label) / Bytes | 汇编中间表示 |
| `Assembled` | prg: [u8; 32768], symbols: Map<label, addr>, listing | 汇编结果 |
| `Diagnostic` | line, col, message | 编译错误 |
| `CompileOutput` | rom: Vec<u8>, symbols, listing, prg_used | `compile()` 返回值 |

## 内存映射（CPU）
| 地址 | 用途 |
|---|---|
| `$00-$0F` | 运行时：tmp0-3、ptr、ptr2、frame、ppuctrl/ppumask 影子、scroll_x/y、rng |
| `$10-$15` | 内建函数参数槽 ARG0-ARG4、VRAM 队列长度 |
| `$16-$17` | NMI 专用指针（音乐引擎） |
| `$18-$FF` | 用户标量变量（溢出到 `$03A0+`） |
| `$0100-$01FF` | 硬件栈 |
| `$0200-$02FF` | OAM 影子（每帧 DMA） |
| `$0300-$039F` | VRAM 队列、音效/镜头/音乐状态（见各特性 plan.md） |
| `$03A0-$07FF` | 用户数组与溢出变量 |
| `$8000-$FFF9` | 代码 + 常量数据 |
| `$FFFA-$FFFF` | NMI / RESET / IRQ 向量 |

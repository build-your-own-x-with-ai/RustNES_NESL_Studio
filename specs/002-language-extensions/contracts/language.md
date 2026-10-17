# Contract: NESL 语言 v2 增量

在 [001 语言参考](../../001-nesl-compiler/contracts/language.md) 基础上新增：

## 结构体
```nesl
struct Enemy { x: u8, y: u8, vx: i8, hp: u8, score: u16 }   // 字段: u8 / i8 / u16，可尾随逗号

var boss: Enemy;                 // 结构体变量（清零）
var enemies: [Enemy; 8];         // 结构体数组（清零）

boss.hp = 10;
enemies[i].x += enemies[i].vx;
let e: Enemy;                    // 局部结构体，执行到此处时清零
```
- 不支持：嵌套结构体、整体赋值/比较、作为函数参数或返回值。

## 有符号 i8
```nesl
var vx: i8 = -2;
x += vx;              // u8 与 i8 加减 → u8（回绕），常用于速度
if vx < 0 { ... }     // 有符号比较
let w = vx as u16;    // 符号扩展：-2 → 0xFFFE
vx >> 1               // 算术右移
```
- i8 范围 -128..=127；常量 128..=255 视为位模式。
- `i8` 与非常量 `u8`/`u16` 比较需显式 `as`。
- `i8` 的 `* / %` 仅支持 2 的幂常量乘法。

## 数组元素类型
```nesl
var scores: [u16; 5];
var offs: [i8; 4];
const NOTES: [u16] = [0x1AB, 0x17C, 0x153];
const DIRS: [i8] = [-1, 0, 1, 0];
fn f() { let buf: [u8; 8]; }     // 局部数组，执行到此处时清零
```

## match
```nesl
match state {
    0 => title(),
    1 | 2 => { play(); }
    3..=9 => score += 1,
    _ => {}
}
```
- 模式：常量表达式、`|` 组合、`a..=b` 闭区间、`_`。按顺序匹配第一个命中的分支；被匹配表达式只求值一次。
- 分支体：块 `{ ... }`（后面逗号可省略），或单条语句后跟 `,`（最后一个分支可省略逗号）。

## 新关键字
`struct`、`match`、类型名 `i8`。

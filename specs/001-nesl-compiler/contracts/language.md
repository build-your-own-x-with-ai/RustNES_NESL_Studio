# Contract: NESL 语言参考 v1

## 词法
- 注释：`// 行注释`、`/* 块注释 */`
- 整数：`123`、`0xFF`、`0b1010`（可含 `_` 分隔）；字符：`'A'`（u8）；布尔：`true`=1、`false`=0
- 字符串：`"HELLO"`，支持 `\n \\ \" \0 \xNN`，仅可用于常量数组初始化和 `print`
- 标识符：`[A-Za-z_][A-Za-z0-9_]*`

## 顶层声明
```nesl
const SPEED = 2;                       // 编译期整数常量（可用常量表达式）
const PAL: [u8] = [0x0F, 0x30, ...];   // ROM 只读字节数组
const MSG: [u8] = "GAME OVER";         // 字符串常量数组
var x: u8 = 120;                       // 全局变量（初值必须是常量表达式），默认 0
var score: u16;
var buf: [u8; 16];                     // 全局 RAM 数组（清零）
tile 1 = [                             // 8x8 图块：. 或 0 = 透明/背景色，1/2/3 = 调色板颜色
  "..3333..",
  ".3....3.",
  ...共 8 行...
];
fn add(a: u8, b: u8) -> u8 { return a + b; }
fn main() { ... }                      // 必需，程序入口；返回后 CPU 停在空循环
```

## 类型
- `u8`（0-255）、`u16`（0-65535），算术回绕。比较结果为 `u8` 的 0/1。
- 整数字面量 ≤ 255 为 u8，否则 u16。二元运算中只要一侧为 u16，结果为 u16（u8 零扩展）。
- 运算宽度 = max(操作数类型, 目标类型)：`var w: u16 = s + s`（s: u8 = 250）结果为 500。
- 常量表达式按精确整数计算后再检查是否放得下目标类型（负数按补码回绕，如 `-1` → 255）。
- u8 → u16 隐式；u16 → u8 必须 `expr as u8`（取低字节）。
- 条件中任何非 0 值为真。

## 语句
```nesl
let v: u8 = expr;   let w = expr;      // 局部变量（类型可省略，按表达式推断）
v = e;  v += e;  -= *= /= %= &= |= ^= <<= >>=
arr[i] = e;   arr[i] += e;
if c { } else if c { } else { }
while c { }
loop { }
for i in a..b { }                      // i: u8，b 不包含，a/b 仅求值一次
break;  continue;  return;  return e;
f(x, y);                               // 表达式语句
```

## 运算符优先级（高 → 低，与 Rust 相同）
| 级别 | 运算符 |
|---|---|
| 后缀 | `f(..)` `a[i]` |
| 一元 | `-` `!` `~` |
| 转换 | `as` |
| 乘除 | `*` `/` `%`（u8 运行时例程；u16 不支持乘除，编译错误） |
| 加减 | `+` `-` |
| 移位 | `<<` `>>` |
| 位与 | `&` |
| 异或 | `^` |
| 位或 | `\|` |
| 比较 | `==` `!=` `<` `<=` `>` `>=` |
| 逻辑与 | `&&`（短路） |
| 逻辑或 | `\|\|`（短路） |

## 内建常量
`BTN_A=0x80 BTN_B=0x40 BTN_SELECT=0x20 BTN_START=0x10 BTN_UP=0x08 BTN_DOWN=0x04 BTN_LEFT=0x02 BTN_RIGHT=0x01`

## 内建函数
| 函数 | 说明 |
|---|---|
| `wait_vblank()` | 等待下一帧 NMI（OAM DMA 已完成，适合少量 VRAM 写入） |
| `ppu_on()` / `ppu_off()` | 打开（背景+精灵+NMI）/ 关闭渲染 |
| `load_palette(PAL)` | 将常量数组（≤ 32 字节）写入调色板 `$3F00` 起 |
| `vram_addr(addr: u16)` | 设置 VRAM 写地址（自动恢复滚动） |
| `vram_put(v: u8)` | 写一个字节到 VRAM，地址 +1 |
| `vram_fill(v: u8, n: u16)` | 从当前地址连续写 n 个 v |
| `print(x: u8, y: u8, text)` | 在背景第 y 行第 x 列写文字；text 为字符串字面量或常量数组 |
| `print_u8(x: u8, y: u8, v: u8)` | 以 3 位十进制显示数字 |
| `sprite(i, x, y, tile, attr)` | 设置第 i 号精灵（0-63），下一帧生效 |
| `hide_sprite(i)` | 隐藏第 i 号精灵 |
| `joypad(port) -> u8` | 读取手柄（port 0/1），按位与 `BTN_*` 判断 |
| `scroll(x: u8, y: u8)` | 设置背景滚动 |
| `rand() -> u8` | 8 位伪随机数 |
| `frame_count() -> u8` | 帧计数器 |
| `peek(addr: u16) -> u8` / `poke(addr: u16, v: u8)` | 直接读写 CPU 地址空间 |
| `play_tone(period: u16)` / `stop_sound()` | 方波通道 1 播放/停止音调（period 为 11 位定时器值） |

> v2（[002-language-extensions](../../002-language-extensions/contracts/language.md)）新增：`struct`、`i8`、`[u16; N]` / `[i8; N]` 数组与常量表、局部数组、`match`。
> v3（[003-game-api](../../003-game-api/contracts/api.md)）新增：VRAM 队列（随时 `print`）、`set_tile`、`map`、`metasprite`、`collide`、`sfx`。
> v4（[004-scrolling](../../004-scrolling/contracts/api.md)）新增：最宽 255 列的地图、`camera_follow` / `camera_x`、`palette_zone`。

## 限制（v1）
- 不支持递归、指针、结构体、有符号数、u16 数组、u16 乘除。
- 数组下标为 u8，数组长度 ≤ 256。

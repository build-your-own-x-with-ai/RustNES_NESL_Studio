# 示例解读

`examples/` 下 8 个程序，从最小可运行到完整游戏，基本按语言版本递进。
在编辑器里用「示例」下拉可以直接载入任何一个。

| 示例 | 行数 | 演示什么 |
|---|---|---|
| `hello.nesl` | 16 | 最小可用程序：调色板 + `print` |
| `sprite.nesl` | 41 | 精灵移动与坐标显示 |
| `shooter.nesl` | 107 | `struct`、`i8`、`match`（v2） |
| `game.nesl` | 113 | 完整小游戏：接星星，速度与失误计数 |
| `maze.nesl` | 153 | 地图、元精灵、碰撞、音效（v3） |
| `runner.nesl` | 153 | 四屏横版卷轴、三声部音乐、状态栏（v4-v6） |
| `split.nesl` | 53 | MMC3 扫描线中断分屏（v13） |
| `quest.nesl` | 684 | 完整游戏：标题、六个世界、敌人、生命与计分 |

## hello —— 最小骨架

```nesl
const PAL: [u8] = [ /* 16 背景 + 16 精灵 */ ];

fn main() {
    load_palette(PAL);
    print(9, 3, "HELLO NES!");
    ppu_on();
    loop { wait_vblank(); }
}
```

三个不可省的步骤：初始化 → `ppu_on()` → 主循环里 `wait_vblank()`。

## sprite —— 输入与精灵

用方向键移动一个精灵，并把坐标用 `print_u8` 显示在屏幕上。
展示了 `joypad(0) & BTN_*` 的读法和 `sprite(i, x, y, tile, attr)`。

## shooter —— 结构体与有符号数

`struct` 管理一组陨石，`i8` 表示速度（可为负），`match` 处理游戏状态。

```nesl
struct Rock { x: u8, y: u8, vx: i8, vy: i8 }
var rocks: [Rock; 8];
rocks[i].x += rocks[i].vx;
```

## game —— 接星星

完整循环：随机落星、篮子移动、每 5 个加速、漏 3 个结束、按 START 重开。
可以看一个真实游戏怎么组织 `loop { wait_vblank(); ... }`。

## maze —— 地图与碰撞

字符地图做墙，`metasprite` 画 16×16 主角，`collide()` 判定，
`sfx(SFX_COIN)` 给反馈，`print` 随时显示分数。

## runner —— 卷轴、音乐、状态栏

最宽 255 列的地图 + `camera_follow` 跟随 + `palette_zone` 分区配色 +
MML 三声部 `play_music` + `status_bar` 固定分数栏。

## split —— MMC3 分屏

```nesl
fn irq() { split_scroll(0); }   // 顶部固定，下面滚动

fn main() {
    scanline_irq(24);
    ...
}
```

声明 `fn irq()` 后程序会编译成 MMC3 卡带类型。

## quest —— 完整游戏

COIN QUEST：标题画面、六个卷轴世界、可踩的史莱姆、金币、生命与计分。
想看"这些 API 组合起来能做出什么"，直接读这一个文件。

## 编译它们

```bash
./target/release/nesc build examples/quest.nesl        # → examples/quest.nes
./target/release/nesc build examples/quest.nesl --asm  # 同时输出汇编清单
```

`cargo test` 会把这些示例编译后在 `nes-sim` 里真正执行，并用 jsnes 交叉验证画面
（见 [参与开发](Development#交叉验证)）。

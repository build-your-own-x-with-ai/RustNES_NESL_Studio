# Contract: 游戏开发 API（v3 增量）

在 [001](../../001-nesl-compiler/contracts/language.md) 与 [002](../../002-language-extensions/contracts/language.md) 基础上新增。

## 随时写屏（VRAM 队列）
渲染开启时，以下函数的写入会排队，在下一次 vblank 内完成；渲染关闭时立即写入：

| 函数 | 说明 |
|---|---|
| `print(x, y, text)` / `print_u8(x, y, v)` | 同 v1，但可在任何时刻调用 |
| `set_tile(x: u8, y: u8, tile: u8)` | 写单个背景格（新） |
| `load_palette(PAL)` | 渲染中调用可做淡入淡出 |

- 一帧最多约 90 字节的排队数据；超出时函数会自动等待一帧再继续（不会丢失）。
- `vram_addr / vram_put / vram_fill` 仍然直接写入，只应在 `ppu_off()` 期间使用。

## 地图
```nesl
map LEVEL1 = [
    "################################",
    "#..............................#",
    "#....####..........#####.......#",
    "################################",
];
tile '#' = [ ...8 行... ];            // 用 tile 重新定义 '#' 的样子

draw_map(LEVEL1);                      // 画满整屏（命名表 0），属性表清零
if map_tile(LEVEL1, x >> 3, y >> 3) == '#' { ... }   // 查询：越界返回 0
```
- 最多 30 行 × 32 列；每个字符的图块编号 = 它的 ASCII 码，`.` 视为空格；不足部分补空格。
- `draw_map` 在渲染中调用时会关闭渲染一帧再恢复。

## 元精灵
```nesl
metasprite HERO = [(0, 0, 1), (8, 0, 2), (0, 8, 3), (8, 8, 4, 0x40)];   // (dx, dy, tile[, attr])
let next = metasprite(0, x, y, HERO);  // 从 0 号精灵开始画，返回下一个空槽位
```
- dx/dy 可为负；移出屏幕边缘的块自动隐藏。

## 碰撞
```nesl
if collide(px, py, 16, 16, ex, ey, 8, 8) { ... }   // 两个矩形是否重叠 (u8 参数)
```
- 按 9 位精度比较，`x + w` 超过 255 也正确；每个参数只求值一次。

## 音效
```nesl
sfx(SFX_JUMP);   // SFX_JUMP SFX_COIN SFX_SHOOT SFX_HIT SFX_EXPLODE
```
- 每帧由 NMI 自动推进并在结束时静音；与 `play_tone` 同时发声；`stop_sound()` 停止全部。

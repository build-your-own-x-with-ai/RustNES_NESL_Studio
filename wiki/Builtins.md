# 内建函数参考

编译器当前共 **37** 个内建函数、**13** 个常量。
本页由 `crates/nesl/src/builtins.rs` 中的 `DOCS` / `CONSTANTS` 表生成，与编辑器里的补全、悬浮提示同源。

## 目录
- [帧与画面](#帧与画面) → `wait_vblank`、`ppu_on`、`ppu_off`、`frame_count`、`rand`
- [调色板与显存](#调色板与显存) → `load_palette`、`vram_addr`、`vram_put`、`vram_fill`
- [文字与背景格](#文字与背景格) → `print`、`print_u8`、`print_u16`、`set_tile`
- [精灵](#精灵) → `sprite`、`hide_sprite`、`metasprite`
- [输入](#输入) → `joypad`
- [地图](#地图) → `draw_map`、`map_tile`、`set_map_tile`、`reset_map`
- [卷轴与镜头](#卷轴与镜头) → `scroll`、`camera_follow`、`camera_x`、`split_scroll`
- [状态栏](#状态栏) → `status_bar`
- [碰撞](#碰撞) → `collide`
- [音频](#音频) → `play_tone`、`stop_sound`、`sfx`、`play_music`、`stop_music`
- [存档](#存档) → `save_game`、`load_game`
- [中断（MMC3）](#中断（mmc3）) → `scanline_irq`
- [底层内存](#底层内存) → `peek`、`poke`

## 帧与画面

| 函数 | 签名 | 说明 |
|---|---|---|
| `wait_vblank` | `wait_vblank()` | 等待下一帧开始（每帧调用一次，游戏主循环的节拍） |
| `ppu_on` | `ppu_on()` | 打开画面（背景 + 精灵） |
| `ppu_off` | `ppu_off()` | 关闭画面；之后可以大量写显存 |
| `frame_count` | `frame_count() -> u8` | 开机以来的帧数（0-255 循环） |
| `rand` | `rand() -> u8` | 伪随机数 1-255 |

## 调色板与显存

| 函数 | 签名 | 说明 |
|---|---|---|
| `load_palette` | `load_palette(PAL)` | 载入调色板：最多 32 字节的常量数组（16 背景 + 16 精灵） |
| `vram_addr` | `vram_addr(addr: u16)` | 设置显存写入地址（底层；画面关闭时使用） |
| `vram_put` | `vram_put(v: u8)` | 写一个字节到显存当前地址（底层） |
| `vram_fill` | `vram_fill(v: u8, n: u16)` | 从当前地址起连续写 n 个 v（底层） |

## 文字与背景格

| 函数 | 签名 | 说明 |
|---|---|---|
| `print` | `print(x: u8, y: u8, text)` | 在背景第 y 行第 x 列写文字；任何时刻都可以调用 |
| `print_u8` | `print_u8(x: u8, y: u8, v: u8)` | 以 3 位十进制显示数字 |
| `print_u16` | `print_u16(x: u8, y: u8, v: u16)` | 以 5 位十进制显示数字（00000-65535），适合分数 |
| `set_tile` | `set_tile(x: u8, y: u8, tile: u8)` | 改写背景上的一格 |

## 精灵

| 函数 | 签名 | 说明 |
|---|---|---|
| `sprite` | `sprite(i, x, y, tile, attr)` | 设置第 i 号精灵（0-63），左上角出现在 (x, y)（y = 0 无法显示）；attr 低 2 位选精灵调色板，$40 水平翻转，$80 垂直翻转，$20 放在背景后 |
| `hide_sprite` | `hide_sprite(i: u8)` | 隐藏第 i 号精灵 |
| `metasprite` | `metasprite(slot, x, y, M) -> u8` | 从第 slot 号精灵起画多块精灵 M，返回下一个空槽位 |

## 输入

| 函数 | 签名 | 说明 |
|---|---|---|
| `joypad` | `joypad(port: u8) -> u8` | 读手柄（0 或 1），用 & BTN_A 等判断按键 |

## 地图

| 函数 | 签名 | 说明 |
|---|---|---|
| `draw_map` | `draw_map(MAP)` | 画出地图（最左两屏），镜头归零 |
| `map_tile` | `map_tile(MAP, tx: u16, ty: u8) -> u8` | 读地图第 tx 列第 ty 行的字符；越界返回 0；能看到 set_map_tile 的修改 |
| `set_map_tile` | `set_map_tile(MAP, tx: u16, ty: u8, tile: u8) -> u8` | 改写地图一格（画面和卷轴都会更新）；返回 0 表示修改记录已满（最多 64 处） |
| `reset_map` | `reset_map(MAP)` | 撤销这张地图的全部修改 |

## 卷轴与镜头

| 函数 | 签名 | 说明 |
|---|---|---|
| `scroll` | `scroll(x: u8, y: u8)` | 设置背景滚动（不用 camera_follow 时） |
| `camera_follow` | `camera_follow(x: u16, MAP)` | 镜头跟随世界坐标 x（每帧调用），自动补画新露出的列 |
| `camera_x` | `camera_x() -> u16` | 镜头的世界坐标；屏幕 x = 世界 x - camera_x() |
| `split_scroll` | `split_scroll(x: u16)` | 只能在 fn irq() 里用：从当前行起水平卷轴改为 x |

## 状态栏

| 函数 | 签名 | 说明 |
|---|---|---|
| `status_bar` | `status_bar(rows)` | 顶部 rows 行（2-8，0 关闭）不随卷轴滚动；占用 0 号精灵 |

## 碰撞

| 函数 | 签名 | 说明 |
|---|---|---|
| `collide` | `collide(ax, ay, aw, ah, bx, by, bw, bh) -> u8` | 两个矩形是否重叠 |

## 音频

| 函数 | 签名 | 说明 |
|---|---|---|
| `play_tone` | `play_tone(period: u16)` | 方波 1 持续发出一个音（period 越大越低） |
| `stop_sound` | `stop_sound()` | 停止全部声音：音乐、音效和 play_tone |
| `sfx` | `sfx(id: u8)` | 播放音效：SFX_JUMP 等内置音效，或 sound 声明的名字 |
| `play_music` | `play_music(MELODY[, BASS[, HARMONY]])` | 播放 music 声明的音乐，1-3 个声部，自动循环 |
| `stop_music` | `stop_music()` | 停止音乐 |

## 存档

| 函数 | 签名 | 说明 |
|---|---|---|
| `save_game` | `save_game()` | 将全局变量写入存档（SRAM） |
| `load_game` | `load_game() -> u8` | 从存档载入变量；存档存在返回 1，否则返回 0 |

## 中断（MMC3）

| 函数 | 签名 | 说明 |
|---|---|---|
| `scanline_irq` | `scanline_irq(line: u8)` | 每帧在第 line 行调用 fn irq()（MMC3）；0 关闭 |

## 底层内存

| 函数 | 签名 | 说明 |
|---|---|---|
| `peek` | `peek(addr: u16) -> u8` | 读任意内存地址 |
| `poke` | `poke(addr: u16, v: u8)` | 写任意内存地址 |

## 内建常量

| 常量 | 值 | 说明 |
|---|---|---|
| `BTN_A` | `0x80` | 手柄按键位（与 `joypad()` 结果按位与） |
| `BTN_B` | `0x40` | 手柄按键位（与 `joypad()` 结果按位与） |
| `BTN_SELECT` | `0x20` | 手柄按键位（与 `joypad()` 结果按位与） |
| `BTN_START` | `0x10` | 手柄按键位（与 `joypad()` 结果按位与） |
| `BTN_UP` | `0x08` | 手柄按键位（与 `joypad()` 结果按位与） |
| `BTN_DOWN` | `0x04` | 手柄按键位（与 `joypad()` 结果按位与） |
| `BTN_LEFT` | `0x02` | 手柄按键位（与 `joypad()` 结果按位与） |
| `BTN_RIGHT` | `0x01` | 手柄按键位（与 `joypad()` 结果按位与） |
| `SFX_JUMP` | `0` | 内置音效编号，传给 `sfx()` |
| `SFX_COIN` | `1` | 内置音效编号，传给 `sfx()` |
| `SFX_SHOOT` | `2` | 内置音效编号，传给 `sfx()` |
| `SFX_HIT` | `3` | 内置音效编号，传给 `sfx()` |
| `SFX_EXPLODE` | `4` | 内置音效编号，传给 `sfx()` |

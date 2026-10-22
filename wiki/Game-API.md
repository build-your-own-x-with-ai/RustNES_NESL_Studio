# 游戏开发 API

按功能组织的用法说明，函数签名以 [内建函数参考](Builtins) 为准。

## 随时写屏（VRAM 队列）

渲染开启时，下面这些写入会**排队**，在下一次 vblank 内统一完成；渲染关闭时立即写入。
所以你不需要为了改一格背景而关画面。

```nesl
print(1, 1, "SCORE");       // 文字，任何时刻可调用
print_u8(8, 1, score);      // 3 位十进制
print_u16(8, 1, score);     // 5 位十进制（00000-65535）
set_tile(5, 10, 3);         // 改单个背景格
load_palette(PAL);          // 渲染中调用可做淡入淡出
```

- 一帧最多排队约 90 字节；超出时函数会自动等一帧再继续，**不会丢数据**。
- 只有底层的 `vram_addr` / `vram_put` / `vram_fill` 是直写，仍应在 `ppu_off()` 期间使用。

## 地图

```nesl
map LEVEL = [
    "################################",
    "#..............................#",
    "################################",
];
tile '#' = ["33333333", "3......3", /* … */];

draw_map(LEVEL);                                  // 画满整屏，镜头归零
if map_tile(LEVEL, x >> 3, y >> 3) == '#' { }     // 查询，越界返回 0
```

- 最多 30 行 × 255 列；图块编号 = 字符的 ASCII 码，`.` 视为空格。
- `draw_map` 在渲染中调用会关闭渲染一帧再恢复。
- 地图列表：`const WORLDS: [map] = [W1, W2, W3];` → `draw_map(WORLDS[level])`。

### 改写地图

```nesl
if map_tile(WORLD, cx, cy) == '$' {
    set_map_tile(WORLD, cx, cy, ' ');   // 读取、补画、屏幕都看到新值
    score += 1;
}
reset_map(WORLD);                       // 撤销全部修改
```

`set_map_tile` 返回 1 表示已记录，0 表示记录已满（**全部地图共用 64 处**）。
记录满时可见格子仍会更新，但卷走再卷回来会恢复原样。修改不会改动 ROM 里的地图本身。

## 精灵

```nesl
sprite(0, x, y, 1, 0);      // 第 0 号精灵，左上角在 (x, y)
hide_sprite(0);
```

- `attr` 低 2 位选精灵调色板，`$40` 水平翻转、`$80` 垂直翻转、`$20` 放到背景后。
- `y = 0` 无法显示（硬件限制）。
- 字体图块编号就是 ASCII 码，所以可以用精灵画数字：`sprite(i, x, y, '0' + d, 0)`。

### 元精灵

```nesl
metasprite HERO = [(0, 0, 1), (8, 0, 2), (0, 8, 3), (8, 8, 4)];
let next = metasprite(0, x, y, HERO);   // 从 0 号槽开始画，返回下一个空槽
```

dx/dy 可为负；移出屏幕的块自动隐藏。

## 输入

```nesl
if joypad(0) & BTN_A { ... }
if joypad(0) & BTN_RIGHT { x += 1; }
```

按键位：`BTN_A`=0x80 `BTN_B`=0x40 `BTN_SELECT`=0x20 `BTN_START`=0x10
`BTN_UP`=0x08 `BTN_DOWN`=0x04 `BTN_LEFT`=0x02 `BTN_RIGHT`=0x01。

## 碰撞

```nesl
if collide(px, py, 16, 16, ex, ey, 8, 8) { ... }
```

两个矩形是否重叠（u8 参数）。按 9 位精度比较，`x + w` 超过 255 也正确；每个参数只求值一次。

## 横向卷轴

```nesl
camera_follow(hero.x, WORLD);               // 每帧调用：跟随、两端停住、自动补画新列
let sx = hero.x - camera_x();               // 世界坐标 → 屏幕坐标（u16）
if sx < 256 { metasprite(0, sx as u8, hero.y, HERO); }
```

- 主角保持在屏幕 x≈120；镜头每帧最多移动 8 像素。只支持横向卷轴。
- 背景上的文字会跟着滚动；固定分数请用 `status_bar()` 或用精灵绘制。
- 不用镜头时可以用底层 `scroll(x, y)` 手动设置滚动。

## 状态栏

```nesl
status_bar(4);              // 顶部 4 行不随卷轴滚动（2-8 行；0 关闭）
print(1, 1, "SCORE");
```

- 地图从第 n 行以下开始绘制和卷动，`map` 的前 n 行不会显示。
- 占用**精灵 0** 和状态栏最后一行最右一格（第 31 列）；请从 1 号精灵开始用，不要 `tile 0x7E` / `tile 0x7F`。
- 状态栏使用背景调色板 0。

## 音乐

```nesl
music THEME = "T140 O4 L8 E G >C< G | A4 G4 R2";
music BASS  = ["O2 L4 C G C G", "A E F G"];
music HARM  = "O4 L4 G E F D";

play_music(THEME);               // 只有旋律
play_music(THEME, BASS);         // 旋律 + 低音
play_music(THEME, BASS, HARM);   // 旋律 + 低音 + 和声
stop_music();
```

三个声部分别是：旋律 = 方波 1，低音 = 三角波，和声 = 方波 2。播完自动循环，播放音效时暂时让出。

### MML 语法

| 写法 | 含义 | 默认 |
|---|---|---|
| `C D E F G A B` | 音符，后接 `+`/`#`（升）或 `-`（降），再接长度和附点 `.` | |
| `R` | 休止符 | |
| `4` `8` `16` … | 音符长度（1-64，4 = 四分音符） | `L4` |
| `L n` | 默认长度 | 4 |
| `O n` / `>` / `<` | 八度 0-7 / 升八度 / 降八度 | `O4` |
| `T n` | 速度（每分钟四分音符数，30-300） | `T120` |
| `V n` | 音量 0-15（方波声部） | `V12` |
| `@n` | 占空比 0-3（12.5% / 25% / 50% / 75%） | `@2` |
| `&` | 与下一个音符连奏 | |
| `\|`、空白 | 忽略（可当小节线） | |

每个音符最后一帧自动静音，让连续相同音符听得清；用 `&` 可取消。
旋律与和声最低 A1，低音最低 A0，最高 B7。

## 音效

```nesl
sfx(SFX_JUMP);      // 内置：JUMP / COIN / SHOOT / HIT / EXPLODE
sfx(LASER);         // 自定义 sound
```

每帧由 NMI 自动推进，结束时静音；与 `play_tone` 可同时发声。
自定义音效与内置音效共用引擎，**合计最多 32 个**。`stop_sound()` 停止一切声音。

## 存档

```nesl
if load_game() == 1 { /* 有存档，已恢复全局变量 */ } else { /* 首次开机 */ }
save_game();
```

数据存在电池供电的 SRAM（`$6000-$7FFF`）里，断电不丢。载入时校验魔数与校验和，
无效则走默认值。适合保存世界编号、分数、生命数。

## MMC3 分屏

```nesl
fn irq() { split_scroll(0); }        // 从当前行起水平卷轴设为 0

fn main() {
    scanline_irq(24);                // 每帧第 24 行触发 irq()；0 关闭
    ...
}
```

`fn irq()` 的存在会让程序编译为 MMC3（32KB PRG）。`split_scroll` 只能在 `irq()` 内调用，
`irq()` 不能被显式调用、也不能调用用户函数。典型效果是顶部几行固定做 HUD，下面区域滚动。

## 底层访问

```nesl
let v = peek(0x2002);      // 读任意 CPU 地址
poke(0x4016, 1);           // 写任意 CPU 地址
play_tone(0x1AB);          // 方波 1 持续发声，period 越大越低
```

`peek` / `poke` 是逃生舱口，正常游戏逻辑用不到。

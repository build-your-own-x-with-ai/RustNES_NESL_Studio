# RustNES / NESL Studio

**NESL** 是一门类 Rust 语法的小语言，`nesc` 编译器把它直接编译成可以在任何 NES 模拟器或烧录卡上运行的 `.nes` ROM —— **不需要写一行 6502 汇编**。

## 在线体验

打开 **<https://build-your-own-x-with-ai.github.io/RustNES_NESL_Studio/>** 就可以直接写代码、编译、在页面里玩。
整个编译器以 WebAssembly 形式跑在浏览器里，没有后端。

## 三十秒上手

```nesl
const PAL: [u8] = [0x0F,0x30,0x16,0x27, 0x0F,0x30,0x16,0x27, 0x0F,0x30,0x16,0x27, 0x0F,0x30,0x16,0x27,
                   0x0F,0x16,0x27,0x30, 0x0F,0x16,0x27,0x30, 0x0F,0x16,0x27,0x30, 0x0F,0x16,0x27,0x30];

var x: u8 = 124;

fn main() {
    load_palette(PAL);
    print(9, 3, "HELLO NES!");
    ppu_on();
    loop {
        wait_vblank();
        if joypad(0) & BTN_RIGHT { x += 1; }
        if joypad(0) & BTN_LEFT  { x -= 1; }
        sprite(0, x, 112, 1, 0);
    }
}
```

命令行下：

```bash
cargo build --release
./target/release/nesc build examples/game.nesl        # → examples/game.nes
```

## 能力一览

| 能力 | 写法 |
|---|---|
| 类型 | `u8`、`i8`、`u16`，`as` 转换（i8 → u16 符号扩展） |
| 变量 | `var score: u16;`（全局）、`let x = 3;`（局部） |
| 常量 / ROM 数据 | `const SPEED = 2;`、`const PAL: [u8] = [...];`、`const MSG: [u8] = "HI";` |
| 数组 | `var buf: [u8; 16];`、`var hi: [u16; 5];`、`const NOTES: [u16] = [...];` |
| 控制流 | `if / else if / else`、`while`、`loop`、`for i in 0..8`、`break`、`continue` |
| 函数 | `fn add(a: u8, b: u8) -> u8 { return a + b; }`（不支持递归） |
| 结构体 | `struct Enemy { x: u8, y: u8, vx: i8 }`、`var es: [Enemy; 8];` |
| `match` | `match state { 0 => title(), 1 \| 2 => play(), 3..=9 => {...} _ => {} }` |
| 图块 | `tile N = [8 行 × 8 字符，. 1 2 3]` |
| 地图 | `map LEVEL = ["####", "#..#"];` → `draw_map(LEVEL)`、`map_tile(...)` |
| 元精灵 | `metasprite HERO = [(0,0,1), (8,0,2), (0,8,3), (8,8,4)];` |
| 碰撞 / 音效 | `collide(...)`、`sfx(SFX_JUMP)` |
| 横向卷轴 | `camera_follow(hero.x, WORLD)`、`camera_x()`、`palette_zone` |
| 音乐 | `music THEME = "T150 O4 L8 C E G >C";` → `play_music(MELODY, BASS, HARMONY)` |
| 自定义音效 | `sound LASER = (pulse, 0x30, 4, 10);` → `sfx(LASER)` |
| 状态栏 | `status_bar(3)`：顶部若干行不随卷轴滚动 |
| 存档 | `save_game()` / `load_game()`（电池供电 SRAM） |
| MMC3 分屏 | 声明 `fn irq()` + `scanline_irq(line)` + `split_scroll(x)` |

内置 8×8 ASCII 字体，`print` 无需任何素材即可显示文字。

## 文档导航

- [安装与快速开始](Getting-Started) —— 构建、CLI、`nesc serve`
- [语言参考](Language-Reference) —— 完整语法与类型规则
- [内建函数参考](Builtins) —— 37 个函数 + 13 个常量
- [游戏开发 API](Game-API) —— 地图、精灵、卷轴、音乐、状态栏、存档、MMC3
- [NESL Studio 编辑器](Studio) —— 浏览器编辑器与页面内模拟器
- [示例解读](Examples) —— 8 个示例分别演示什么
- [架构](Architecture) —— crate 结构、编译流水线、wasm 与 Pages
- [参与开发](Development) —— 测试策略、交叉验证、Spec Kit

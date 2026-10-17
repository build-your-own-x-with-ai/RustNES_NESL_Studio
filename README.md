# RustNES — 用高级语言写 NES 游戏

**NESL** 是一门类 Rust 语法的小语言，`nesc` 编译器把它直接编译成可在任何 NES 模拟器 / 烧录卡上运行的 `.nes` ROM ——
**不需要写一行 6502 汇编**。附带浏览器编辑器：写代码、点运行、立即在页面里玩。

```nesl
const PAL: [u8] = [   // 4 组背景调色板 + 4 组精灵调色板
    0x0F,0x30,0x16,0x27, 0x0F,0x30,0x16,0x27, 0x0F,0x30,0x16,0x27, 0x0F,0x30,0x16,0x27,
    0x0F,0x16,0x27,0x30, 0x0F,0x16,0x27,0x30, 0x0F,0x16,0x27,0x30, 0x0F,0x16,0x27,0x30];

tile 1 = ["..2222..", ".222222.", "22322322", "22222222",
          "23222232", "22333322", ".222222.", "..2222.."];

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

## 快速开始

```bash
cargo build --release
./target/release/nesc build examples/game.nesl        # → examples/game.nes
./target/release/nesc build examples/game.nesl --asm  # 同时输出汇编清单 game.s
./target/release/nesc check examples/game.nesl        # 只检查
./target/release/nesc serve                           # 浏览器编辑器 http://127.0.0.1:8080
```

编辑器：左侧带语法高亮和行号的代码区，右侧是 jsnes 模拟器画面（点击画面后用键盘操作：方向键、`X`=A、`Z`=B、
`Enter`=Start、`Shift`=Select），下方显示编译错误（点击跳转到对应行）和汇编清单，可载入示例、下载 `.nes`。
`Ctrl/⌘+Enter` 编译并运行。
左侧还有「图块 / 地图 / 调色板」三个标签页：点格子画像素、画地图、从 NES 64 色里选颜色，修改会直接写回代码里对应的
`tile`、`map`、`load_palette` 数组（可撤销）。写代码时有内建函数补全（带参数）、鼠标悬停显示中文说明，停止输入半秒后自动检查并标出错误行。

## 语言速览

| 能力 | 写法 |
|---|---|
| 类型 | `u8`、`i8`（有符号）、`u16`，`as` 转换（i8 → u16 符号扩展） |
| 全局 / 局部 | `var score: u16;`、`let x = 3;` |
| 常量 / ROM 数据 | `const SPEED = 2;`、`const PAL: [u8] = [...];`、`const MSG: [u8] = "HI";` |
| 数组 | `var buf: [u8; 16];`、`var hi: [u16; 5];`、`const NOTES: [u16] = [...];`、函数内 `let tmp: [u8; 8];`（每次清零） |
| 控制流 | `if / else if / else`、`while`、`loop`、`for i in 0..8`、`break`、`continue` |
| 函数 | `fn add(a: u8, b: u8) -> u8 { return a + b; }`（不支持递归） |
| 图块 | `tile N = [8 行 × 8 像素，字符 . 1 2 3]` |
| 结构体 | `struct Enemy { x: u8, y: u8, vx: i8 }`、`var es: [Enemy; 8];` → `es[i].x += es[i].vx;` |
| match | `match state { 0 => title(), 1 \| 2 => play(), 3..=9 => {...} _ => {} }` |
| 随时写屏 | 渲染中调用 `print` / `print_u8` / `set_tile` / `load_palette` 自动排队到下一个 vblank，不会花屏 |
| 地图 | `map LEVEL = ["####", "#..#", ...];` → `draw_map(LEVEL)`、`map_tile(LEVEL, tx, ty)` |
| 元精灵 | `metasprite HERO = [(0,0,1), (8,0,2), (0,8,3), (8,8,4)];` → `metasprite(slot, x, y, HERO)` |
| 碰撞 / 音效 | `collide(ax, ay, aw, ah, bx, by, bw, bh)`、`sfx(SFX_JUMP)`（JUMP/COIN/SHOOT/HIT/EXPLODE） |
| 横向卷轴 | 地图最宽 255 列；`camera_follow(hero.x, WORLD)` 跟随并自动补画，`camera_x()` 换算屏幕坐标；`palette_zone WORLD = [...]` 分区配色 |
| 音乐 | `music THEME = "T150 O4 L8 C E G >C";` → `play_music(MELODY, BASS, HARMONY)`（方波 / 三角波 / 方波 2，自动循环）、`stop_music()` |
| 自定义音效 | `sound LASER = (pulse, 0x30, 4, 10);` → `sfx(LASER)`，与 `SFX_*` 共用 |
| 改写地图 | `set_map_tile(WORLD, tx, ty, ' ')`：读取、画面、卷轴补画都看到新值（最多 64 处）；`reset_map(WORLD)` 撤销 |
| 状态栏 | `status_bar(3)`：顶部 2-8 行不随卷轴滚动，直接 `print` 分数（占用 0 号精灵） |
| 地图列表 | `const WORLDS: [map] = [W1, W2, W3];` → `draw_map(WORLDS[level])`，凡是要地图的地方都能写 `WORLDS[i]` |
| 大数字 | `print_u16(x, y, score)` 显示 5 位数（00000-65535） |
| NES API | `wait_vblank ppu_on ppu_off load_palette vram_addr vram_put vram_fill print print_u8 sprite hide_sprite joypad scroll rand frame_count peek poke play_tone stop_sound`，按键常量 `BTN_A…BTN_RIGHT` |

完整参考：[`specs/001-nesl-compiler/contracts/language.md`](specs/001-nesl-compiler/contracts/language.md)，
v2 新增内容见 [`specs/002-language-extensions/contracts/language.md`](specs/002-language-extensions/contracts/language.md)，
游戏开发 API 见 [`specs/003-game-api/contracts/api.md`](specs/003-game-api/contracts/api.md)，
卷轴见 [`specs/004-scrolling/contracts/api.md`](specs/004-scrolling/contracts/api.md)，
音乐与音效见 [`specs/005-music/contracts/api.md`](specs/005-music/contracts/api.md)，
改写地图与状态栏见 [`specs/006-map-edit-status/contracts/api.md`](specs/006-map-edit-status/contracts/api.md)。
内置 8×8 ASCII 字体，`print` 无需任何素材即可显示文字。

> 硬件提示：`print`、`set_tile`、`load_palette` 可在任何时刻调用；只有底层的 `vram_addr` / `vram_put` / `vram_fill` 仍需在 `ppu_off()` 期间使用。

## 项目结构

```text
crates/nes-sim   6502 CPU（通过 nestest 全部 5003 行官方指令校验）+ 简化 NES 测试机
crates/nesl      编译器库：lexer → parser → codegen → assembler → CHR/iNES
crates/nesc      CLI 与浏览器编辑器（std-only HTTP 服务 + web/index.html）
examples/        hello / sprite / game（接星星）/ shooter（struct + i8 + match）/ maze（地图 + 元精灵 + 碰撞 + 音效）/ runner（4 屏横版跳跃 + 三声部音乐 + 状态栏）/ quest（完整游戏：标题、3 个世界、敌人、生命与计分）
specs/00x-…      Spec Kit 文档：spec.md（需求）plan.md（计划）tasks.md（实现任务）等
```

## 开发方式

- **Spec Kit**：`.specify/` + `specs/001-nesl-compiler/`，可继续使用 `/speckit-*` 命令迭代新特性。
- **TDD**：git 历史中每个模块都是先提交失败的测试（`test:`），再提交实现（`feat:`）。
  编译器测试会把源码编译成 ROM，在 `nes-sim` 中真正执行，再断言 RAM / VRAM / OAM / APU 状态。
- `cargo test --workspace` 运行全部测试；`cargo clippy --workspace --all-targets` 保持零警告。
- **交叉验证**：`cd tools/xcheck && npm ci` 后，`cargo test` 会把示例 ROM 同时放进测试机和 jsnes（另一个独立模拟器）运行，
  逐像素比较最后一帧，截图写到 `target/xcheck/`。没有 Node 或没装依赖时这些测试自动跳过。

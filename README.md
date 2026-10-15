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

## 语言速览

| 能力 | 写法 |
|---|---|
| 类型 | `u8`、`u16`，`as` 转换 |
| 全局 / 局部 | `var score: u16;`、`let x = 3;` |
| 常量 / ROM 数据 | `const SPEED = 2;`、`const PAL: [u8] = [...];`、`const MSG: [u8] = "HI";` |
| RAM 数组 | `var buf: [u8; 16];` → `buf[i] += 1;` |
| 控制流 | `if / else if / else`、`while`、`loop`、`for i in 0..8`、`break`、`continue` |
| 函数 | `fn add(a: u8, b: u8) -> u8 { return a + b; }`（不支持递归） |
| 图块 | `tile N = [8 行 × 8 像素，字符 . 1 2 3]` |
| NES API | `wait_vblank ppu_on ppu_off load_palette vram_addr vram_put vram_fill print print_u8 sprite hide_sprite joypad scroll rand frame_count peek poke play_tone stop_sound`，按键常量 `BTN_A…BTN_RIGHT` |

完整参考：[`specs/001-nesl-compiler/contracts/language.md`](specs/001-nesl-compiler/contracts/language.md)。
内置 8×8 ASCII 字体，`print` 无需任何素材即可显示文字。

> 硬件提示：`print` / `vram_*` 会写显存，请在 `ppu_on()` 之前，或主循环里紧跟 `wait_vblank()` 之后少量调用。

## 项目结构

```text
crates/nes-sim   6502 CPU（通过 nestest 全部 5003 行官方指令校验）+ 简化 NES 测试机
crates/nesl      编译器库：lexer → parser → codegen → assembler → CHR/iNES
crates/nesc      CLI 与浏览器编辑器（std-only HTTP 服务 + web/index.html）
examples/        hello / sprite / game（接星星小游戏）
specs/001-…      Spec Kit 文档：spec.md（需求）plan.md（计划）tasks.md（实现任务）等
```

## 开发方式

- **Spec Kit**：`.specify/` + `specs/001-nesl-compiler/`，可继续使用 `/speckit-*` 命令迭代新特性。
- **TDD**：git 历史中每个模块都是先提交失败的测试（`test:`），再提交实现（`feat:`）。
  编译器测试会把源码编译成 ROM，在 `nes-sim` 中真正执行，再断言 RAM / VRAM / OAM / APU 状态。
- `cargo test --workspace` 运行全部测试；`cargo clippy --workspace --all-targets` 保持零警告。

# 安装与快速开始

## 构建

需要 Rust 工具链（edition 2021，无第三方运行时依赖）。

```bash
git clone https://github.com/build-your-own-x-with-ai/RustNES_NESL_Studio.git
cd RustNES_NESL_Studio
cargo build --release
```

产物是 `target/release/nesc`。

## CLI

```
nesc — compile NESL programs into NES ROMs

USAGE:
    nesc build <file.nesl> [-o <out.nes>] [--asm]
    nesc check <file.nesl>
    nesc serve [--port <N>]
    nesc help
```

| 命令 | 作用 |
|---|---|
| `nesc build hello.nesl` | 编译成 `hello.nes` |
| `nesc build hello.nesl -o out.nes` | 指定输出路径 |
| `nesc build hello.nesl --asm` | 额外输出汇编清单 `hello.s` |
| `nesc check hello.nesl` | 只做检查，不产出 ROM；打印 `PRG 已用 N 字节` |
| `nesc serve` | 启动本地编辑器，默认 <http://127.0.0.1:8080> |
| `nesc serve --port 3000` | 换端口 |

编译错误会带位置，格式与编译器诊断一致：

```
examples/x.nesl:2:3: error: unknown variable `x`
```

## 第一个程序

新建 `hello.nesl`：

```nesl
// 背景调色板 0：黑底白字
const PAL: [u8] = [0x0F, 0x30, 0x30, 0x30];

fn main() {
    load_palette(PAL);
    print(9, 3, "HELLO NES!");
    ppu_on();
    loop { wait_vblank(); }   // 主循环：每帧等一次，否则 CPU 会跑飞
}
```

```bash
nesc build hello.nesl
# wrote hello.nes (PRG 已用 … 字节)
```

用任意 NES 模拟器打开 `hello.nes` 即可看到文字。

## 程序骨架

每个 NESL 程序**必须有 `fn main()`**，它是入口；`main` 返回后 CPU 停在空循环。
典型结构是：

```nesl
fn main() {
    load_palette(PAL);   // 1. 初始化
    ppu_on();            // 2. 开画面
    loop {               // 3. 主循环
        wait_vblank();   //    每帧同步一次
        // 读输入 → 更新状态 → 画精灵
    }
}
```

## 直接用浏览器编辑器

```bash
./target/release/nesc serve
```

打开 <http://127.0.0.1:8080>，左侧写代码、右侧就是模拟器，`Ctrl/⌘+Enter` 编译并运行。
不想装工具链的话，线上版在 <https://build-your-own-x-with-ai.github.io/RustNES_NESL_Studio/>，
编译器以 WebAssembly 形式在浏览器里跑。详见 [NESL Studio 编辑器](Studio)。

## 常见报错

| 报错 | 原因 |
|---|---|
| `missing input file` | 没给源文件路径 |
| `unknown command` / `unknown option` | 命令或参数写错，运行 `nesc help` |
| `u16 → u8 需要 as` 之类 | 窄化转换必须显式：见 [语言参考](Language-Reference#类型) |
| `PRG 已用 … 超过 32762` | 程序太大，需要拆分或开启 banking |

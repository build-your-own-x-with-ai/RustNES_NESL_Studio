# Implementation Plan: 游戏开发 API

**Branch**: `003-game-api` | **Date**: 2026-10-17 | **Spec**: [spec.md](./spec.md)

## Summary

运行时新增 VRAM 更新队列（主程序入队、NMI 出队），`print` 等内建函数按渲染状态选择「入队」或「直写」；
新增 `map` / `metasprite` 两种 ROM 数据声明及对应内建函数；`collide` 在编译期降级为
「参数存入隐藏槽 + u16 比较表达式」，复用现有代码生成；`sfx` 是 NMI 驱动的小型音效引擎。
测试机增加「渲染期 VRAM 写入」计数，作为 US1 的客观验收标准。

## Technical Context

与 001/002 相同（Rust 1.99、零依赖、cargo test + nes-sim 端到端）。

## Constitution Check

| 原则 | 结论 |
|---|---|
| I 高级语言优先 | ✅ 消除「vblank 内少量写显存」这一必须懂硬件才能避开的陷阱 |
| II 测试先行 | ✅ 先给测试机加违规计数器，所有故事先写失败测试 |
| III 库优先 | ✅ 逻辑在 `nesl`；`nes-sim` 只加观测能力 |
| V 简单 | ✅ collide 复用表达式生成；map/metasprite 是普通 ROM 表 |

## Design

### 测试机（nes-sim）
- `NesBus.now`：当前指令开始时的 CPU 周期，由 `TestNes::step` 设置。
- vblank 窗口 = 帧边界后 `VBLANK_CYCLES = 2273` 个周期（20 条扫描线）。NMI 在帧边界触发，因此 NMI 处理程序运行在窗口内。
- `pub render_conflicts: u64`：`ppumask & 0x18 != 0` 且不在窗口内时对 `$2006/$2007` 的写入计数。

### 内存
| 地址 | 用途 |
|---|---|
| `$15` | `VQ_LEN` 已提交队列长度 |
| `$0300-$035F` | VRAM 队列（96 字节） |
| `$0360-$0362` | `print_u8` 缓冲（原 `$0300`） |
| `$0363` | `VQ_LOCK`：主程序入队期间置位，NMI 跳过本帧刷新 |
| `$0364-$036F` | 音效状态与 `$4015` 影子 |
| `$0370-$07FF` | 用户数组与溢出变量（原 `$0310`） |

### 队列
- `__vq_push`：输入 `TMP1:TMP0` = VRAM 地址、`PTR` = 源、`TMP2` = 长度（≤32）。空间不足时 `JSR __wait_vblank` 后重试。
  先写条目字节，最后 `STX VQ_LEN` 提交 —— NMI 在入队中途触发也只会看到完整条目；
  入队期间 `VQ_LOCK` 置位，NMI 跳过刷新（否则刷新后清零 `VQ_LEN` 会与入队中的写偏移冲突）。
- `__vq_flush`：NMI（渲染开启时）与 `__ppu_off` 共用。在 NMI 中先于 OAM DMA 执行（两者都必须在 vblank 内）。
- `__print` 计算地址后：渲染关闭 → 直写；开启 → 按 32 字节分块调用 `__vq_push`。
- `set_tile` = 1 字节的 `print`；`load_palette` 开启渲染时入队到 `$3F00`。

### map
- 解析：顶层 `Ident("map") Ident "="`（上下文关键字，不保留 `map`）。
- ROM：960 字节；`.`→`0x20`，缺失补 `0x20`。
- `draw_map`：渲染开启时先 `__ppu_off`（会刷新队列），写 960 字节 + 64 字节属性表 0，再 `__ppu_on`。
- `map_tile`：`tx ≥ 32 || ty ≥ 30` 返回 0；否则 `PTR = base + ty*32`，`LDY tx; LDA (PTR),Y`。

### metasprite
- 解析：顶层 `Ident("metasprite") Ident "="`；元组 `(dx, dy, tile[, attr])`。
- ROM：`[count, dx, dy, tile, attr, ...]`。
- `__metasprite`：每块 `x+dx`、`y+dy` 用 ADC 计算，`越界 = C xor (d < 0)`，越界则 Y=$FF 隐藏。返回 `slot + count`。

### collide
- 用 `slots()`（已处理参数含调用的情况）把 8 个参数求值到固定 RAM 槽，在临时作用域中绑定为隐藏变量，
  生成 `(ax as u16) < (bx as u16) + bw && (bx as u16) < (ax as u16) + aw && ...y...` 并调用 `expr()`。

### sfx
- 表项 `[ch, lo, hi, delta, frames, maxvol, duty]`；状态 `SFX_TIMER/CH/PERIOD/DELTA/VOL/DUTY`；`APU_EN` 影子。
- 开始：载入状态，写高位/长度寄存器一次，`APU_EN |= 通道位` 写 `$4015`。
- 每帧（NMI 末尾，无论渲染与否）：`vol = min(timer, maxvol)` 写控制寄存器；`period_lo += delta` 写低位寄存器；
  计时归零时 `APU_EN &= !bit` 写 `$4015`。
- `play_tone` 改为 `APU_EN |= 1`；`stop_sound` 清零 `APU_EN` 与计时器。

## Project Structure

```text
crates/nes-sim/src/nes.rs        now / render_conflicts
crates/nesl/src/ast.rs           Decl::Map, Decl::Metasprite
crates/nesl/src/parser.rs        上下文声明解析、元组
crates/nesl/src/codegen.rs       map/metasprite 数据与符号、collide 降级
crates/nesl/src/builtins.rs      新内建、队列/地图/元精灵/音效运行时
crates/nesl/src/runtime.rs       内存布局、NMI 刷新队列与音效 tick
crates/nesl/tests/game_api.rs    003 端到端测试
examples/maze.nesl               地图 + 元精灵 + 碰撞 + 音效 + 随处 print
```

## Complexity Tracking

无。

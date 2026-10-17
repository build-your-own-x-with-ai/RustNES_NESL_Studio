# Implementation Plan: 横向卷轴与多屏关卡

**Branch**: `004-scrolling` | **Date**: 2026-10-17 | **Spec**: [spec.md](./spec.md)

## Summary

地图改为**按列存储**（每列 30 字节连续），使「补画一列」成为一次 PPU +32 递增的连续写入；
VRAM 队列条目的长度字节最高位表示纵向写入。两个并排命名表（ROM 已是垂直镜像）组成 64 列环形缓冲：
世界第 c 列总是画在环形槽 `c % 64`。镜头每帧最多移动 8 像素，列号每帧最多变化 1，
所以只需补画一列：向右为 `新镜头列 + 32`，向左为 `新镜头列`。属性表在每 4 列一组的边界整组写入。

## Constitution Check

| 原则 | 结论 |
|---|---|
| I 高级语言优先 | ✅ 环形命名表、属性位打包、滚动寄存器全部隐藏在 `camera_follow` 后 |
| II 测试先行 | ✅ 「可见列全部正确 + 渲染期写入为 0」是客观的端到端断言 |
| V 简单 | ✅ 列存储 + 每帧至多一列，避免复杂的脏区追踪 |

## Design

### 数据（编译期）
- `m_NAME`：W×30 字节，第 c 列第 r 行在 `c*30 + r`；`.` → 空格；W = max(32, 最长行)。
- `ma_NAME`：⌈W/4⌉×8 字节，第 g 组第 r 行块的属性字节在 `g*8 + r`；
  字节 = `TL | TR<<2 | BL<<4 | BR<<6`（各为 16×16 区域的调色板号）。无 `palette_zone` 时全 0。
- 宽度 W、最大镜头 `W*8−256` 是编译期常量，直接作为立即数传给运行时。

### 内存（RAM）
| 地址 | 用途 |
|---|---|
| `$036B-$036C` | `CAM_X` |
| `$036D` | `CAM_COL`（镜头左边缘列号） |
| `$036E-$0371` | `MAP_T`、`MAP_A`（当前地图的图块/属性表指针） |
| `$0372` | `MAP_W` |
| `$0373-$0375` | 补画属性的循环变量 |
| `$0380-$07FF` | 用户数组与溢出变量（原 `$0370`） |

### 运行时
- `__col_addr`：`PTR += A*30`（`A*32 − A*2`）。
- `__vram_write_f`：003 的 `__vram_write` 增加 `TMP3` 标志（$80 = 纵向）：渲染关闭时直写（PPUCTRL 临时置 +32），
  开启时入队，标志写入长度字节最高位；`__vq_flush` 按条目切换 PPUCTRL 递增模式，结束时恢复。
- `__draw_map`：关闭渲染 → 按列写前 min(W,64) 列 → 写前 min(⌈W/4⌉,16) 组属性 → 镜头归零 → 恢复渲染。
- `__camera_follow`：16 位钳制与步长限制 → 列号比较 → `__stream_col` / `__stream_attr` → 更新 `SCROLL_X` 与 `CTRL_SHADOW` 的命名表位。
  滚动影子最后更新：NMI 若在补画中途到来，只会刷新已完整入队的条目，新列不会先于数据露出。
- `__map_tile`：`tx` 高字节非 0 或 `tx ≥ W` 或 `ty ≥ 30` → 0；否则 `__col_addr` + `LDY ty`。

### 解析
- `palette_zone NAME = [...]`：与 `map` 相同的上下文声明；codegen 先收集分区，再在声明 `map` 时一并生成属性表。

## Project Structure

```text
crates/nesl/src/{ast,parser,codegen,builtins,runtime}.rs
crates/nesl/tests/scrolling.rs      004 端到端测试
examples/runner.nesl                4 屏横版跳跃示例
```

## Complexity Tracking

无。

# Implementation Plan: 可修改地图与固定状态栏

**Branch**: `006-map-edit-status` | **Date**: 2026-10-17 | **Spec**: [spec.md](./spec.md)

## Summary

地图仍在 ROM；修改记录在 RAM 的「覆盖表」（最多 64 项，按 SoA 存储 地图号/列/行/图块）。
所有读地图的路径（`map_tile`、列补画）先查覆盖表。列补画先把 ROM 列复制到 RAM 缓冲区再套用覆盖项。
状态栏用 sprite-0 分屏：NMI 把滚动设为 (0, 命名表 0)，等待 sprite-0 碰撞后写入镜头滚动；补画跳过状态栏的行。

## Constitution Check

| 原则 | 结论 |
|---|---|
| I 高级语言优先 | ✅ 覆盖表、环形命名表定位、sprite-0 时序全部隐藏 |
| II 测试先行 | ✅ 先给测试机加 sprite-0 碰撞与寄存器写入日志 |
| V 简单 | ✅ 覆盖表线性查找（≤64 项）；不调用 `set_map_tile` 的程序不占 RAM |

## Design

### 测试机
- `NesBus::sprite0_hit(&self) -> Option<(u8, u8)>`：按当前 OAM[0]、PPUCTRL、滚动、VRAM 计算第一个「不透明精灵像素 × 不透明背景像素」的 (x, y)；x=255 不算；渲染关闭时 None。
- 每帧在 `VISIBLE_START = 2273 + 341/3`（预渲染线之后）锁存一次，得到碰撞周期；`$2002` 读在 [碰撞周期, 下一帧 vblank 结束) 内返回第 6 位。
- `pub reg_log: Vec<(u64, u8, u8)>`：`(周期, 种类, 值)`，种类 0 = `$2000`、1 = `$2005` x、2 = `$2005` y。

### 内存（固定地址，运行时状态）
| 地址 | 用途 |
|---|---|
| `$039B` | `OV_COUNT` |
| `$039C` | `STATUS_ROWS` |
| `$039D` | `MAP_ID`（当前绘制/跟随的地图，1 起；0 = 无） |
| `$039E-$039F` | 临时 |
| `$03A0-$03BD` | `COL_BUF`（30 字节） |
| `$03BE-$04BD` | `OV_ID / OV_COL / OV_ROW / OV_TILE`（各 64 字节） |

用户 RAM 起点仍是 `$03A0`；程序调用了 `set_map_tile` 时，codegen 从 `$04BE` 开始分配（预扫描 AST）。

### 运行时
- 地图编号 = 声明顺序 + 1，作为立即数传入 `ARG4`。
- `__ov_find`：在覆盖表里找 (ARG4, 列, 行)，找到返回下标且 C=0，否则 C=1。
- `__map_tile`：越界 → 0；`OV_COUNT > 0` 时先查覆盖表；否则读 ROM。
- `__stream_col`：`OV_COUNT == 0` 时直接从 ROM 写；否则复制到 `COL_BUF`、套用该地图该列的覆盖项再写。
  两种情况都从第 `STATUS_ROWS` 行开始写（VRAM 地址 +n×32，源 +n，长度 30−n）。
- `__attr_group`：从属性行 ⌈n/4⌉ 开始。
- `__set_map_tile`：更新或追加覆盖项；若 `MAP_ID` 相同、行 ≥ n、列在 [`CAM_COL`, `CAM_COL`+32] 内，按环形槽位写一格。
- `__reset_map`：压缩删除该地图的项。
- `__status_bar`：设置 `STATUS_ROWS`，经队列清空 `$2000`/`$2400` 的前 n 行与对应属性行。
- NMI：状态栏开启时，DMA 前写 OAM[0] = (y = (n−1)×8+5, tile $7E, 属性 $20, x = 248)，刷新队列后写标记格 `$7F` 到 (31, n−1)；
  滚动写 (0, 命名表 0)；末尾若渲染开启，限时等待第 6 位清零、再置位，写 `LAST_CTRL` 与 `LAST_SX`。

### CHR
- `$7E`（精灵）与 `$7F`（背景）各在第 6 行最左一个像素，颜色 1；用户 `tile` 可覆盖（但会使状态栏失效，文档注明）。

## Project Structure

```text
crates/nes-sim/src/nes.rs                 sprite-0 碰撞、寄存器日志
crates/nesl/src/{chr,codegen,builtins,runtime}.rs
crates/nesl/tests/common/mod.rs           共享宽地图辅助函数
crates/nesl/tests/map_edit.rs             US1
crates/nesl/tests/status_bar.rs           US2
examples/runner.nesl                      US3
```

## Complexity Tracking

无。

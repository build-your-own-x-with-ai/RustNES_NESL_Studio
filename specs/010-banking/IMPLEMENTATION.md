# Implementation Notes

## Two-Pass Compilation

The compiler tries NROM first (mapper 0). If the assembled PRG code exceeds 32762 bytes (leaving 6 bytes for vectors), it retries with UNROM (mapper 2):

1. **NROM path** (code + data under 32KB): single 32KB bank, 8KB CHR-ROM, vectors at $FFFA
2. **UNROM path** (code + data over 32KB):
   - Maps split across switchable 16KB banks (first-fit into banks 0-15)
   - Code and runtime in the fixed bank ($C000-$FFFF)
   - CHR data (8KB) at the start of bank 0
   - __chr_copy runs at reset to load CHR-RAM from bank 0

## Bank Assignment

`assign_banks()` uses first-fit: each map (tiles + attributes) is placed in the first bank with enough free space, or a new bank is created. Bank 0 reserves 8KB for CHR data. If more than 16 banks are needed, compilation fails (UNROM's limit).

## Map Access

Map builtins (`draw_map`, `camera_follow`, `map_tile`) call `map_bank(&m)` before accessing map data:

- **Static maps**: `LDA #bank; JSR __bank`
- **Dynamic maps** (from a list): `LDA SEL_BANK; JSR __bank` (SEL_BANK set by __map_sel)

`__bank` writes to a bus-conflict-safe table where ROM[addr]=value, preventing CPU/ROM conflicts during the write to $8000-$BFFF.

## CHR Copy

At reset, `__chr_copy`:
1. Switches to bank 0
2. Copies 8KB from $8000 (after CHR data in bank 0) to PPU $0000-$1FFF
3. Returns with PPUCTRL=0

Games using CHR-RAM can update tiles at runtime via PPUADDR/PPUDATA when rendering is off.

## ROM Layout (UNROM)

```
Header:    PRG banks (power-of-2), CHR banks=0, mapper 2
Bank 0:    [8KB CHR data][map data, up to 8KB]
Bank 1-N:  [map data, each bank 16KB]
Fixed:     [code + runtime, 16KB] with vectors at $FFFA
Padding:   empty banks to reach power-of-2 count
```

The fixed bank is always last. Padding banks are inserted before it, not after, so the reset vector stays at the correct offset.

## Testing

- `banking.rs`: UNROM triggers correctly, CHR copy works, bank padding preserves vectors
- `quest.rs`: 6-world COIN QUEST (34KB) compiles to UNROM, all maps accessible, zero bus conflicts
- `layout_tests`: runtime variables don't overlap (catches SEL_BANK/VQ_LOCK collision)
- `banking.rs::scrolling_an_unrom_level_keeps_the_screen_in_sync_with_the_map`: after scrolling, every visible nametable column equals the map
- `xcheck.rs::scrolled_unrom_level_matches_jsnes`: a scrolled UNROM level matches jsnes pixel for pixel

## Jump Physics Fix

COIN QUEST's platforms were unreachable with JUMP=-7 (21px rise). Changed to JUMP=-9 (36px rise), allowing tier A platforms at row 17 (32px above ground) and tier B at row 13 (32px above tier A). Pits capped at 4 tiles wide (28px) to stay within ~40px horizontal jump clearance.

## FR-1001: NROM Compatibility

All existing example ROMs (hello, sprite, game, shooter, maze, runner, quest-3world) remain byte-identical to the pre-banking builds. Only programs exceeding 32KB trigger UNROM.

## 镜像位 Bug（FCEUX 中地面/平台与碰撞不一致）

现象：FCEUX 中 COIN QUEST 平地上掉下去、吃掉的金币不消失、踩不到空中平台；runner.nes（NROM）正常。

原因：`build_unrom` 的 iNES flags6 写成 `0x20`（bit0=0，水平镜像）。横向卷轴需要垂直镜像；水平镜像下 $2400 是 $2000 的镜像，
镜头流式写入的新列覆盖了正在显示的屏幕，画面与地图（碰撞）不再一致。NROM 一直写 `0x01`，所以只有 UNROM 出错。

为什么测试没发现：nes-sim 过去总是按垂直镜像处理，忽略头部的镜像位；jsnes 交叉验证只比较了静态标题画面。

修复：
- flags6 改为 `0x21`
- nes-sim 按 iNES 镜像位折叠名称表（`nt_index` / `nt_base`），渲染器同样处理
- 新增回归测试：UNROM 头部镜像位、卷轴后屏幕与地图一致、卷轴 UNROM 场景与 jsnes 对比

jsnes 已知缺陷：第 0 行来自预渲染的虚拟扫描线，合成时会从未填充的背景缓冲区复制，出现 undefined 像素（精细水平卷轴时可见）。
run.js 将其输出为 `UNDRAWN`，xcheck 排除这些像素，并断言它们只出现在第 0 行。

调试日志说明：UNROM 卡带在 $6000-$7FFF 没有 PRG-RAM，写入会被丢弃，所以那里的调试日志在 FCEUX 中读到的全是开路值。

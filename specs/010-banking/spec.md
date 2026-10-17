# Feature Specification: 地图换 bank（UNROM，最大 256KB）

**Feature Branch**: `010-banking`

**Created**: 2026-10-18

**Status**: Implemented

**Input**: User description: "更大的游戏 & 优化"；实测后确认范围为「只做地图换 bank」。

## 实测（决定范围）

| 项目 | COIN QUEST（3 个世界）节省 |
|---|---|
| 窥孔优化（`STA x / LDA x` 等） | 约 50 字节 |
| 局部变量槽位复用 | 0（零页只用到 `$24`） |
| 删除没用到的运行时 | 约 100 字节 |
| 音乐放进换 bank | 282 字节（总量太小，没意义） |
| **地图与画面数据** | **15.8KB / 21.7KB（73%）** |

结论：只有把地图放进可切换的 bank 才能让游戏变大；其余几项不做。

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 超过 32KB 自动换 bank (Priority: P1) 🎯 MVP

程序装得下 32KB 时照旧输出 NROM，不变；装不下时编译器自动输出 UNROM（mapper 2）：代码、常量、运行时放在固定的
最后一个 16KB bank（`$C000-$FFFF`），地图放进可切换 bank（`$8000-$BFFF`），图块图形开机时复制到 CHR-RAM。
用户代码一个字都不用改，`draw_map` / `map_tile` / `camera_follow` / 地图列表照常工作。

**Independent Test**: 编译 9 张 128 列地图（约 36KB）的程序，断言 iNES 头为 mapper 2、CHR 0 块；在测试机中
`draw_map` 第 9 张图、`map_tile` 读第 1 张和第 9 张、`camera_follow` 卷过一张换 bank 地图、`L[i]` 动态选图都正确；
字体已复制进 CHR-RAM；没有总线冲突写入，渲染期显存写入为 0。

**Acceptance Scenarios**:

1. **Given** 现有的所有示例（都小于 32KB），**When** 编译，**Then** 仍是 NROM，行为不变。
2. **Given** 9 张宽地图，**When** 编译运行，**Then** 每张地图的读取、绘制、卷轴结果与它单独存在时相同。
3. **Given** 代码与小数据超过 16KB 固定 bank，或地图需要超过 15 个可切换 bank，**When** 编译，**Then** 报错说明原因。

### User Story 2 - COIN QUEST 扩到 6 个世界 (Priority: P2)

6 个世界（超过 32KB）自动以 UNROM 输出；端到端测试从第 1 世界一路过关到 YOU WIN!；标题画面与 jsnes 一致。

## Requirements *(mandatory)*

- **FR-1001**: 总大小 ≤ 32762 字节时 MUST 输出 NROM（与之前逐字节相同）；否则 MUST 输出 UNROM。
- **FR-1002**: 读地图 ROM 的内建函数 MUST 在调用前切换到该地图所在的 bank；NMI 不读可切换 bank。
- **FR-1003**: 切换 bank MUST 避免总线冲突（写入的值等于 ROM 中该地址的值）。
- **FR-1004**: 测试机 MUST 支持 mapper 2（可切换 + 固定 bank、CHR-RAM），并统计总线冲突写入。

## Success Criteria *(mandatory)*

- **SC-1001**: 6 世界 COIN QUEST 通关测试与 jsnes 交叉验证通过。
- **SC-1002**: `cargo test --workspace` 全绿，已有 ROM 不变。

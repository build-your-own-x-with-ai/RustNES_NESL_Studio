# Feature Specification: 完整游戏 COIN QUEST（顺带修掉写游戏时遇到的问题）

**Feature Branch**: `009-quest`

**Created**: 2026-10-18

**Status**: Implemented

**Input**: User description: "做一个完整游戏：标题画面、3 个关卡、敌人、生命与计分、音乐，顺便暗审语言哪里用起来别扭、把发现的问题修掉"

## 写游戏时遇到的问题（先用现有功能写出草稿，再记录）

| # | 问题 | 草稿里的绕法 | 处理 |
|---|---|---|---|
| F1 | 地图不是值：不能放进数组、不能按关卡号选 | `tile_at` / `take_coin` / `follow` / 画图 4 个函数各写一遍 `match world { 0 => …W1, 1 => …W2, _ => …W3 }`，`reset_map` 写 3 遍 | **修**：地图列表 `const WORLDS: [map] = [W1, W2, W3];`，凡是要地图的地方都能写 `WORLDS[i]` |
| F2 | 没法显示 u16 数字：分数一超过 255 就只能放弃 | 草稿里删掉了分数 | **修**：`print_u16(x, y, v)` |
| F3 | `map_tile` 的列是 u16，`set_map_tile` 的列却是 u8 | `set_map_tile(W1, tx as u8, ty, ' ')` | **修**：`set_map_tile` 的列也接受 u16 |
| F4 | 调色板也不能按关卡选 | `match` 选 `load_palette(PAL1/2/3)` | 保留：3 个分支可以接受，记录为限制 |
| — | （游戏 bug）被打中后重生没有无敌时间 | — | 游戏代码修正 |

检查过但**不是问题**的：`match` 覆盖所有情况后不需要多余的 `return`；主循环每帧都跑得完（600 帧 596 次）。

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 地图列表 (Priority: P1)

`const WORLDS: [map] = [W1, W2, W3];` 之后，`draw_map`、`map_tile`、`set_map_tile`、`reset_map`、`camera_follow`
都能接受 `WORLDS[i]`（i 可以是变量或表达式，只求值一次）；列表可以写在地图声明之前；下标越界时使用列表第一张地图。

**Acceptance Scenarios**:

1. **Given** 两张宽度不同的地图组成列表，**When** `draw_map(L[i])` / `map_tile(L[i], …)` / `camera_follow(x, L[i])`，**Then** 行为与直接写那张地图完全相同（包括宽度与镜头上限）。
2. **Given** `set_map_tile(L[1], …)`，**Then** `map_tile(V, …)` 看到修改。
3. **Given** 列表里有不存在的名字或不是地图的名字、把列表本身当地图用、常量下标越界，**Then** 编译报错。

### User Story 2 - `print_u16` 与 `set_map_tile` 的 u16 列 (Priority: P1)

`print_u16(x, y, v)` 以 5 位十进制显示（00000-65535）；`set_map_tile(M, tx, …)` 的 tx 可为 u16，超过地图宽度返回 0。

### User Story 3 - COIN QUEST (Priority: P1)

标题画面 → 3 个 128 列的横向卷轴世界 → 通关画面；史莱姆会巡逻、在墙和悬崖边掉头；踩它得 50 分，碰到失去一条命；
金币 10 分（死了也不复活）；掉进坑失去一条命；3 条命用完进入 GAME OVER；状态栏显示分数、金币、生命、世界；有背景音乐与音效。

**Independent Test**: 在测试机里用脚本按键/改坐标验证：开始游戏、踩死史莱姆、被史莱姆打中、掉坑、GAME OVER、过关进入下一世界、通关；全程渲染期显存写入为 0；标题与第一关画面与 jsnes 逐像素一致。

## Requirements *(mandatory)*

- **FR-901**: 语言 MUST 支持 `const NAME: [map] = [M, …];`，地图参数 MUST 接受 `NAME[i]`。
- **FR-902**: 内建 `print_u16(x: u8, y: u8, v: u16)`。
- **FR-903**: `set_map_tile` 的列参数为 u16。
- **FR-904**: `examples/quest.nesl` 与端到端测试；编辑器示例列表加入 quest。

## Success Criteria *(mandatory)*

- **SC-901**: quest 用地图列表后，草稿里 4 个按关卡分派的函数与 3 次 `reset_map` 全部消失。
- **SC-902**: `cargo test --workspace` 全绿。

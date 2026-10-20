# Feature Specification: 更多敌人行为与音效

**Feature Branch**: `011-foes`

**Created**: 2026-10-20

**Status**: In Progress

**Input**: User choice: "更多敌人与音效 (Recommended)"

## 动机

COIN QUEST 现在只有一种敌人（史莱姆：左右巡逻），游戏单调。增加几种行为（跳跃、飞行）和对应音效（踩死、跳跃），
用很少代码就能让游戏更有趣，同时验证现有语言（struct、i8、sound、match）写游戏对象足够方便。

## 写草稿后发现的问题

| # | 问题 | 草稿里的做法 | 处理 |
|---|---|---|---|
| F1 | `sfx(SFX_HIT)` 是内置音效（id 3），`sfx(STOMP)` 是 `sound` 声明，写法一样但类型不同 | 草稿直接通过编译 | **保留**：这是预期行为，`sound` 名字就是给 `sfx` 用的常量 |
| F2 | `const WAVE: [i8] = […]` 用 i8 数组描述蝙蝠飞行的正弦波 | 编译成功 | **确认**：i8 数组已支持（002 语言增强） |
| F3 | `for i in 0..nfoes { match foes[i].kind { 0 => walk(i), 1 => hop(i), _ => fly(i) } }` | 草稿通过 | **确认**：match、struct 数组全部工作 |
| — | （非问题）三种敌人是否需要不同的元精灵 | 史莱姆 1 tile、跳跳怪 1 tile、蝙蝠 2-frame 动画 | 草稿里已实现 |

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 跳跃敌人（跳跳怪）(Priority: P1) 🎯 MVP

在地面时走动，每 64 帧跳一次（vy = -6，类似主角但更低），掉进坑会消失。

**Acceptance Scenarios**:

1. **Given** 跳跳怪站在平地，**When** 运行 64 帧，**Then** y 坐标先减后增，最后回到地面。
2. **Given** 跳跳怪掉进坑（y > 216），**Then** `alive = 0`。

### User Story 2 - 飞行敌人（蝙蝠）(Priority: P1)

固定高度（y = 120）来回飞行，y 坐标按 16 项正弦表上下波动（`WAVE: [i8]`），无视墙壁和坑。

**Acceptance Scenarios**:

1. **Given** 蝙蝠飞行 128 帧，**Then** y 坐标完成 8 个周期，落在 `BAT_Y + WAVE[…]` 上。
2. **Given** 蝙蝠到达地图最右边，**Then** 掉头继续飞（dir 反向）。

### User Story 3 - 新音效与音效测试 (Priority: P1)

`sound STOMP = (pulse, 0x1A0, -12, 8, 12, 2);` 踩死敌人时播放；`sound HOP = (pulse, 0x180, -20, 6, 6, 1);` 跳跳怪起跳时播放。

**Acceptance Scenarios**:

1. **Given** 踩死敌人，**When** 检查 APU 写入日志，**Then** `$4006` 从 0xA0 起每帧 -12（共 8 帧），音量 ≤ 12。
2. **Given** quest 运行中每种敌人至少出现一次，**Then** 测试机里 600 帧都能跑完（main loop 不掉帧）。

### User Story 4 - COIN QUEST 用上新敌人 (Priority: P1)

6 个世界的敌人列位置保持不变（`FOE_COL` / `FOE_FROM`），每个位置增加类型标记（`FOE_KIND: [u8]`，0 = 史莱姆，
1 = 跳跳怪，2 = 蝙蝠）；`struct Foe` 增加 `kind: u8, vy: i8, t: u8`（行为计时器）；
`update_foes` 改写为 `match foes[i].kind` 调用 `walk(i)` / `hop(i)` / `fly(i)`。

**Independent Test**: quest 端到端测试（标题 → 6 世界 → 通关）仍然全绿；jsnes 交叉验证第一关与标题画面逐像素一致。

## Requirements *(mandatory)*

- **FR-1101**: quest 增加 3 种敌人行为（走、跳、飞）和对应图块、2 个自定义音效。
- **FR-1102**: 测试覆盖：跳跃/飞行行为，新音效 APU 写入，600 帧不掉帧。

## Success Criteria *(mandatory)*

- **SC-1101**: `cargo test --workspace` 全绿，quest 测试包含 3 种敌人行为验证。
- **SC-1102**: quest 主循环 600 帧全部跑完，跳跳怪会跳、蝙蝠上下波动飞行。
- **SC-1103**: jsnes 交叉验证通过（静态画面无退化）。

## Assumptions

- 现有 `struct` / `i8` 数组 / `sound` / `match` 全部正常工作（002、005 已实现并测试）。
- quest 的 6 世界敌人位置（38 个）手工分配类型，不需要地图编辑器支持。
- 敌人图块直接写在 quest.nesl 里（tile 4/5/6），不需要独立素材文件。

## Out of Scope

- 更多敌人类型（投掷物、追踪玩家）：留给 012 或更晚。
- 敌人 HP / 多段伤害：单次踩死已足够，无需复杂化。
- 编辑器可视化放置敌人类型：手写数组对 38 个位置可以接受。

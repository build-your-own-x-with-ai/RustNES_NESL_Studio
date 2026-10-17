# Feature Specification: 横向卷轴与多屏关卡

**Feature Branch**: `004-scrolling`

**Created**: 2026-10-17

**Status**: Implemented

**Input**: User description: "卷轴与多屏关卡：横向卷轴、比一屏大的地图（自动在滚动时补画新的一列）、属性表/背景调色板分区、跟随主角的镜头。能做出马里奥式的横版关卡"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 比一屏更宽的地图 (Priority: P1) 🎯 MVP

开发者用 `map WORLD = [...]` 写出最多 255 列宽的关卡（仍为 30 行）。`draw_map(WORLD)` 画出最左侧两屏
（两个并排的命名表），`map_tile(WORLD, tx, ty)` 可用 u16 列号查询任意位置。

**Why this priority**: 没有宽地图，卷轴无从谈起；它也独立有用（例如两屏宽的竞技场）。

**Independent Test**: 编译 64/128 列地图，`draw_map` 后断言 `$2000` 与 `$2400` 两个命名表的内容；
`map_tile` 对远处列、越界列（含 u16 大值）返回正确值。

**Acceptance Scenarios**:

1. **Given** 64 列地图，**When** `draw_map`，**Then** 第 0-31 列在 `$2000`，第 32-63 列在 `$2400`。
2. **Given** 128 列地图，**When** `map_tile(W, 100, 5)`，**Then** 返回第 100 列第 5 行的字符；`map_tile(W, 300, 5)` 返回 0。
3. **Given** 256 列的行，**When** 编译，**Then** 报错并指出上限 255。
4. **Given** 003 的 ≤32 列地图，**When** 编译运行，**Then** 行为与 003 完全一致。

---

### User Story 2 - 跟随主角的镜头与自动补画 (Priority: P1)

开发者每帧调用 `camera_follow(hero.x, WORLD)`：镜头把主角保持在屏幕中部附近，在地图两端停止，
每帧最多移动 8 像素；滚动时新露出的一列（及其属性）自动排队绘制。`camera_x()` 返回镜头的世界坐标，
用于把世界坐标换算成屏幕坐标。

**Why this priority**: 这是「横版关卡」的核心体验；与 US1 一起构成 MVP。

**Independent Test**: 主角持续向右移动穿过 4 屏宽的地图再返回；每帧断言卷轴寄存器与 `camera_x()` 一致，
任意时刻屏幕可见的 33 列在环形命名表中的内容都等于地图对应列；全程渲染期显存写入为 0。

**Acceptance Scenarios**:

1. **Given** 主角 x=200，**When** `camera_follow`，**Then** 镜头逐步移到 x=80（主角在屏幕 x=120）。
2. **Given** 主角接近地图右端，**When** 继续向右，**Then** 镜头停在 `宽度×8−256`。
3. **Given** 主角瞬移到远处，**When** 下一帧，**Then** 镜头只移动 8 像素（逐帧追上）。
4. **Given** 向右滚过多屏后再向左回到起点，**When** 检查命名表，**Then** 可见列全部正确。
5. **Given** 渲染关闭时调用，**When** 执行，**Then** 新列直接写入而不是进入不会被刷新的队列。

---

### User Story 3 - 背景调色板分区 (Priority: P2)

开发者用 `palette_zone WORLD = ["0011", "2233", ...];` 为地图每个 16×16 区域指定背景调色板（0-3），
例如天空、草地、砖块用不同颜色。`draw_map` 与卷轴补画都会写入对应的属性表字节。

**Why this priority**: 不同区域不同颜色让关卡好看得多，但不影响能否卷轴。

**Independent Test**: 断言 `draw_map` 后属性表字节按 NES 规则（左上/右上/左下/右下各 2 位）组合；
滚动到第三屏后断言补画的属性字节。

**Acceptance Scenarios**:

1. **Given** 区域 `["01", "23"]`，**When** `draw_map`，**Then** `$23C0` = `0b11_10_01_00`。
2. **Given** 128 列地图带分区，**When** 镜头滚到能看到第 64 列，**Then** 环形中对应属性字节为第 16 组的数据。
3. **Given** 分区引用不存在的地图、含 `0-3` 以外的字符、超过 15 行或宽于地图，**When** 编译，**Then** 报错。

---

### User Story 4 - 横版跳跃示例 (Priority: P2)

一个 128 列（4 屏）的横版关卡：重力与跳跃（i8 速度）、站在地面/平台上、金币、敌人在镜头外隐藏、
用精灵显示的固定分数。

**Independent Test**: 测试机中按住右键 + 跳跃，主角世界坐标超过 256，镜头跟随，渲染期显存写入为 0。

### Edge Cases

- 地图宽度 = max(32, 最长行)，≤ 255 列；高度固定 30 行，不足补空格。
- `draw_map` 重置镜头到 0（滚动寄存器 X 归零）。镜头不能「瞬移」：需要时重新 `draw_map`。
- 背景文字（`print`）会随卷轴滚动并可能被补画覆盖；固定 HUD 请用精灵（示例演示了用字体图块的精灵显示分数）。
- 只支持横向卷轴；`scroll(x, y)` 的 y 不受 `camera_follow` 影响。
- `map_tile` 的列参数改为 u16（u8 自动加宽），超过宽度返回 0。

## Requirements *(mandatory)*

- **FR-401**: `map` MUST 支持最多 255 列；数据按列存储；`draw_map` 绘制前 min(宽度, 64) 列到两个并排命名表并重置镜头。
- **FR-402**: `map_tile(M, tx: u16, ty: u8) -> u8` MUST 支持宽地图，越界返回 0。
- **FR-403**: `camera_follow(x: u16, M)` MUST：目标 = x − 120，限制在 [0, 宽度×8−256]；每帧最多移动 8 像素；
  列号变化时补画新露出的列（向右为 镜头列+32，向左为 镜头列）；在列组边界补画 8 个属性字节；最后更新滚动影子寄存器。
- **FR-404**: `camera_x() -> u16` MUST 返回当前镜头世界坐标。
- **FR-405**: `palette_zone NAME = [行...];` MUST 生成每个地图的属性表数据（≤15 行，每行 ≤ ⌈宽度/2⌉ 列，字符 `0-3`，`.`=0）。
- **FR-406**: VRAM 队列条目 MUST 支持「纵向写入」（PPU +32 递增）。
- **FR-407**: 001-003 的全部测试 MUST 通过（003 中「33 列地图报错」的测试按新上限更新为 256 列）。

## Success Criteria *(mandatory)*

- **SC-401**: 4 屏宽地图往返卷轴全程，测试机渲染期显存写入为 0，可见列 100% 正确。
- **SC-402**: 横版跳跃示例逻辑代码 ≤ 150 行（不含地图行与像素图）。
- **SC-403**: `cargo test --workspace` 全绿。

## Assumptions

- 镜头每帧 ≤ 8 像素，因此每帧最多补画一列（33 字节）+ 8 个属性条目（32 字节），在 vblank 预算内。
- 不实现纵向卷轴、状态栏分屏（sprite-0 hit）与镜头瞬移，留待后续。

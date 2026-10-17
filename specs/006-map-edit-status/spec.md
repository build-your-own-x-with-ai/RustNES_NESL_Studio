# Feature Specification: 可修改地图与固定状态栏

**Feature Branch**: `006-map-edit-status`

**Created**: 2026-10-17

**Status**: Implemented

**Input**: User description: "可修改地图与状态栏：set_map_tile 改写地图（吃掉的金币卷回来不再出现、可破坏的砖块），map_tile 与卷轴补画都看得到修改；再加固定在屏幕顶部的状态栏（sprite-0 分屏），分数不再只能用精灵画"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 改写地图格子 (Priority: P1) 🎯 MVP

开发者用 `set_map_tile(WORLD, tx, ty, ' ')` 改掉地图上的一格（吃掉金币、打碎砖块）。
之后 `map_tile` 读到新值；这一格在屏幕上立即更新；卷轴卷走再卷回来时补画的也是新值。

**Why this priority**: 004 的 runner 里金币卷回来会复活，这是写卷轴游戏最明显的缺陷。

**Independent Test**: 修改一格后断言 `map_tile` 返回新值、命名表对应格更新；在宽地图上卷到远处再卷回来，断言该格仍是新值；
修改屏幕外的格子，卷过去时断言补画为新值。

**Acceptance Scenarios**:

1. **Given** `set_map_tile(W, 5, 10, ' ')`，**When** `map_tile(W, 5, 10)`，**Then** 返回空格；相邻格不受影响。
2. **Given** 该格当前可见，**When** 修改，**Then** 命名表中该格在下一帧更新（渲染中不花屏）。
3. **Given** 宽地图上修改了第 40 列，**When** 镜头卷到 200 列附近再回来，**Then** 第 40 列仍是修改后的值。
4. **Given** 修改了屏幕外的第 90 列，**When** 镜头卷过去，**Then** 补画出的是修改后的值。
5. **Given** 两张地图，**When** 只改其中一张，**Then** 另一张不变。
6. **Given** `reset_map(W)`，**When** 之后读取或重新 `draw_map`，**Then** 恢复原始地图。
7. **Given** 已记录 64 处修改，**When** 再改一个新的格子，**Then** `set_map_tile` 返回 0（未记录，但仍更新可见画面）；改已记录过的格子仍返回 1。

---

### User Story 2 - 固定在顶部的状态栏 (Priority: P2)

`status_bar(4)` 让屏幕顶部 4 行不随卷轴滚动。`print` / `print_u8` 写在这几行里的文字固定显示；
地图从第 4 行以下开始卷动；`status_bar(0)` 关闭。

**Why this priority**: 004 里分数只能用精灵逐个画；有了状态栏就能直接 `print`。

**Independent Test**: 测试机模拟 sprite-0 碰撞；断言每帧画面顶部滚动为 0、碰撞发生后、第 `n*8` 条扫描线开始前写入了镜头滚动值；
卷轴补画不覆盖状态栏的行；状态栏文字在卷轴后仍在。

**Acceptance Scenarios**:

1. **Given** `status_bar(4)` 且镜头在 x=300，**When** 一帧开始渲染，**Then** 顶部使用滚动 (0, 命名表 0)，并在第 32 条扫描线之前切换到 (300 & 255, 命名表 1)。
2. **Given** 状态栏里 `print(1, 1, "SCORE")`，**When** 卷轴卷过几屏，**Then** 命名表 `$2000` 第 1 行仍是 `SCORE`，补画只写第 4-29 行。
3. **Given** 状态栏行数不是 2-8 的常量（0 表示关闭），**When** 编译，**Then** 报错。
4. **Given** 状态栏标记被用户覆盖导致碰撞永不发生，**When** 运行，**Then** 程序不卡死（等待有超时）。

---

### User Story 3 - 示例更新 (Priority: P3)

runner 改用 `set_map_tile` 收金币（卷回来不复活），分数改用状态栏 `print` 显示。

### Edge Cases

- 修改记录最多 64 处，所有地图共用；只有程序里调用了 `set_map_tile` 才会占用这部分 RAM（约 290 字节）。
- 状态栏占用精灵 0 和状态栏最后一行最右边一格（第 31 列）作为碰撞标记；精灵从 1 号开始用。
- 状态栏的调色板固定为背景调色板 0；`palette_zone` 对状态栏所在的属性行不生效。
- 状态栏行数 2-8（硬件限制：精灵无法出现在第 0 条扫描线，所以 1 行做不到）。
- 只支持横向卷轴（同 004）。

## Requirements *(mandatory)*

- **FR-601**: `set_map_tile(M, tx: u8, ty: u8, tile: u8) -> u8` MUST 修改地图格子并返回是否已记录；`map_tile`、`draw_map`、`camera_follow` 补画 MUST 使用修改后的值。
- **FR-602**: 若修改的格子在当前可见区域内，MUST 立即更新画面（经 VRAM 队列）。
- **FR-603**: `reset_map(M)` MUST 删除该地图的全部修改（不重画屏幕）。
- **FR-604**: `status_bar(n)`（n = 0 或 2-8，编译期常量）MUST 让顶部 n 行固定，地图补画与属性补画跳过这些行。
- **FR-605**: 测试机 MUST 模拟 sprite-0 碰撞（`$2002` 第 6 位）并记录 `$2000/$2005` 写入的周期。
- **FR-606**: 001-005 的全部测试 MUST 继续通过。

## Success Criteria *(mandatory)*

- **SC-601**: runner 里收集的金币在卷轴往返后不再出现（端到端断言）。
- **SC-602**: 状态栏开启时，测试机中每帧的顶部/卷动区域滚动值都正确，渲染期显存写入为 0。
- **SC-603**: `cargo test --workspace` 全绿。

## Assumptions

- 测试机中的 sprite-0 碰撞是简化模型（不含精灵溢出、左 8 像素裁剪的细节），足以验证分屏时序。
- 不实现底部状态栏、多段分屏与纵向卷轴。

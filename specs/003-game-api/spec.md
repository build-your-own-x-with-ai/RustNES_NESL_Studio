# Feature Specification: 游戏开发 API（VRAM 队列 / 地图 / 元精灵 / 碰撞 / 音效）

**Feature Branch**: `003-game-api`

**Created**: 2026-10-17

**Status**: Implemented

**Input**: User description: "游戏开发 API：VRAM 更新队列（主循环里随时 print 不花屏）、背景地图 map 声明、16x16 元精灵、碰撞检测、简单音效。解决现在写游戏最别扭的硬件限制"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 随时更新画面不花屏 (Priority: P1) 🎯 MVP

开发者在主循环任何位置调用 `print` / `print_u8` / `set_tile` / `load_palette`，不再需要关心
「只能在 vblank 刚开始时少量写显存」的硬件规则。渲染开启时，这些写入进入 RAM 队列，由下一次 NMI
在 vblank 内统一刷新；渲染关闭时直接写入。

**Why this priority**: 001/002 里所有示例都要小心地把 `print` 放在 `wait_vblank()` 之后，
写多了就花屏 —— 这是写游戏最别扭、最容易出错的地方。

**Independent Test**: 测试机增加「渲染期间写 VRAM」计数器。渲染开启后在主循环的任意位置大量 `print`，
运行若干帧后断言命名表内容正确且计数器为 0。

**Acceptance Scenarios**:

1. **Given** 渲染已开启，**When** 在一帧的中途 `print(2, 5, "SCORE")`，**Then** 下一帧后命名表出现文字，且没有任何 `$2006/$2007` 写入发生在 vblank 之外。
2. **Given** 一帧内排队的数据超过队列容量，**When** 继续 `print`，**Then** 调用自动等待一帧刷新后再入队，内容不丢失、顺序不变。
3. **Given** 渲染关闭，**When** `print`，**Then** 立即写入 VRAM（与 001 行为一致）。
4. **Given** 队列中还有未刷新的数据，**When** `ppu_off()`，**Then** 数据在关闭渲染后立即写入，不丢失。
5. **Given** 渲染开启，**When** `load_palette(PAL)`，**Then** 调色板在下一帧 vblank 内更新（可用于淡入淡出）。

---

### User Story 2 - 用文本画关卡地图 (Priority: P2)

开发者用 `map LEVEL1 = ["####", "#..#", ...];` 以字符画声明一屏背景（最多 30 行 × 32 列），
`draw_map(LEVEL1)` 绘制整屏，`map_tile(LEVEL1, tx, ty)` 查询某格的图块用于墙壁碰撞。

**Why this priority**: 关卡布局是几乎所有游戏都需要的；目前只能逐个 `vram_put`。

**Independent Test**: 编译含 map 的程序，`draw_map` 后断言命名表 960 字节与地图一致、属性表清零；
`map_tile` 对边界内/外坐标返回正确值。

**Acceptance Scenarios**:

1. **Given** 地图字符 `c`，**When** 绘制，**Then** 对应格的图块编号为 `c` 的 ASCII 码（与内置字体及 `tile '#' = [...]` 一致）；`.` 视为空格 `0x20`。
2. **Given** 少于 30 行或某行少于 32 列，**When** 绘制，**Then** 其余格填空格。
3. **Given** 渲染开启时调用 `draw_map`，**When** 执行，**Then** 自动关闭渲染、绘制、在下一帧重新开启，不产生花屏写入。
4. **Given** `map_tile(L, 40, 3)` 越界，**When** 执行，**Then** 返回 0。
5. **Given** 超过 30 行、超过 32 列，**When** 编译，**Then** 报错并指出行号。

---

### User Story 3 - 多块拼成的元精灵 (Priority: P2)

开发者用 `metasprite HERO = [(0, 0, 1), (8, 0, 2), (0, 8, 3), (8, 8, 4, 0x40)];` 声明由多个 8×8 精灵组成的角色，
`metasprite(slot, x, y, HERO)` 一次画出全部，返回下一个可用的精灵槽位。

**Why this priority**: 16×16 角色是 NES 游戏的标配，手写 4 次 `sprite` 既啰嗦又容易算错偏移。

**Independent Test**: 调用后断言 OAM 影子中从 slot 开始的精灵坐标/图块/属性；部分移出屏幕的块被隐藏。

**Acceptance Scenarios**:

1. **Given** HERO 4 块，**When** `metasprite(2, 100, 50, HERO)`，**Then** 精灵 2-5 依次为各块，返回 6。
2. **Given** 偏移为负（`(-8, 0, 1)`），**When** 在 x=100 绘制，**Then** 该块 x=92。
3. **Given** 某块超出屏幕边缘（坐标回绕），**When** 绘制，**Then** 该块被隐藏而不是出现在对侧。
4. **Given** 第四个元素省略，**When** 编译，**Then** 属性默认为 0。

---

### User Story 4 - 一行代码的矩形碰撞 (Priority: P2)

开发者用 `collide(ax, ay, aw, ah, bx, by, bw, bh)` 判断两个矩形是否重叠。

**Why this priority**: 每个动作游戏都要写，手写边界比较时 u8 回绕极易出错（002 的 shooter 就手写了 4 个比较）。

**Independent Test**: 对重叠、相邻（不重叠）、包含、靠近 255 边界等情况断言结果；参数含函数调用时只求值一次。

**Acceptance Scenarios**:

1. **Given** 两矩形部分重叠，**Then** 返回 1；仅边相接，**Then** 返回 0。
2. **Given** `x + w > 255`（如 x=250, w=16），**When** 判断，**Then** 不因 u8 回绕而误判。
3. **Given** 参数里有 `rand()`，**When** 调用，**Then** 每个参数恰好求值一次。

---

### User Story 5 - 内置音效 (Priority: P3)

开发者用 `sfx(SFX_JUMP)` 播放内置音效（`SFX_JUMP, SFX_COIN, SFX_SHOOT, SFX_HIT, SFX_EXPLODE`），
音效由 NMI 每帧推进（音高滑动 + 音量衰减），播放完自动静音；与 `play_tone` 互不打断。

**Why this priority**: 有声音反馈的游戏手感好得多，但不影响能否做出游戏，所以 P3。

**Independent Test**: 调用 `sfx` 后运行若干帧，断言 APU 写入日志中音量递减、周期滑动、结束后通道被关闭；
同时播放 `play_tone` 时 `$4015` 同时保留两个通道。

**Acceptance Scenarios**:

1. **Given** `sfx(SFX_JUMP)`，**When** 运行帧，**Then** 方波 2 的周期每帧递减，持续指定帧数后关闭。
2. **Given** `sfx(SFX_EXPLODE)`，**Then** 使用噪声通道。
3. **Given** 正在 `play_tone`，**When** `sfx(...)`，**Then** `$4015` 写入同时包含两个通道位；`stop_sound()` 停止全部。

### Edge Cases

- 队列容量 96 字节（含每条 3 字节头），足以在一个 vblank 内刷完；单次 `print` 超过 32 字节会被自动拆分。
- `vram_addr/vram_put/vram_fill` 仍然直接写入（它们依赖 PPU 地址状态，无法排队），文档注明只在 `ppu_off()` 期间使用。
- `metasprite` 槽位超过 63 时回绕；文档注明。
- `map_tile` 越界返回 0；`map` 中 `.` 映射为空格，其余字符为 ASCII 码。
- `map` / `metasprite` 不是保留字：仅在顶层 `map NAME =` / `metasprite NAME =` 形式下作为声明解析，不破坏已有标识符。
- `collide` 的 8 个参数必须是 u8。

## Requirements *(mandatory)*

### Functional Requirements

- **FR-301**: 渲染开启时，`print`、`print_u8`、`set_tile`、`load_palette` MUST 通过 RAM 队列在 NMI 中写入 VRAM；渲染关闭时直接写入。
- **FR-302**: 队列满时 MUST 等待下一帧刷新而不是丢弃数据；`ppu_off()` MUST 先刷新队列。
- **FR-303**: 语言 MUST 支持 `map NAME = [行字符串...];`（≤30 行，每行 ≤32 列）与内建 `draw_map(M)`、`map_tile(M, tx, ty) -> u8`。
- **FR-304**: 语言 MUST 支持 `metasprite NAME = [(dx, dy, tile[, attr]), ...];`（dx/dy 为 -128..127，≤64 块）与内建 `metasprite(slot, x, y, M) -> u8`。
- **FR-305**: 内建 `collide(ax, ay, aw, ah, bx, by, bw, bh) -> u8` MUST 按 9 位精度比较，每个参数只求值一次。
- **FR-306**: 内建 `sfx(id)` 与常量 `SFX_JUMP/COIN/SHOOT/HIT/EXPLODE` MUST 由 NMI 驱动播放，不影响 `play_tone`。
- **FR-307**: 新增 `set_tile(x, y, tile)` 写单个背景格。
- **FR-308**: 测试机 MUST 统计渲染开启且不在 vblank 时对 `$2006/$2007` 的写入次数。
- **FR-309**: 001/002 的全部测试 MUST 继续通过。

### Key Entities

- **VRAM 队列**：RAM `$0300-$035F`，条目格式 `[addr_hi, addr_lo, len, bytes...]`；零页 `VQ_LEN` 为已提交长度（最后一步写入，保证 NMI 只看到完整条目）。
- **Map**：ROM 中 960 字节（30×32）。
- **Metasprite**：ROM 中 `[count, (dx, dy, tile, attr)×count]`。
- **SFX 表**：每项 8 字节 `[channel, period_lo, period_hi, delta(i8), frames, max_vol, duty, pad]`（按 8 对齐便于用移位计算偏移）。

## Success Criteria *(mandatory)*

- **SC-301**: 示例游戏在主循环任意位置 `print`，测试机统计到的渲染期 VRAM 写入为 0。
- **SC-302**: 一个带墙壁、16×16 主角、碰撞和音效的迷宫示例 ≤ 120 行 NESL 逻辑代码（不含地图行与 `tile` 像素定义）。实测 `examples/maze.nesl` 为 97 行。
- **SC-303**: `cargo test --workspace` 全绿，001/002 无回归。

## Assumptions

- 队列刷新与 OAM DMA 共享 vblank 时间（约 2273 个 CPU 周期）：DMA 513 周期 + 队列 96 字节 × ~16 周期，留有余量。
- 不实现属性表（调色板分区）编辑与卷轴地图，留待后续特性。
- 音效固定为 5 个内置预设，暂不支持用户自定义音效。

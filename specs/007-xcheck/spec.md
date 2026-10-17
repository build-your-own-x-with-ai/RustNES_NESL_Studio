# Feature Specification: 第二模拟器交叉验证

**Feature Branch**: `007-xcheck`

**Created**: 2026-10-17

**Status**: Implemented

**Input**: User description: "第二模拟器交叉验证：用本机已有的 Node 跑无界面 jsnes，对所有示例 ROM 截帧，检查画面（状态栏分屏、卷轴、文字）与我们的测试机一致"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 测试机能「画」出画面 (Priority: P1) 🎯 MVP

测试机根据 VRAM、OAM、调色板和逐扫描线的滚动寄存器，渲染出 256×240 的 NES 颜色索引画面。

**Why this priority**: 现在测试机只记录状态、不出画面，无法与另一个模拟器的截图比较。

**Independent Test**: 单元测试构造背景图块、属性、调色板、精灵（翻转、优先级、遮挡顺序）和中途改滚动的寄存器日志，断言具体像素。

**Acceptance Scenarios**:

1. **Given** 背景图块与属性表，**When** 渲染，**Then** 像素颜色 = 对应调色板项；颜色 0 显示通用背景色。
2. **Given** 精灵在背景之后，**When** 背景像素不透明，**Then** 显示背景；透明时显示精灵。
3. **Given** 帧中途写 `$2000/$2005`，**When** 渲染，**Then** 在下一条扫描线起使用新的水平滚动。

---

### User Story 2 - 与 jsnes 逐像素比较 (Priority: P1)

`cargo test` 用 Node 运行 jsnes（编辑器用的同一版本），对示例 ROM 运行相同帧数，逐像素比较两边的画面；
不一致时在 `target/xcheck/` 输出两边与差异的 PNG。

**Why this priority**: 状态栏分屏、卷轴等目前只在自己写的模拟器里验证过——两边都按我的理解实现时，测试会一起错。

**Independent Test**: hello、sprite、runner（含状态栏 + 卷轴 + 音乐）、宽地图卷轴场景各比较一次。

**Acceptance Scenarios**:

1. **Given** 本机有 Node 与 `tools/xcheck/node_modules/jsnes`，**When** `cargo test`，**Then** 每个场景两边像素一致（或差异仅在文档化的容差内）。
2. **Given** 没有 Node 或未执行 `npm ci`，**When** `cargo test`，**Then** 测试打印跳过原因并通过（与 nestest 数据缺失时一致）。

### Edge Cases

- jsnes 默认裁掉四周 8 像素：比较时关闭（`clipToTvSize = false`），比较完整画面。
- 只比较画面静止的时刻（镜头停下、无输入变化），避免两个模拟器帧对齐方式不同带来的误报。
- 渲染器只支持 8×8 精灵、垂直镜像、无强调色/灰度（我们的运行时不使用这些）。

## Requirements *(mandatory)*

- **FR-701**: `nes_sim::render::frame(bus, k)` MUST 返回第 k 帧 256×240 的颜色索引。
- **FR-702**: `tools/xcheck/run.js` MUST 以固定版本 jsnes 运行 ROM 指定帧数并输出最后一帧与调色板。
- **FR-703**: 交叉验证测试 MUST 在不一致时输出 PNG 并给出不一致像素数与首个位置。

## Success Criteria *(mandatory)*

- **SC-701**: 4 个场景在两个模拟器中画面一致。
- **SC-702**: 发现的任何不一致都有结论：修复编译器/运行时、修复测试机，或记录为 jsnes 与真机的已知差异。

## Assumptions

- jsnes 1.2.1 足够接近真机（尤其 sprite-0 碰撞与中途改滚动）；它与真机的差异无法在本特性内验证。

## Findings (T706)

| 发现 | 结论 |
|---|---|
| 卷轴场景右边缘 6 像素不同 | jsnes 每行只画 32 个背景图块，精细水平滚动 f>0 时最右 f 像素留作背景色（硬件会取第 33 个图块）。jsnes 的缺陷：比较时排除这些像素，并断言 jsnes 在那里只显示背景色。 |
| hello / sprite / runner / 卷轴+状态栏 | 其余像素全部一致；状态栏分屏行、sprite-0 碰撞位置、调色板分区、改写的地图格子两边相同。 |
| 编译器 / 运行时 | 未发现缺陷。 |

**验证这个检查有效（变异测试）**：分别把测试机的 sprite-0 碰撞提前 8 行、推迟 8 行、把渲染器的分屏推迟 3 行，
交叉验证都会失败（651 / 1011 / 450 个像素不同）。注意：它检查的是「测试机模型与 jsnes 一致」；
ROM 本身的逻辑错误两边会一起错，那部分仍由 001-006 的端到端测试负责。

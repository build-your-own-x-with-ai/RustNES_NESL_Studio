# Feature Specification: NESL Studio 编辑器增强

**Feature Branch**: `008-studio`

**Created**: 2026-10-17

**Status**: Implemented

**Input**: User description: "编辑器增强：可视化图块/地图/调色板编辑器（点格子画，自动写回 tile / map / PAL 代码）、内建函数补全与悬停说明、输入时实时报错"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 画图块，自动写回代码 (Priority: P1) 🎯 MVP

「图块」标签页列出源码中的 `tile` 声明（也可新建），在 8×8 网格上点格子、按住拖动画像素，选 0-3 号颜色；
每次修改立即改写源码里对应的 `tile N = [...]`，其余代码一字不动。

**Why this priority**: 手写 `"..3333.."` 字符串画图是最不像「编辑器」的地方。

**Independent Test**: 纯函数 `findTiles / upsertTile` 用 Node 单元测试；端到端：用它改写示例源码后交给编译器，断言 CHR 中的图块字节。

**Acceptance Scenarios**:

1. **Given** 源码有 `tile 1 = [...]`，**When** 在编辑器里改一个像素，**Then** 只有那一行字符串变化，编译后 CHR 图块 1 对应位改变。
2. **Given** 新建图块 7，**When** 保存，**Then** 源码末尾追加 `tile 7 = [...]`（8 行），可直接编译。
3. **Given** `tile '#' = [...]`，**When** 编辑，**Then** 保留 `'#'` 写法。

---

### User Story 2 - 补全、悬停说明、实时报错 (Priority: P1)

输入标识符时弹出补全（内建函数带参数签名、关键字、`BTN_*`/`SFX_*`、源码里定义的名字），回车/Tab 插入；
鼠标停在内建函数上显示中文说明；停止输入 0.5 秒后自动检查并标出错误行（不运行游戏）。

**Why this priority**: 32 个内建函数靠记是记不住的；报错要点「运行」才看到也太慢。

**Independent Test**: 内建函数说明表与编译器签名表的一致性由 Rust 测试保证；补全排序、取词由 Node 单元测试；
`/api/builtins`、`/api/check` 由服务器测试。

**Acceptance Scenarios**:

1. **Given** 输入 `spr`，**Then** 第一个候选是 `sprite(i, x, y, tile, attr)`。
2. **Given** 编译器新增一个内建函数却没写说明，**When** `cargo test`，**Then** 测试失败。
3. **Given** 第 3 行引用未定义变量，**When** 停止输入，**Then** 行号 3 标红，无需点运行。

---

### User Story 3 - 地图与调色板编辑 (Priority: P2)

「地图」标签页把 `map` 用真实图块画出来（取自上一次编译的 ROM 的 CHR 与调色板），选一个字符后点格子改写地图；
「调色板」标签页显示 `PAL`（`load_palette` 用的常量数组）的 32 个颜色，点击后从 NES 64 色中选择，写回数组。

**Independent Test**: `findMaps / setMapCell / findPalette / setPaletteEntry` Node 单元测试；端到端编译断言地图字节与调色板。

### Edge Cases

- 写回时只替换对应声明的文本范围；其他代码、注释与格式不变。
- 地图行短于目标列时用 `.` 补齐。
- 调色板数组项是数字字面量时才可编辑（`0x0F` 形式写回）；含表达式时只读并提示。
- 没有网络（jsnes 未加载）时编辑功能照常可用。

## Requirements *(mandatory)*

- **FR-801**: `nesl::builtins::DOCS` MUST 为每个内建函数提供签名与中文说明，与 `signature()` 一一对应（测试强制）。
- **FR-802**: 服务器 MUST 提供 `GET /studio.js`、`GET /api/builtins`、`POST /api/check`（只返回诊断）。
- **FR-803**: 编辑器 MUST 提供图块、地图、调色板可视化编辑，写回源码。
- **FR-804**: 编辑器 MUST 提供补全、悬停说明、输入后自动检查。

## Success Criteria *(mandatory)*

- **SC-801**: 写回逻辑的单元测试与「写回 → 编译 → 断言 ROM」端到端测试全绿。
- **SC-802**: `cargo test --workspace` 全绿（有 Node 时包含 JS 测试）。

## Assumptions

- 界面交互（拖动画像素、弹出框位置）无法自动化验证，需人工验收；浏览器自动化在本机目前不可用。

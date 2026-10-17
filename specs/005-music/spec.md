# Feature Specification: 音乐与自定义音效

**Feature Branch**: `005-music`

**Created**: 2026-10-17

**Status**: Implemented

**Input**: User description: "音乐与自定义音效：用文本写背景音乐（类似 MML：\"C4 E4 G4 >C\"），NMI 中自动播放并循环；自定义音效声明，不再只有 5 个预设"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 用文本写旋律 (Priority: P1) 🎯 MVP

开发者用 `music THEME = "T140 O4 L8 C E G >C< G E C R";` 写旋律，`play_music(THEME)` 开始播放，
音乐由 NMI 每帧推进，播完自动从头循环；`stop_music()` 停止。

**Why this priority**: 目前游戏只有音效没有音乐，这是最明显的缺口；单声部旋律就已经有价值。

**Independent Test**: 编译器把 MML 编成确定的事件字节（单元测试逐字节断言）；端到端在测试机里逐帧运行，
断言方波 1 的周期寄存器在预期的帧写入预期的值，且循环后重复。

**Acceptance Scenarios**:

1. **Given** `T150 L4 C D E`，**When** 编译，**Then** 每个四分音符 24 帧（23 帧发声 + 1 帧断奏间隙）。
2. **Given** `O4 A`，**When** 播放，**Then** 方波 1 周期 = 253（A4 = 440 Hz）。
3. **Given** 附点 `C4.`、连音 `C4&C8`、升降号 `C+ D# E-`、`>` `<`、`L`、`R`、小节线 `|`，**When** 编译，**Then** 时值与音高正确。
4. **Given** 歌曲结束，**When** 继续运行，**Then** 从头循环，且循环处时序不漂移（总帧数 = 理论值四舍五入）。
5. **Given** 未知命令、音符超出范围、没有任何音符的乐谱，**When** 编译，**Then** 报错并指向字符串中的具体列。

---

### User Story 2 - 多声部 (Priority: P2)

`play_music(MELODY, BASS)` 或 `play_music(MELODY, BASS, HARMONY)`：旋律用方波 1、低音用三角波、
和声用方波 2；三个声部各自计时、独立循环。

**Why this priority**: 有低音的音乐才像 NES 音乐；但单声部已可用，所以 P2。

**Independent Test**: 断言三角波寄存器 `$400A/$400B` 写入低音周期（三角波同音高周期约为方波的一半）、
休止时三角波在 `$4015` 中被关闭；三个声部的写入时序各自正确。

**Acceptance Scenarios**:

1. **Given** 低音 `O3 A`，**When** 播放，**Then** `$400A` = 253（三角波 A3）。
2. **Given** 只有旋律，**When** `play_music(M)`，**Then** 其他声部静音。
3. **Given** 旋律含低于 A1 的音，**When** 作为旋律声部编译，**Then** 报错（方波无法发出）；作为低音则允许（最低 A0）。

---

### User Story 3 - 自定义音效 (Priority: P2)

`sound LASER = (pulse, 0x30, -4, 10);`（通道、起始周期、每帧周期变化、帧数[, 音量[, 占空比]]），
`sfx(LASER)` 播放；与内置 `SFX_*` 共用同一引擎。

**Independent Test**: 断言 `sfx(LASER)` 后方波 2 的控制/周期寄存器按声明变化，噪声音效写噪声寄存器；参数越界报错。

**Acceptance Scenarios**:

1. **Given** `sound S = (pulse, 0x120, 3, 6, 9, 1)`，**When** `sfx(S)`，**Then** `$4007` = 1、`$4006` 从 0x20 起每帧 +3、音量不超过 9、占空比位 = `0b01`。
2. **Given** `sound N = (noise, 5, 0, 20)`，**Then** 写 `$400E` = 5。
3. **Given** 通道不是 `pulse`/`noise`、周期越界、帧数为 0、超过 27 个自定义音效，**When** 编译，**Then** 报错。

---

### User Story 4 - 音效与音乐共存 (Priority: P2)

音效（方波 2 / 噪声）播放时暂时盖过和声声部，旋律和低音不受影响；音效结束后和声在下一个音符恢复。
`stop_sound()` 停止全部声音（含音乐）。

**Independent Test**: 音效持续期间方波 2 寄存器中不出现和声音符的周期；之后再次出现；旋律写入不中断。

### Edge Cases

- 默认值：`T120 O4 L4 V12 @2`；`T` 范围 30-300，`O` 0-7，`V` 0-15，`@` 0-3，长度 1-64。
- 太短（0 帧）的音符被跳过但计入时间；超过 255 帧的音符拆成多个事件（不重新触发）。
- 每个音符的最后一帧静音（断奏），`&` 连接的音符之间不加间隙。
- `play_tone` 与旋律声部共用方波 1：音乐播放时下一个音符会覆盖它。
- `music` / `sound` 是上下文关键字，仍可作为普通标识符。

## Requirements *(mandatory)*

- **FR-501**: 语言 MUST 支持 `music NAME = "MML";` 或 `music NAME = ["...", "..."];`。
- **FR-502**: MML MUST 支持音符 `A-G`（`+`/`#` 升、`-` 降）、`R`、长度数字与附点、`L`、`O`、`>`、`<`、`T`、`V`、`@`、`&`、`|`、空白。
- **FR-503**: 内建 `play_music(m1[, m2[, m3]])`、`stop_music()`；NMI 每帧推进，自动循环。
- **FR-504**: 语言 MUST 支持 `sound NAME = (pulse|noise, period, delta, frames[, volume[, duty]]);`，名称是可传给 `sfx` 的常量。
- **FR-505**: 编译期计算音高周期表（NTSC 1.789773 MHz），时值用累计时间四舍五入避免漂移。
- **FR-506**: 001-004 的全部测试 MUST 继续通过。

## Success Criteria *(mandatory)*

- **SC-501**: 循环 10 次后，每个声部的事件仍在理论帧号 ±0 帧处触发。
- **SC-502**: 至少一个示例带有两声部以上的背景音乐，且与音效共存。
- **SC-503**: `cargo test --workspace` 全绿。

## Assumptions

- 不支持 DPCM 采样、颤音/包络、循环片段语法（`[...]n`），留待后续。
- 用户自定义音效与内置音效合计最多 32 个。

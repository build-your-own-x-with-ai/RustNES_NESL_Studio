# Feature 020: Super Mario Bros 反编译到 C（静态重编译）

## 目标

在 019（字节级汇编往返）的基础上，把 SMB 反编译为**高级语言源码**，
并保持 019 的验收标准：生成的程序可编译、可运行，与原始 ROM 对比
**图像与运动逐像素一致**（声音不要求）。

## 目标语言选型：C，而非 NESL

| 维度 | NESL | C（静态重编译） |
|------|------|----------------|
| 控制流表达 | 结构化，无 goto，无法机械重建任意汇编控制流 | goto/switch/函数，可逐指令忠实翻译 |
| 重编译后时序 | nesc 重新布局代码，指令数/周期完全不同，分屏卷轴（状态栏）等时序行为无法逐像素复原 | 翻译时携带精确周期计数，与 nes-sim 的 CPU 周期模型对齐 |
| 可验证性 | 帧级一致无法保证（018 spec 已论证） | 帧级一致可测试 |

018 的可行性分析结论"完全自动化 NESL 反编译几乎不可能"依然成立。
本 feature 采用**静态重编译**（static recompilation）：每条 6502 指令
翻译为等价 C 语句，JSR 目标翻译为 C 函数，JumpEngine 跳转表翻译为
`switch`，数据仍以字节数组承载。这是能同时满足"高级语言"与
"帧级一致"两个要求的唯一工程路线。

## 功能需求

### FR-1 C 代码生成器

输入：019 的追踪结果（指令、标签、跳转表）。输出：单个自包含 `smb.c`：

1. **机器状态**：`a/x/y/sp`、独立标志变量（n/z/c/v/i/d）、`ram[0x800]`、
   64 位周期计数器 `cyc`。
2. **函数划分**：每个 JSR 目标 / 跳转表项 / 中断向量入口 = 一个 C 函数
   `sub_XXXX()`（向量入口命名 `reset`/`nmi_handler`/`irq_handler`）。
   函数体 = 从入口经分支/JMP 可达的指令块（不跨调用边界）；
   被多个函数到达的块在每个函数内复制（尾部复制）。
   `JMP 其他入口` → 尾调用 `sub_X(); return;`。
3. **JumpEngine 调用点** → `switch (a) { case 0: sub_T0(); return; … }`。
4. **周期精确**：按 nes-sim `cpu.rs` 的模型（基础周期表、分支命中 +1/
   跨页 +2、索引寻址跨页 +1、JSR/RTS 6、RTI 6、NMI 入口 7、OAM DMA 513、
   reset 起始 cyc=7）。周期按基本块累加，I/O 访问前先结清。
5. **I/O 模型**：移植 nes-sim `NesBus`（mapper 0 子集）到 C：
   PPU 寄存器（$2002 含 sprite-0 hit 周期判定、$2005/$2006 toggle、
   $2007 缓冲读、镜像折叠）、$4014 OAM DMA、$4016/17 手柄移位、
   APU 写入忽略。`$2000/$2005` 写入记录 `(cyc, kind, val)` 日志
   （与 nes-sim `reg_log` 同构，供渲染对比）。
6. **可读性**：经行为验证的 RAM 符号表（`#define FrameCounter 0x09` 等，
   见 FR-3）；每个函数标注原始地址范围；跳转表 switch 标注表地址。
7. **运行驱动**：生成的 `main()` 接受脚本文件（逐帧按键 + 检查点），
   `reset()` 执行到主线程停机循环后返回，此后每帧调用 NMI；
   检查点处把 `vram/oam/ppuctrl/ppumask/reg_log/ram/frames` 写入
   快照文件，供 Rust 测试用 nes-sim 的渲染器重建帧并对比。

### FR-2 CLI

`nesc decompile-c <rom.nes> -o <out.c>`（NROM；生成后打印统计）。

### FR-3 经验证的符号表

仅收录在 nes-sim 中行为验证通过的地址（2026-10-25 实测）：
`$09 FrameCounter`（逐帧 +1）、`$0770 OperMode`（标题 0 → 游戏 1）、
`$86 Player_X_Position`、`$6D Player_PageLoc`、`$CE Player_Y_Position`
（跳跃时减小）、`$57 Player_X_Speed`、`$1D Player_State`、
`$075A NumberofLives`（初值 2）、`$075F WorldNumber`、`$0760 AreaNumber`、
`$06FC SavedJoypadBits`（= 手柄位）、`$073F HorizontalScroll`、
`$0776 GamePauseStatus`。命名沿用社区反汇编（smbdis）惯例。

### FR-4 验证（TDD 验收标准）

| # | 测试 | 验收 |
|---|------|------|
| 1 | 生成与编译 | `generate_c()` 产出；`cc` 编译零错误 |
| 2 | 标题+演示 | C 版快照经 nes-sim 渲染，与原版 ROM 第 30/60/600/700 帧逐像素一致；演示画面与标题差异 >1000 像素 |
| 3 | 操作脚本 | Start→1-1→右+B+间歇跳，检查点帧逐像素一致；原版卷轴 ≥30 个不同值（运动真实发生） |
| 4 | 可读性 | 含 `switch` 跳转表、≥100 个函数、符号表被使用 |

## 非目标

- 声音：APU 写入忽略（nes-sim 亦只记录）。
- 不要求消除 goto / 完全结构化控制流。
- 不支持 mapper ≠ 0。

## 已知风险与缓解

- **周期漂移**：C 按基本块累加周期，$2002 轮询循环内的判定点与
  nes-sim 的指令起点语义有 <10 周期误差。SMB 的分屏在 sprite-0 等待
  循环处重新同步（轮询直到 `cyc ≥ 命中周期`），vblank 等待同理，
  误差不跨扫描线（一行 ≈ 113.67 周期），帧渲染不受影响。
- **JumpEngine 以外的栈技巧 / 间接 JMP**：翻译为 `dispatch(addr)`
  （全入口 switch）或显式 `fatal()`，测试运行即暴露。
- **init 末尾开 NMI 到停机之间跨帧界**：概率极低且确定性，
  测试帧对比直接暴露。

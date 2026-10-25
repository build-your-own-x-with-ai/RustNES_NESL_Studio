# Feature 020 完成报告

## 结论

✅ **Super Mario Bros 已反编译为高级语言（C），生成的程序可编译、可运行，
与原始 ROM 在 nes-sim 中逐像素一致（标题、attract 演示、实际操作脚本）。**

```
原版 ROM ─ nesc decompile-c ─▶ output/smb/smb.c（22900 行，459 个函数，18 个跳转表 switch）
                                   │ cc -O1
                                   ▼
                              ./smb <script>  ──快照──▶ nes-sim 渲染器 ──▶ 与原版逐像素对比 ✅
```

## 为什么是 C 而不是 NESL

验收标准要求"图像与运动一致"。NESL 经 nesc 重新编译后代码布局与指令
时序完全不同，SMB 依赖周期级时序的分屏卷轴（状态栏 sprite-0 等待）无法
逐像素复原（018 的 spec 已论证"完全自动化 NESL 反编译几乎不可能"）。
C 静态重编译逐指令翻译并携带与 nes-sim 对齐的周期计数，是同时满足
"高级语言"与"帧级一致"的唯一工程路线。

## 测试结果（crates/nesl/tests/smb_c_decompile.rs，4/4 通过）

| # | 测试 | 结果 |
|---|------|------|
| 1 | `generate_c()` 产出，`cc -O1` 编译零错误 | ✅ |
| 2 | 标题/演示：第 30/60/600/700 帧逐像素一致；演示与标题差异 >1000 像素 | ✅ |
| 3 | 操作脚本：Start→1-1→右+B+间歇跳，第 264/464/664/863 帧逐像素一致；卷轴 ≥30 个不同值 | ✅ |
| 4 | 可读性：459 个函数、18 个 `switch` 跳转表、经验证符号表（`ram[FrameCounter]` 等）被使用 | ✅ |

全 workspace 回归（nes-sim / nesl / nesc，含 019 的 5 项）无失败。

## 实现要点

- **函数划分**：入口 = 中断向量 ∪ JSR 目标 ∪ 跳转表项（459 个）；函数体 =
  入口经分支/JMP 可达的块；跨入口跳转 → 尾调用；入口不在最低地址时函数
  开头 `goto l_入口`（SMB 的 `UpdateScreen` 等例程写屏循环在入口之前）。
- **JumpEngine → switch**：翻译时复现其残留副作用（`$04/$05` 表指针、
  `$06/$07` 目标地址、Y=2A+2、A=目标高字节），标题画面解码依赖这些残留。
- **周期模型**：基本块累计、I/O 前结清；分支命中 +1/跨页 +2 翻译期算出；
  索引跨页 +1 在读助手内判定；JumpEngine 本体 43 周期。
- **C 运行时**：移植 nes-sim `NesBus`（mapper 0）：PPU 寄存器、$2007 缓冲读、
  OAM DMA(+513)、手柄移位、sprite-0 惰性锁存（任何 I/O 前完成本帧锁存）
  + 重判、$2006 装载 t、`reg_log` 事件日志。

## 顺带发现并修复的 nes-sim PPU 模型缺陷

调试操作脚本时发现**原版 ROM 在 nes-sim 中于第 460 帧起冻结**（019 的
对比测试未发现：原版与重建版冻结得一模一样）。原因是 SMB 跨命名表时的
列写入过长，两处 PPU 行为未建模：

1. 渲染在可见区内重新开启后，sprite-0 仍应在其扫描线命中（旧模型只在
   第一个像素处一次性锁存）→ `$2001` 写入时重判。
2. `$2006` 写入装载 PPU 的 t 寄存器（命名表位 / coarse x,y / fine y）；
   `UpdateScreen` 收尾的 `$2006 <- $0000` 把命名表位重置为 0，状态栏仍从
   NT0 取 → 新增 `nt_sel`，渲染与 sprite-0 判定改用有效命名表位。

两项修复均按 TDD（红 `8ab3d13` → 绿 `b7510cc`），nes-sim 新增 4 个单测。

## 复现

```bash
cargo run -p nesc -- decompile-c "roms/FC_ROMS-main/ROM/0066/Super Mario Bros. (W) [!].nes" -o output/smb/smb.c
cc -O1 -o smb output/smb/smb.c
printf 'btn 60 16\nbtn 64 0\nbtn 264 65\ndump 400 /tmp/f400.bin\nend 401\n' > s.txt && ./smb s.txt
cargo test -p nesl --test smb_c_decompile
```

脚本命令：`btn <帧> <掩码>`（A=128 B=64 Select=32 Start=16 上=8 下=4 左=2 右=1）、
`dump <帧> <路径>`（写 RAM/VRAM/OAM/寄存器日志快照）、`end <帧>`。

## 限制

- 声音：APU 写入忽略（spec 非目标）。
- 控制流保留 goto（可读性靠函数划分、switch、符号表、地址注释）。
- 仅 mapper 0；跳转表以外的间接 JMP 走 `dispatch()` 全入口 switch。

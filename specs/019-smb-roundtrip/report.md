# Feature 019 完成报告

## 结论

✅ **Super Mario Bros 反编译完成，重建 ROM 与原版逐字节一致，
图像与运动经 nes-sim 帧级对比验证完全一致。**

```
原版 ROM（40976 字节，SHA1 ea343f4e…）
   │  nesc disasm-full
   ▼
output/smb/smb.asm（13245 行，可读可编辑）+ smb.chr（8192 字节）
   │  nesc asm
   ▼
output/smb/smb_rebuilt.nes —— cmp 逐字节一致 ✅
```

## 测试结果（crates/nesl/tests/smb_roundtrip.rs，5/5 通过）

| # | 测试 | 结果 |
|---|------|------|
| 1 | 指令表互逆（151 条官方指令） | ✅ |
| 2 | 往返逐字节一致（40976/40976） | ✅ |
| 3 | 标题画面：第 30/60 帧逐像素一致，≥5 色非空白 | ✅ |
| 4 | Attract 演示：第 600/700 帧一致，与标题差异 >1000 像素（游戏在动） | ✅ |
| 5 | 操作脚本：Start→1-1，右+B+间歇跳，检查点帧一致，600 帧内卷轴 ≥30 个不同值 | ✅ |

全 workspace 回归测试无失败。

## 反汇编统计

- PRG 32768 字节全覆盖：约 22.7KB 识别为指令，9496 字节 `.byte` 数据
  （关卡数据、布局表、音乐数据），250 条 `.word`（跳转表项 + 中断向量）
- 1641 个标签（含 Reset / NMI / IRQ 入口命名）
- JumpEngine 启发式成功识别 SMB 的跳转表惯用法，表项作为代码入口追踪

## 声音

按 spec 非目标：APU 写入字节原样保留在源码中（字节一致 ⇒ 声音数据也一致），
但未做音频输出验证。

## 复现步骤

```bash
cargo run -p nesc -- disasm-full "roms/FC_ROMS-main/ROM/0066/Super Mario Bros. (W) [!].nes" -o output/smb/smb.asm
cargo run -p nesc -- asm output/smb/smb.asm -o output/smb/smb_rebuilt.nes
cmp "roms/FC_ROMS-main/ROM/0066/Super Mario Bros. (W) [!].nes" output/smb/smb_rebuilt.nes
cargo test -p nesl --test smb_roundtrip
```

## 与 014/015/018 的关系

高层 NESL 反编译（014/015/018）作为探索保留；本 feature 采用
业界标准的"可重建汇编源码"路线（同 smbdis.asm），是唯一能保证
帧级一致的反编译形式。smb.asm 可直接编辑后用 `nesc asm` 重建，
为后续的改版/移植提供可靠底座。

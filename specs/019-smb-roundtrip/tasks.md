# Feature 019 任务清单

## 红：失败测试

- [x] T1 `test:` crates/nesl/tests/smb_roundtrip.rs（commit fb581a1）
  - [x] T1.1 指令表互逆 + 官方指令数 151
  - [x] T1.2 SMB 往返逐字节一致
  - [x] T1.3 标题画面帧一致（检查点 + 非空白断言）
  - [x] T1.4 Attract 演示帧一致 + 画面随时间变化
  - [x] T1.5 操作脚本帧一致 + 水平卷轴前进
  - [x] T1.6 asm.rs: Mode pub + opcode_table() 桩；roundtrip/asm_text 模块桩

## 绿：实现

- [x] T2 `feat:` asm.rs opcode_table()（测试 1 验证表完整性）
- [x] T3 `feat:` disasm/roundtrip.rs（commit 9b86bce）
- [x] T4 `feat:` disasm/asm_text.rs（commit 9b86bce）
- [x] T5 全部 T1 测试转绿（5/5 通过，全 workspace 回归无失败）

## 工件与收尾

- [x] T6 `feat:` nesc CLI：disasm-full / asm 命令（commit 0958afa）
- [x] T7 生成 output/smb/：smb.asm、smb.chr、smb_rebuilt.nes（cmp 逐字节一致）
- [x] T8 `docs:` 完成报告 specs/019-smb-roundtrip/report.md
- [x] T9 merge --no-ff 到 main

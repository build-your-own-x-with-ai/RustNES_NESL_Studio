# Feature 019 任务清单

## 红：失败测试

- [ ] T1 `test:` crates/nesl/tests/smb_roundtrip.rs：
  - [ ] T1.1 指令表互逆 + 官方指令数 151
  - [ ] T1.2 SMB 往返逐字节一致
  - [ ] T1.3 标题画面帧一致（检查点 + 非空白断言）
  - [ ] T1.4 Attract 演示帧一致 + 画面随时间变化
  - [ ] T1.5 操作脚本帧一致 + 水平卷轴前进
  - [ ] T1.6 asm.rs: Mode pub + opcode_table() 桩；roundtrip/asm_text 模块桩（返回 Err）

## 绿：实现

- [ ] T2 `feat:` asm.rs opcode_table()（含表完整性单测）
- [ ] T3 `feat:` disasm/roundtrip.rs：iNES 解析/重建、追踪反汇编、JumpEngine 启发式、文本发射
- [ ] T4 `feat:` disasm/asm_text.rs：文本汇编器
- [ ] T5 全部 T1 测试转绿

## 工件与收尾

- [ ] T6 `feat:` nesc CLI：disasm-full / asm 命令
- [ ] T7 生成 output/smb/：smb.asm、smb.chr、smb_rebuilt.nes + 验证记录
- [ ] T8 `docs:` 完成报告 specs/019-smb-roundtrip/report.md
- [ ] T9 merge --no-ff 到 main

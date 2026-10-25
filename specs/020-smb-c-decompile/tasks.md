# Feature 020 任务清单

## 红：失败测试

- [ ] T1 `test:` crates/nesl/tests/smb_c_decompile.rs + c_codegen 桩
  - [ ] T1.1 生成 + cc 编译零错误
  - [ ] T1.2 标题/演示帧逐像素一致 + 演示在动
  - [ ] T1.3 操作脚本帧一致 + 卷轴前进
  - [ ] T1.4 可读性：switch 跳转表、≥100 函数、符号表使用

## 绿：实现

- [ ] T2 `feat:` roundtrip.rs 重构：trace_rom() + jump_tables（019 测试保持绿）
- [ ] T3 `feat:` c_codegen.rs：函数划分 + 指令翻译 + 周期 + 运行时模板
- [ ] T4 全部 T1 测试转绿

## 工件与收尾

- [ ] T5 `feat:` nesc CLI：decompile-c 命令
- [ ] T6 生成 output/smb/smb.c + 编译验证记录
- [ ] T7 `docs:` 报告 report.md；merge --no-ff 到 main

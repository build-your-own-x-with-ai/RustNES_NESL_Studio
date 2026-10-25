# Feature 020 任务清单

## 红：失败测试

- [x] T1 `test:` crates/nesl/tests/smb_c_decompile.rs + c_codegen 桩
  - [x] T1.1 生成 + cc 编译零错误
  - [x] T1.2 标题/演示帧逐像素一致 + 演示在动
  - [x] T1.3 操作脚本帧一致 + 卷轴前进
  - [x] T1.4 可读性：switch 跳转表、≥100 函数、符号表使用

## 绿：实现

- [x] T2 `feat:` roundtrip.rs 重构：trace_rom() + jump_tables（019 测试保持绿）
- [x] T3 `feat:` c_codegen.rs：函数划分 + 指令翻译 + 周期 + 运行时模板（commit 4acf134）
- [x] T3b nes-sim PPU 模型缺陷修复（调试中发现，红 8ab3d13 → 绿 b7510cc）
- [x] T4 全部 T1 测试转绿（4/4，全 workspace 回归无失败）

## 工件与收尾

- [x] T5 `feat:` nesc CLI：decompile-c 命令
- [x] T6 生成 output/smb/smb.c（22900 行，459 函数）+ cc 编译验证
- [x] T7 `docs:` 报告 report.md；merge --no-ff 到 main

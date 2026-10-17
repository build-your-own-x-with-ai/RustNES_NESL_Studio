---
description: "Task list for second-emulator cross-checking"
---

# Tasks: 第二模拟器交叉验证

- [x] T701 创建分支 `007-xcheck` 与 spec/plan/tasks 文档；安装固定版本 jsnes 到 `tools/xcheck`
- [x] T702 [US1] 测试：渲染器像素（背景、属性、通用背景色、精灵优先级/翻转/顺序、中途滚动）`crates/nes-sim/src/render.rs`
- [x] T703 [US1] 实现渲染器
- [ ] T704 [US2] Node 运行器 `tools/xcheck/run.js`
- [ ] T705 [US2] 交叉验证测试（4 个场景、缺失时跳过、不一致输出 PNG）`crates/nesl/tests/xcheck.rs`
- [ ] T706 分析并处理发现的不一致
- [ ] T707 README（如何运行）；clippy 清零；合并到 `main`

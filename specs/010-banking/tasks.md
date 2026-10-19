---
description: "Task list for UNROM map banking"
---

# Tasks: 地图换 bank

- [x] T1001 分支、实测（spec.md）、确认范围
- [x] T1002 测试 + 实现：测试机 mapper 2（bank 切换、固定 bank、CHR-RAM、总线冲突计数）
- [x] T1003 测试 + 实现：汇编器 `Equ`（只定义符号不占空间）
- [x] T1004 测试：大程序自动 UNROM（头、读取/绘制/卷轴/列表、CHR 复制、无总线冲突、错误）
- [x] T1005 实现：编译两次（NROM 失败再 UNROM）、bank 分配、`__bank`、`__chr_copy`、地图函数调用前切 bank
- [x] T1006 COIN QUEST 6 个世界 + 测试 + jsnes 交叉验证
- [ ] T1007 文档；clippy；合并

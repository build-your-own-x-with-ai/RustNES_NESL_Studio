---
description: "Task list for horizontal scrolling"
---

# Tasks: 横向卷轴与多屏关卡

**Input**: `/specs/004-scrolling/`

**Tests**: TDD —— 每个故事先提交失败测试，再实现。

## Phase 1: Setup

- [x] T401 创建分支 `004-scrolling` 与 spec/plan/contracts/tasks 文档

## Phase 2: Foundational

- [x] T402 测试 + 实现：parser `palette_zone` 上下文声明 `crates/nesl/src/{ast,parser}.rs`

## Phase 3: US1 宽地图 (P1) 🎯 MVP

- [x] T403 [US1] 测试：64/128 列 draw_map 写两个命名表、map_tile u16 列号与越界、255 列上限、003 回归 `crates/nesl/tests/scrolling.rs`
- [x] T404 [US1] 实现：列存储、纵向队列条目、`__col_addr`、新 `__draw_map` / `__map_tile`

## Phase 4: US2 镜头 (P1)

- [x] T405 [US2] 测试：跟随/钳制/限速、往返 4 屏可见列全部正确、滚动寄存器、渲染关闭时直写、零违规
- [x] T406 [US2] 实现：`__camera_follow`、`__stream_col`、`__stream_attr`、`camera_x`

## Phase 5: US3 调色板分区 (P2)

- [x] T407 [US3] 测试：属性字节打包、卷轴补画属性、错误
- [x] T408 [US3] 实现：分区 → 属性表数据

## Phase 6: US4 示例与收尾 (P2)

- [x] T409 示例 `examples/runner.nesl` + 端到端测试；编辑器示例与高亮
- [x] T410 README / 语言参考；clippy 清零
- [x] T411 合并到 `main`

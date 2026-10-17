---
description: "Task list for editable maps and the status bar"
---

# Tasks: 可修改地图与固定状态栏

**Input**: `/specs/006-map-edit-status/`

**Tests**: TDD —— 每个故事先提交失败测试，再实现。

## Phase 1: Setup

- [x] T601 创建分支 `006-map-edit-status` 与 spec/plan/contracts/tasks 文档

## Phase 2: Foundational

- [x] T602 测试 + 实现：测试机 sprite-0 碰撞与 `$2000/$2005` 写入日志 `crates/nes-sim`
- [ ] T603 共享测试辅助函数（宽地图、命名表读取）移到 `crates/nesl/tests/common/mod.rs`

## Phase 3: US1 改写地图 (P1) 🎯 MVP

- [ ] T604 [US1] 测试：读回、可见格更新、卷轴往返保持、屏幕外补画、多地图隔离、reset_map、容量与返回值、不用时不占 RAM `crates/nesl/tests/map_edit.rs`
- [ ] T605 [US1] 实现：覆盖表、`__ov_find`、`__map_tile`、`__stream_col` 缓冲、`__set_map_tile`、`__reset_map`、预扫描保留 RAM

## Phase 4: US2 状态栏 (P2)

- [ ] T606 [US2] 测试：分屏时序、补画跳过状态栏行、状态栏文字保留、关闭、超时不卡死、参数错误 `crates/nesl/tests/status_bar.rs`
- [ ] T607 [US2] 实现：CHR 标记、`__status_bar`、NMI 分屏、补画/属性跳过行

## Phase 5: US3 示例

- [ ] T608 runner 改用 `set_map_tile` 与状态栏 + 测试（金币不复活）

## Phase 6: Polish

- [ ] T609 README / 语言参考 / 数据模型；编辑器高亮；clippy 清零
- [ ] T610 合并到 `main`

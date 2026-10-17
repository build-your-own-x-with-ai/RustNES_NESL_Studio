---
description: "Task list for music and custom sound effects"
---

# Tasks: 音乐与自定义音效

**Input**: `/specs/005-music/`

**Tests**: TDD —— 每个故事先提交失败测试，再实现。

## Phase 1: Setup

- [x] T501 创建分支 `005-music` 与 spec/plan/contracts/tasks 文档

## Phase 2: Foundational

- [x] T502 测试 + 实现：parser `music` / `sound` 上下文声明 `crates/nesl/src/{ast,parser}.rs`

## Phase 3: US1 旋律 (P1) 🎯 MVP

- [x] T503 [US1] 测试：MML 编码逐字节（音符、长度、附点、连音、升降、八度、速度、拆分、错误列号）`crates/nesl/src/mml.rs`
- [x] T504 [US1] 实现：`mml::compile`
- [x] T505 [US1] 测试：端到端帧级时序、A4 周期、循环不漂移、stop_music `crates/nesl/tests/music.rs`
- [x] T506 [US1] 实现：周期表、`__music_tick`、`play_music`、`stop_music`、零页 `NMI_PTR`

## Phase 4: US2 多声部 (P2)

- [ ] T507 [US2] 测试：三角波低音、和声、未用声部静音、方波音域检查
- [ ] T508 [US2] 实现：三声部

## Phase 5: US3 自定义音效 (P2)

- [ ] T509 [US3] 测试：sound 声明写寄存器、噪声、参数错误
- [ ] T510 [US3] 实现：音效表移入代码生成、`sound` 常量

## Phase 6: US4 共存 (P2)

- [ ] T511 [US4] 测试：音效期间和声让出、之后恢复、stop_sound 停止音乐
- [ ] T512 [US4] 实现（若 T508 未覆盖）

## Phase 7: Polish

- [ ] T513 runner 示例加入三声部音乐 + 测试；编辑器示例与高亮
- [ ] T514 README / 语言参考 / 数据模型；clippy 清零
- [ ] T515 合并到 `main`

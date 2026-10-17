---
description: "Task list for game development API"
---

# Tasks: 游戏开发 API

**Input**: `/specs/003-game-api/`（spec.md、plan.md、contracts/api.md）

**Tests**: TDD —— 每个故事先提交失败测试，再实现。

## Phase 1: Setup

- [x] T301 创建分支 `003-game-api` 与 spec/plan/contracts/tasks 文档

## Phase 2: Foundational

- [x] T302 测试 + 实现：测试机 `render_conflicts` 计数（vblank 窗口）`crates/nes-sim`
- [x] T303 测试 + 实现：parser 上下文声明 `map` / `metasprite` 与元组 `crates/nesl/src/{ast,parser}.rs`

## Phase 3: US1 VRAM 队列 (P1) 🎯 MVP

- [x] T304 [US1] 测试：渲染中 print/set_tile/load_palette 无违规写入、溢出等待、ppu_off 刷新；001 示例在测试机中零违规 `crates/nesl/tests/game_api.rs`
- [x] T305 [US1] 实现：内存布局调整、`__vq_push/__vq_flush`、NMI 刷新、print 分块、set_tile `crates/nesl/src/{runtime,builtins}.rs`

## Phase 4: US2 map (P2)

- [x] T306 [US2] 测试：draw_map 命名表/属性表、渲染中调用、map_tile 边界、编译错误
- [x] T307 [US2] 实现：map 数据、`draw_map`、`map_tile`

## Phase 5: US3 metasprite (P2)

- [x] T308 [US3] 测试：OAM 布局、负偏移、越界隐藏、返回槽位、默认属性
- [x] T309 [US3] 实现：metasprite 数据与 `__metasprite`

## Phase 6: US4 collide (P2)

- [x] T310 [US4] 测试：重叠/相邻/包含/255 边界、参数只求值一次
- [x] T311 [US4] 实现：collide 降级

## Phase 7: US5 sfx (P3)

- [ ] T312 [US5] 测试：音量衰减、周期滑动、结束静音、噪声通道、与 play_tone 共存
- [ ] T313 [US5] 实现：音效表、NMI tick、`APU_EN` 影子

## Phase 8: Polish

- [ ] T314 [P] 示例 `examples/maze.nesl` + 端到端测试；编辑器示例与高亮
- [ ] T315 [P] README / 语言参考更新；clippy 清零
- [ ] T316 合并到 `main`

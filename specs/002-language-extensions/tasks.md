---
description: "Task list for NESL language extensions"
---

# Tasks: NESL 语言增强

**Input**: `/specs/002-language-extensions/`（spec.md、plan.md、contracts/language.md）

**Tests**: TDD —— 每个故事先写失败测试并提交，再实现。

## Phase 1: Setup

- [x] T201 创建分支 `002-language-extensions` 与 spec/plan/contracts/tasks 文档

## Phase 2: Foundational

- [x] T202 测试：lexer `.`/`struct`/`match`；parser 解析 struct 声明、字段访问降级、数组元素类型、无初始化 let、match `crates/nesl/src/tests/parser.rs`
- [x] T203 实现 AST/lexer/parser 扩展 `crates/nesl/src/{ast,lexer,parser}.rs`

## Phase 3: US1 struct (P1) 🎯 MVP

- [ ] T204 [US1] 测试：结构体变量、结构体数组、复合赋值、错误信息 `crates/nesl/tests/lang2.rs`
- [ ] T205 [US1] 实现结构体注册（SoA 降级）`crates/nesl/src/codegen.rs`

## Phase 4: US2 i8 (P2)

- [ ] T206 [US2] 测试：混合加减、有符号比较、符号扩展、算术右移、错误 `crates/nesl/tests/lang2.rs`
- [ ] T207 [US2] 实现 I8 类型规则与代码生成 `crates/nesl/src/codegen.rs`

## Phase 5: US3 u16 数组与局部数组 (P2)

- [ ] T208 [US3] 测试：u16/i8 RAM 数组与常量表、局部数组清零 `crates/nesl/tests/lang2.rs`
- [ ] T209 [US3] 实现数组元素类型与局部数组 `crates/nesl/src/codegen.rs`

## Phase 6: US4 match (P3)

- [ ] T210 [US4] 测试：值/或/区间/通配/只求值一次/i8 负数模式 `crates/nesl/tests/lang2.rs`
- [ ] T211 [US4] 实现 match 降级 `crates/nesl/src/codegen.rs`

## Phase 7: Polish

- [ ] T212 [P] 示例 `examples/shooter.nesl`（struct + i8 + match）+ 端到端测试；编辑器示例列表与高亮
- [ ] T213 [P] README / 语言参考更新；clippy 清零
- [ ] T214 合并到 `main`

## Dependencies

T203 阻塞所有故事；US1-US4 依次在同一文件实现，按顺序进行。

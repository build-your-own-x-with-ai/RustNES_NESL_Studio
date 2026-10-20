---
description: "Task list for more enemy behaviors and sound effects"
---

# Tasks: 更多敌人行为与音效

- [ ] T1101 分支 `011-foes`，草稿 `/tmp/q11.nesl` 探测（已确认：struct 字段增加、i8 数组、match、sound 混用全部编译通过，跳跳怪会跳、蝙蝠飞行波动、主循环 160 帧 = 160 次）
- [ ] T1102 red: `quest.rs::three_foe_kinds_move_as_expected` 测试跳跳怪跳跃、蝙蝠飞行轨迹
- [ ] T1103 red: `quest.rs::hop_and_stomp_sound_effects` 测试新音效 APU 写入（`$4006` 滑音）
- [ ] T1104 实现：`examples/quest.nesl` 采纳 `/tmp/q11.nesl` 的敌人行为、音效、图块
- [ ] T1105 green: 补全测试（APU 写入断言、600 帧不掉帧）
- [ ] T1106 jsnes 交叉验证（标题、第一关静态画面）仍然通过
- [ ] T1107 clippy、全部测试、文档（语言参考不变、README 示例不变）、合并

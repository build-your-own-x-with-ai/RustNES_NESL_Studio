---
feature: 012-save-system
description: "存档系统：断电保存进度（世界、分数、生命），开机时恢复"
status: draft
---

# Spec: 存档系统

## 背景

当前 COIN QUEST 每次开机都从头开始（世界 0，分数 0，生命 3）。玩家通关一个世界后关机，下次开机会丢失进度。NES 卡带可以通过电池供电的 SRAM 保存数据（battery-backed PRG-RAM at `$6000-$7FFF`，8KB），断电后数据仍然保留。

典型的 NES 游戏存档包括：当前关卡、分数、生命数、道具状态。COIN QUEST 目前只需要保存世界编号、分数、生命数。

## 目标用户

玩 COIN QUEST 并希望断电后保留进度的用户。

## 需求

### US1: 开机时恢复上次的世界、分数、生命
**作为** COIN QUEST 玩家  
**我希望** 开机时自动恢复上次的进度  
**以便** 不必每次从头开始

**验收**：
- 如果 SRAM 里有有效存档（校验和正确），`new_game()` 从 SRAM 读取 `world`、`score`、`lives`
- 如果 SRAM 为空或校验和错误（首次开机、电池耗尽、数据损坏），从默认值开始（世界 0，分数 0，生命 3）
- 开机后按 START，游戏从恢复的世界开始

### US2: 通关一个世界后自动保存
**作为** COIN QUEST 玩家  
**我希望** 每次通关一个世界时进度自动保存  
**以便** 断电后不丢失已通关的世界

**验收**：
- 世界通关后（吃到旗子，state 变为 2），调用 `save()` 将当前 `world`、`score`、`lives` 写入 SRAM
- 下次开机时恢复这些值

### US3: 标题画面显示「继续」或「新游戏」
**作为** 玩家  
**我希望** 在标题画面看到当前是否有存档  
**以便** 知道按 START 后会发生什么

**验收**：
- 有存档时，标题画面显示 `"CONTINUE (WORLD N)"` 和分数
- 无存档时，显示 `"PRESS START"`
- 按 START 后，有存档进入对应世界，无存档从世界 0 开始

## 技术方案

### SRAM 布局（$6000-$7FFF, 8KB）
只使用前 16 字节：
```
$6000: 'N' 'E' 'S' 'L'  // 魔数（magic，用于识别有效存档）
$6004: world (u8)
$6005: lives (u8)
$6006: score low (u8)
$6007: score high (u8)
$6008: checksum (u8)  // world + lives + score_lo + score_hi 的和的低字节
$6009-$600F: 保留（未来扩展：道具、已收集的金币等）
```

### 校验和算法
`checksum = (world + lives + score_lo + score_hi) & 0xFF`

读取时，如果 `magic != "NESL"` 或 `checksum` 不匹配，认为存档无效。

### 编译器改动
1. `build_unrom()` 和 `build_ines()`（NROM）的 flags6 增加 bit1（battery-backed PRG-RAM）：
   - NROM: `0x01 | 0x02 = 0x03`（vertical mirroring + battery）
   - UNROM: `0x21 | 0x02 = 0x23`（mapper 2 low + vertical + battery）
2. 新增运行时函数：
   - `save_game()`: 将 `world`、`lives`、`score` 写入 `$6000`，计算并写入校验和
   - `load_game() -> u8`: 检查 magic 和校验和，有效返回 1 并恢复变量，无效返回 0
3. `new_game()` 开头调用 `load_game()`，如果返回 1 则跳过初始化

### NESL 改动
examples/quest.nesl：
- 标题画面（state 0）增加存档检测：调用内建函数 `has_save() -> u8`，返回 1 表示有有效存档
- 有存档时显示 `"CONTINUE (WORLD N)"`，无存档显示 `"PRESS START"`
- 世界通关时（state 2 进入前）调用内建函数 `save_game()`

### 内建函数
```nesl
has_save() -> u8       // 检查 $6000 magic 和 checksum，有效返回 1
save_game()            // 将 world/lives/score 写入 SRAM，写 magic 和 checksum
```

`load_game()` 不暴露给 NESL（运行时自动调用，在 `__init_globals` 之后、`main` 之前）。

## 测试策略

### 单元测试（crates/nesl/tests/save.rs）
1. `save_and_load_restores_progress()`: 编译带 `save_game()` 的程序，poke 变量，调用 save，重启 NES（from_ines 读同一个 ROM，bus.ram `$6000` 保留），调用 load，检查变量恢复
2. `invalid_magic_is_ignored()`: SRAM `$6000` 写 `'X' 'X' 'X' 'X'`，load 返回 0
3. `invalid_checksum_is_ignored()`: 写有效 magic 但校验和错误，load 返回 0
4. `nrom_and_unrom_both_have_battery_bit()`: 检查 NROM ROM[6] & 0x02 != 0，UNROM ROM[6] & 0x02 != 0

### 端到端测试（crates/nesl/tests/quest.rs）
`save_restores_world_and_score()`: 开始游戏 → 通关世界 0 → 检查 `$6000` magic/checksum → 重启 NES（保留 SRAM）→ 按 START → 检查 `world == 1`

## 限制
- 每个 NESL 程序只能有一个存档（不支持多个存档槽）
- 存档在模拟器里有效，烧录到实体卡带需要焊接电池
- `$6000-$7FFF` 目前只用 16 字节，剩余空间留给未来扩展（道具、已收集金币位图等）

## 交付
- [ ] 测试用例（红）
- [ ] 运行时 `save_game` / `load_game` / `has_save`
- [ ] ROM flags6 增加 battery bit
- [ ] nes-sim 支持断电保留 SRAM（TestNes 新增 `preserve_sram()` 方法）
- [ ] quest.nesl 集成存档（标题画面 + 通关后保存）
- [ ] 文档（language.md、README）

## 未来扩展
- 多存档槽（A/B/C 三个存档，每个 16 字节）
- 保存已收集的金币（位图，每个世界 128 列 × 32 行 = 4096 位 = 512 字节）
- 保存道具状态（双跳、冲刺已解锁）

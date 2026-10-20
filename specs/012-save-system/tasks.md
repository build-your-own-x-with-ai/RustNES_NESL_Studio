---
description: "任务列表：存档系统"
---

# Tasks: 存档系统

## T1201: nes-sim 支持 SRAM 保留
**目标**：`TestNes` 增加 `sram_snapshot()` 和 `from_ines_with_sram()` 方法，模拟断电重启但 SRAM 保留

**步骤**：
1. `NesBus` 增加 `pub fn sram(&self) -> Vec<u8>`，返回 `self.ram[0x6000..0x8000].to_vec()`
2. `TestNes::from_ines_with_sram(rom: &[u8], sram: &[u8])` → 先 `from_ines(rom)`，再 `bus.ram[0x6000..0x6000+sram.len()].copy_from_slice(sram)`
3. 测试：写 `$6123 = 0x42`，snapshot SRAM，重启，读 `$6123` 应为 `0x42`

## T1202: 运行时 save_game / load_game / has_save
**目标**：三个汇编例程，操作 `$6000` 的存档结构

**步骤**：
1. `runtime.rs` 增加 `SAVE_MAGIC = 0x6000`（4 字节 "NESL"）、`SAVE_WORLD = 0x6004`、`SAVE_LIVES = 0x6005`、`SAVE_SCORE_LO = 0x6006`、`SAVE_SCORE_HI = 0x6007`、`SAVE_CHECKSUM = 0x6008`
2. `emit_save_game(e: &mut Emitter)`：
   ```asm
   __save_game:
       LDA #'N'; STA $6000
       LDA #'E'; STA $6001
       LDA #'S'; STA $6002
       LDA #'L'; STA $6003
       LDA world; STA $6004
       LDA lives; STA $6005
       LDA score; STA $6006
       LDA score+1; STA $6007
       ; checksum = world + lives + score_lo + score_hi
       CLC
       LDA world; ADC lives; ADC score; ADC score+1
       STA $6008
       RTS
   ```
3. `emit_load_game(e: &mut Emitter)`（返回 A = 1 成功，0 失败）：
   ```asm
   __load_game:
       ; check magic
       LDA $6000; CMP #'N'; BNE .fail
       LDA $6001; CMP #'E'; BNE .fail
       LDA $6002; CMP #'S'; BNE .fail
       LDA $6003; CMP #'L'; BNE .fail
       ; verify checksum
       CLC
       LDA $6004; ADC $6005; ADC $6006; ADC $6007  ; world+lives+score_lo+score_hi
       CMP $6008; BNE .fail
       ; restore
       LDA $6004; STA world
       LDA $6005; STA lives
       LDA $6006; STA score
       LDA $6007; STA score+1
       LDA #1; RTS
   .fail:
       LDA #0; RTS
   ```
4. `emit_has_save(e: &mut Emitter)`（与 `load_game` 相同的 magic/checksum 检查，但不恢复变量）：
   ```asm
   __has_save:
       LDA $6000; CMP #'N'; BNE .fail
       LDA $6001; CMP #'E'; BNE .fail
       LDA $6002; CMP #'S'; BNE .fail
       LDA $6003; CMP #'L'; BNE .fail
       CLC
       LDA $6004; ADC $6005; ADC $6006; ADC $6007
       CMP $6008; BNE .fail
       LDA #1; RTS
   .fail:
       LDA #0; RTS
   ```
5. `emit_core()` 在 `__init_globals` 之后、跳转 `main` 之前，调用 `__load_game`（结果忽略）

**测试**：
- `crates/nesl/tests/save.rs::save_and_load_restores_variables()`：编译 `var world: u8; var lives: u8; var score: u16; fn main() { save_game(); }`，先 poke `world=5, lives=2, score=1234`，`call(__save_game)`，snapshot SRAM，重启（`from_ines_with_sram`），检查 `world/lives/score` 恢复
- `invalid_magic_returns_zero()`：`$6000` 写 `'X' 'X' 'X' 'X'`，`call(__load_game)` 返回 `A=0`
- `invalid_checksum_returns_zero()`：magic 正确但 `$6008` 改成错误值，返回 0

## T1203: builtins.rs 暴露 save_game / has_save
**目标**：NESL 程序可以调用 `save_game()` 和 `has_save() -> u8`

**步骤**：
1. `builtins.rs` 增加：
   ```rust
   "save_game" => {
       self.check_args(&args, 0)?;
       self.e.jsr("__save_game");
       Type::Unit
   }
   "has_save" => {
       self.check_args(&args, 0)?;
       self.e.jsr("__has_save");  // A = 1 or 0
       Type::U8
   }
   ```
2. 文档：language.md v8 新增 `save_game()` / `has_save() -> u8`

## T1204: ROM flags6 增加 battery bit
**目标**：NROM 和 UNROM 的 iNES 文件头 flags6 bit1 置 1

**步骤**：
1. `rom.rs::build_ines()`: `0x01 | 0x02 = 0x03`（vertical + battery）
2. `rom.rs::build_unrom()`: `0x21 | 0x02 = 0x23`（mapper2 low + vertical + battery）
3. 测试：
   ```rust
   #[test]
   fn nrom_has_battery_bit() {
       let out = nesl::compile("fn main() {}").unwrap();
       assert_eq!(out.rom[6] & 0x02, 0x02);
   }
   #[test]
   fn unrom_has_battery_bit() {
       let out = nesl::compile(&format!("{} fn main() {{}}", big_maps())).unwrap();
       assert_eq!(out.mapper, 2);
       assert_eq!(out.rom[6] & 0x02, 0x02);
   }
   ```

## T1205: quest.nesl 集成存档
**目标**：标题画面显示「继续」或「新游戏」，通关后自动保存

**步骤**：
1. `new_game()` 开头不再写死 `world=0; lives=3; score=0`，改为：
   ```nesl
   if !has_save() {
       world = 0; lives = 3; score = 0;
   }
   // else: 变量已在运行时 __load_game 中恢复
   ```
2. 标题画面（state 0）：
   ```nesl
   if has_save() {
       print(8, 12, "CONTINUE");
       print(7, 14, "WORLD ");
       print_u8(13, 14, world + 1);  // 显示人类友好的编号（1-6）
   } else {
       print(7, 14, "PRESS START");
   }
   ```
3. 世界通关（flag 碰撞后 `state = 2` 之前）：
   ```nesl
   save_game();
   state = 2;
   ```

## T1206: 端到端测试
**目标**：`crates/nesl/tests/quest.rs::save_restores_world_and_score()`

**步骤**：
1. 编译 quest.nesl
2. 开始游戏（按 START）
3. poke `hero.x = 980`（旗子位置），等待通关
4. 检查 `$6000` == `'N' 'E' 'S' 'L'`，`$6004` == 1（world+1）
5. snapshot SRAM
6. 重启（`from_ines_with_sram`）
7. 不按 START，检查 `has_save()` 返回 1，画面显示 "CONTINUE"
8. 按 START，等待进入 play 状态
9. 检查 `world == 1`

## T1207: clippy + 文档 + 合并
- `cargo clippy --workspace --all-targets` 零警告
- README.md 增加「存档」行
- language.md v8 新增 `save_game()` / `has_save()`
- 合并 012-save-system 到 main

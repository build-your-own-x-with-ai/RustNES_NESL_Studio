# 010-banking 总结

## 实现内容
- **自动 UNROM (mapper 2) 切换**：PRG > 32KB 时编译器自动改用 UNROM，不超过仍用 NROM。
- **地图分 bank**：大地图数据按 16KB bank 切分，动态切换后访问地图指针。
- **CHR-RAM**：UNROM 芯片规格固定用 CHR-RAM，启动时从 bank 0 拷贝瓦片数据到 VRAM。
- **总线冲突**：PRG-ROM 写入必须从值等于 bank id 的地址写，避免数据冲突损坏卡带。
- **镜像修正**：UNROM 必须使用垂直镜像（flags6 = 0x21），横向镜像会让卷轴绘制覆盖可见屏幕。
- **COIN QUEST 扩展**：6 个世界（W1-W6），每个世界独立地图、不同敌人分布、不同调色板。

## 技术细节
- **两次编译**：先按 NROM 汇编一次得知大小，> 32762 字节时重新按 UNROM 汇编（固定 bank 从 `$C000` 起）。
- **Equ 汇编指令**：动态地图参数（`WORLDS[i]`）生成 `Equ` 符号（不占字节，仅定义标签地址），供 `map_select` 读取。
- **bank 分配**：首次适配算法，bank 0 前 8KB 留给 CHR 数据和运行时，地图数据从 `$A000` 起；剩余地图按序放入后续 bank；不够一个 bank 时填满；超过 7 个 switchable bank 时固定 bank 放代码。
- **运行时**：`__bank` (TAX; STA __bank_table,X; RTS) + `__bank_table` (0..15)；`map_select` 设置 `SEL_BANK`；`draw_map` / `camera_follow` / `map_tile` 先切 bank 再读指针。
- **测试策略**：nes-sim 添加 mapper 2 支持、总线冲突计数、SRAM；单元测试检查 ROM 文件头、bank 切换、跨 bank 地图访问；端到端测试检查 COIN QUEST 关卡（跳跃高度、坑、碰撞、通关检测）；jsnes 交叉验证（标题画面 + 卷轴关卡）。

## Bug 与修正
1. **总线冲突处理错误**：初次实现用 `val % self.prg_banks` 导致测试 ROM（每个 bank 重复填充 `0x11 * bank_id`）崩溃；测试 ROM 不是总线冲突安全的（`0x11 & 0x00 = 0x00`）。修正：(a) 测试 ROM 改为 bank table（ROM[i] = i）；(b) `prg_bank = ((val & rom_byte) as usize) % prg_banks`。
2. **Equ 未插入符号表**：补丁脚本失败时没有明显错误；排查发现 layout pass 没有 `symbols.insert` for Equ。修正：添加 insert、删除重复 unreachable 分支、补全 listing。
3. **Bank 地址错误**：初次 `emit_banked_maps` 让 bank 0 从 `$C000` 起、后续 bank 从 `$8000` 起，导致指针错乱。修正：所有 bank base_addr = 0x8000（前 8KB 是 CHR + 运行时，地图数据从 `+chr.len()` 起）；固定 bank 仍在最后（`$C000-$FFFF`）。
4. **__chr_copy 寄存器错误**：LDX #pages; LDY #0; 内循环用 Y 索引 `(PTR),Y` 需要 `PTR` 为 u8（$00-$FF）。修正：PTR = 0; 外循环 INC PTR+1。
5. **PPUCTRL 错误**：CHR 拷贝前设置 PPUCTRL = 0x20 (8x16 sprites) 导致地址自增异常。修正：CHR 拷贝时 PPUCTRL = 0。
6. **空 bank 位置错误**：填充空 bank 放在固定 bank 后面，导致 reset vector 丢失。修正：空 bank 插在固定 bank 之前。
7. **NROM 阈值包含 CHR**：初版判断 `asm.code.len() + chr.len() > 32 * 1024` 错误（CHR 是独立芯片）。修正：只算 PRG（`asm.code.len() > 32762`）。
8. **SEL_BANK 与 VQ_LOCK 冲突**：SEL_BANK = 0x0363 与 VRAM 队列锁地址冲突，导致 `print` / `set_tile` 卡死。修正：SEL_BANK 改为 0x037E；添加 `layout_tests::runtime_variables_do_not_overlap`。
9. **JUMP 太弱**：初版 `JUMP = -7` 无法跳上平台（平台在 row 17，坑 ≤4 列）；gen6.py 生成的地图不符合规则（平台 row 14）。修正：JUMP = -9；平台 rows = [17, 13]；确保坑宽 ≤ 4。
10. **bank 切换时机错误**：初版 `draw_map` / `camera_follow` 顺序为 `map_ptrs → map_select → map_bank → map_byte`，导致 `LDA (PTR),Y` 读到错误 bank 的数据。修正：调整为 `map_select → map_bank → map_ptrs → map_byte`，确保指针读取前 bank 已切换。
11. **UNROM 镜像标志错误**：`build_unrom` flags6 = 0x20（水平镜像）导致 FCEUX 中卷轴绘制覆盖可见屏幕，表现为地面缺失、平台错位、金币消失后画面未更新。修正：flags6 = 0x21（垂直镜像）；nes-sim 改为按 iNES 头读取镜像方式；添加 `scrolled_unrom_level_matches_jsnes` jsnes 交叉验证。

## 提交历史
```
e81c6a7 test(010): mirroring and complete-world checks (red)
84ea1f7 feat(010): vertical mirroring for UNROM; honor iNES mirroring in nes-sim
ecca673 feat(010): complete world detection test
```

## 文件修改
- `crates/nes-sim/src/nes.rs`：添加 `mapper`, `prg_bank`, `prg_banks`, `bus_conflicts`, `sram: Vec<u8>`；按 iNES flags6 bit0 设置 mirroring (0=horizontal, 1=vertical)；mapper 2 读写逻辑。
- `crates/nes-sim/tests/mapper.rs`：UNROM 单元测试（bank 切换、总线冲突）。
- `crates/nesl/src/asm.rs`：`Item::Equ(String, u16)`。
- `crates/nesl/src/codegen.rs`：`Sym::Map { bank }`、`map_table: Vec<(String,u8,u8)>`、`banked: bool`、`declared_maps`、`assign_banks()`、`emit_map_select()` 读 `SEL_BANK`、`emit_banked_maps()`。
- `crates/nesl/src/runtime.rs`：`SEL_BANK = 0x037E`、`emit_banked()`（`__bank` + `__bank_table`）、`__chr_copy`（bank 0 → VRAM）、`layout_tests::runtime_variables_do_not_overlap`。
- `crates/nesl/src/rom.rs`：`build_unrom()` flags6 = 0x21（mapper 2 + 垂直镜像）；空 bank 填充在固定 bank 之前；PRG banks 向上取到 2 的幂。
- `crates/nesl/src/lib.rs`：`CompileOutput.mapper`；`compile()` PRG > 32762 时调用 `compile_unrom()`。
- `crates/nesl/src/builtins.rs`：`MapArg::Static { bank }`；`map_bank()` JSR `__bank` 切换；`draw_map` / `camera_follow` / `map_tile` 顺序改为 `map_select → map_bank → map_ptrs → map_byte`；删除未使用的 `map_setup`。
- `crates/nesl/tests/{banking.rs,quest.rs,asm.rs}`：新增单元测试和端到端测试。
- `crates/nesl/tests/xcheck.rs`：新增 `scrolled_unrom_level_matches_jsnes`。
- `examples/quest.nesl`：6 个世界（W1-W6），独立地图（来自 /tmp/quest/gen6.py）、敌人分布、调色板，JUMP = -9。
- `specs/010-banking/{spec.md,tasks.md,IMPLEMENTATION.md,SUMMARY.md}`。

## 后续工作
- **存档系统（feature 012）**：`save_game()` / `load_game()` 自动保存/读取全局变量到 SRAM $6000；`has_save()` 检查存档有效性；flags6 增加 battery bit (0x02)；COIN QUEST 通关时存档、标题画面显示 CONTINUE。
- **nes-sim 完善**：目前 bus conflicts 和 sram 计数为 0 可能是因为测试里没有写入；实体 NES 测试（烧录卡）。
- **性能优化**：超大地图（> 15 个 bank）时考虑压缩或地图生成算法。

# 013-mmc3 实现任务

## 1. nes-sim 支持 MMC3

- [x] `nes.rs` 添加 mapper 4 寄存器：bank_select, bank_data[8], mirroring, irq_latch, irq_counter, irq_enabled, irq_reload
- [x] PRG 读取：$8000/$A000 可切换（bank_data[6]/[7]），$C000 倒数第二，$E000 最后
- [x] CHR 读取：2KB banks (bank_data[0..2]) 和 1KB banks (bank_data[2..6])
- [x] 写入 $8000/$A000/$C000/$E000：bank_select, bank_data, mirroring, irq 控制
- [x] IRQ 计数器：每条扫描线递减，0→reload 并触发 IRQ（A12 上升沿检测简化为固定时机）
- [x] `from_ines` 识别 mapper 4
- [x] 测试：bank 切换、IRQ 触发时机

## 2. ROM 构建

- [x] `rom.rs` 添加 `build_mmc3(prg_banks, chr_banks, ...)` → iNES header + PRG + CHR
- [x] header: mapper 4 (flags6 低 4 位 = 4, flags7 高 4 位 = 0), 支持 battery/mirroring
- [x] CHR-ROM 支持：打包图块数据到 CHR banks

## 3. 编译器 mapper 选择

- [ ] `lib.rs` 三阶段检查：
  - PRG ≤ 32KB → NROM
  - PRG ≤ 256KB 且无 irq() → UNROM
  - 否则 → MMC3
- [x] 识别 `fn irq() { ... }` 声明（IRQ 向量指向 `__irq_handler`）
- [ ] CHR-ROM vs CHR-RAM：声明了 `tile` 则用 CHR-ROM，否则 CHR-RAM

## 4. IRQ 内建函数

- [x] `builtins.rs` 添加 `scanline_irq(n: u8)`：写 MMC3 寄存器 $C000/$C001/$E000/$E001
- [x] 运行时 `__irq_handler`：保存 A/X/Y/P，调用用户 `irq()`，恢复，RTI
- [x] `irq()` 签名检查：无参数，无返回值

## 5. 测试

- [x] `mapper.rs` 测试 MMC3 bank 切换和 IRQ
- [ ] `banking.rs` 测试超大程序自动使用 MMC3
- [x] 新建 `irq.rs`：状态栏分割屏测试（IRQ 在第 N 行改变滚动）
- [ ] xcheck：分割屏画面与 jsnes 对比

## 6. 文档

- [x] `IMPLEMENTATION.md`：MMC3 寄存器实现、IRQ 时序、bank 布局
- [ ] `SUMMARY.md`：功能总结、示例、与 UNROM 对比
- [ ] 更新 `README.md` 和 `specs/001-.../contracts/language.md`（添加 `scanline_irq` 和 `fn irq()`）

## v1 实际范围（与上面计划的差异）

- MMC3 只在声明 `fn irq()` 时启用；PRG 限 32KB（线性映射，不做 bank 切换），超过会报错
- 未做：PRG > 256KB 自动选 MMC3、MMC3 + 地图分 bank、COIN QUEST 迁移到 MMC3（它是 34KB 的 UNROM）
- 未做：xcheck 分割屏与 jsnes 对比（改为测试机逐行画面对比 + examples/split.nesl）
- `scroll()` 之外新增 `split_scroll(x)`，只能在 `fn irq()` 里用

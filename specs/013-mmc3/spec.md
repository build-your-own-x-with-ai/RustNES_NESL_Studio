# 013-mmc3：MMC3 mapper 支持

## 背景

当前支持 NROM (mapper 0, 32KB PRG) 和 UNROM (mapper 2, 最多 256KB PRG, CHR-RAM)。
MMC3 (mapper 4) 是 NES 最流行的 mapper，提供：
- 最多 512KB PRG-ROM (32 个 16KB 或 64 个 8KB bank)
- 最多 256KB CHR-ROM (256 个 1KB bank)
- IRQ 计数器：精确的扫描线中断，用于状态栏、视差滚动、分割屏
- PRG-RAM (6000-7FFF, 8KB, battery-backed 可选)

## 目标

实现 MMC3 mapper，使 NESL 程序可以：
- 自动使用 MMC3 当 PRG 超过 UNROM 限制 (256KB) 或需要 CHR-ROM
- 使用 `scanline_irq(行号)` 在指定扫描线触发 IRQ 中断
- 定义 IRQ 处理函数 `fn irq() { ... }`
- 支持 CHR-ROM bank 切换（背景和精灵分别可用 2KB 和 1KB bank）

## 用例

```nesl
// 状态栏：顶部 3 行固定，下方滚动
var status_split: u8 = 24;  // 第 24 条扫描线触发 IRQ

fn irq() {
    scroll(0, 0);  // IRQ 时重置滚动为状态栏
}

fn main() {
    status_bar(3);
    scanline_irq(status_split);
    ppu_on();
    loop {
        wait_vblank();
        scroll(camera_x() as u8, 0);  // vblank 设置游戏区滚动
        // MMC3 IRQ 会在第 24 行自动恢复状态栏滚动
    }
}
```

```nesl
// 大型游戏：自动切换到 MMC3
// 10 个大地图 + 大量代码 → 编译器检测到超过 UNROM 256KB，输出 MMC3 ROM
```

## 非目标（v1）

- 手动 PRG/CHR bank 切换（编译器自动管理）
- 多种 bank 切换模式（固定用 $8000/$A000 可切换，$C000 固定倒数第二 bank，$E000 固定最后 bank）
- PRG-RAM 超过 8KB
- Scanline IRQ 的手动启用/禁用（自动管理：设置了 irq() 则启用，否则禁用）

## 验收标准

1. nes-sim 支持 mapper 4：bank 切换寄存器、IRQ 计数器
2. 编译器自动选择 mapper：NROM → UNROM → MMC3（PRG > 256KB 或声明了 irq()）
3. 程序可以定义 `fn irq() { ... }`，在 IRQ 向量指向运行时
4. `scanline_irq(n)` 设置 MMC3 IRQ 扫描线计数器
5. 测试：状态栏分割屏（顶部固定，下方滚动），与 FCEUX 对比
6. 测试：超大程序（>256KB PRG）自动使用 MMC3
7. COIN QUEST 可选用 MMC3 状态栏（替代当前的 sprite-0 方案）

## 参考

- [MMC3 - Nesdev Wiki](https://www.nesdev.org/wiki/MMC3)
- [IRQ timing](https://www.nesdev.org/wiki/MMC3#IRQ_Specifics)
- iNES header: mapper 4, flags6 = 0x41 (vertical mirroring, battery), flags7 = 0x40

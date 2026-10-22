# 013-mmc3 实现说明

## 用法

```nesl
var cam: u16;
fn irq() { split_scroll(cam); }      // 每帧第 24 行调用
fn main() {
    scroll(0, 0);                    // 顶部（状态栏）不卷动
    scanline_irq(24);                // 0 = 关闭
    ppu_on();
    loop { wait_vblank(); cam += 1; }
}
```

声明 `fn irq()` 的程序编译成 MMC3（mapper 4）；没有 `fn irq()` 的程序照旧是 NROM / UNROM。

## ROM 布局

- iNES 头：`2, 1, 0x43, 0x00`（32KB PRG、8KB CHR、mapper 4、电池、垂直镜像）
- PRG 线性放在 `$8000-$FFFF`；`$FFD0` 起是复位桩：依次写 R7..R0（PRG 0/1，CHR 0,2,4,5,6,7）、垂直镜像、打开 PRG-RAM，再跳到 `__reset`。上电时 `$8000-$DFFF` 的映射不确定，只有 `$E000` 固定，所以复位向量指向桩
- CHR：图块（`$0000-$0FFF`）复制到两个图案表；PPUCTRL = `$88`，精灵从 `$1000` 取。这样每条扫描线 PPU A12 上升一次，MMC3 计数器才会走（背景和精灵都在 `$0000` 时 A12 永远不升，真机上 IRQ 不会来）

## 运行时

- `__reset`：先写 `$E000` 关 IRQ；`__init_globals` 后 `CLI`
- NMI（`__nmi_skip` 处，VRAM 写完之后）：`$E000` 关闭并应答，`IRQ_LINE` 非 0 时写 latch = line - 2、`$C001` 重载、`$E001` 打开。预渲染行重载计数器，IRQ 在 (latch + 1) 行后到达；减 2 是给 IRQ 延迟和保存现场留时间，使 `split_scroll` 从第 line 行生效
- `__irq`：保存 A/X/Y 和 `$00-$07`、`$0E-$14`（内建函数用的零页临时区），写 `$E000` 应答，`JSR f_irq`，恢复后 RTI

## 编译规则

- `irq` 不能有参数和返回值，不能被调用，也不能调用用户函数（局部变量是静态分配的，中断里调用会破坏被打断代码的局部变量）；内建函数可以用
- `scanline_irq` 需要 `fn irq()`；`split_scroll` 只能在 `fn irq()` 里

## 测试机（nes-sim）

- mapper 4：PRG 8KB 窗口（bit 6 交换 `$8000/$C000`）、CHR 2KB/1KB 窗口（bit 7 交换图案表）、`$A000` 镜像
- IRQ 计数器：每帧 241 次时钟（预渲染行 + 240 可见行，dot 260），只在渲染打开时；按 CPU 周期区间闭式计算
- `Cpu::irq`：I 标志置位时忽略；IRQ 线为电平，直到 `$E000` 应答
- 渲染器、精灵 0 检测、PPUDATA 读取都经过 mapper 的 CHR 映射

简化：真机按 A12 上升沿计数，测试机按固定时机计数；`$2006` 写入引起的 A12 翻转未模拟。

## 测试

- `nes-sim/tests/mapper.rs`：默认 bank、PRG/CHR 切换、首次 IRQ 时机、间隔 10 行、每帧 241 次、渲染关闭时不计数
- `nesl/tests/irq.rs`：MMC3 头与 CHR 复制、每帧一次且在第 100 行附近、`scanline_irq(0)` 关闭、IRQ 不破坏主程序运算、分割线上下画面分别与不卷动 / 卷动 40 的画面逐行一致、错误信息、examples/split.nesl 顶部不动下方滚动

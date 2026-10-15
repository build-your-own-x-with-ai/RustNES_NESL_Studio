# Research: NESL 编译器技术决策

## R1. 源语言形态
- **Decision**: 自定义类 Rust 语法的小语言 NESL（`fn`、`let`、`var`、`u8/u16`、Rust 运算符优先级）。
- **Rationale**: 编译到 6502 必须严格控制类型宽度；复用 C/Rust 现成前端（如 cc65、llvm-mos）会引入外部工具链，违背零依赖和「一条命令出 ROM」。
- **Alternatives**: 编译 C 子集（语义复杂：指针/整型提升）；BASIC 方言（表达力不足，类型隐式）。

## R2. 局部变量分配
- **Decision**: 静态分配，每个函数的参数和局部变量拥有唯一地址；拒绝递归。
- **Rationale**: 6502 只有 256 字节硬件栈且无栈相对寻址，软件栈代码慢 3-5 倍；NES 游戏几乎不需要递归。
- **Alternatives**: cc65 式软件栈（慢、复杂）；调用图着色复用槽位（后续优化项）。

## R3. 表达式求值
- **Decision**: 累加器式：u8 结果在 A，u16 在 A(低)/X(高)。右操作数是常量/变量时直接寻址，否则压栈。
- **Rationale**: 实现简单、正确性容易用模拟器验证，生成代码对常见 `x + 1`、`a < b` 已经很紧凑。

## R4. 分支距离
- **Decision**: 汇编器迭代松弛：先全部短分支，超出 ±127 的改为 `B!cc +3; JMP target`，直到稳定。
- **Rationale**: 代码生成器无需关心距离；大循环体也能正确编译。

## R5. 测试策略
- **Decision**: 独立实现 6502 模拟器 + 简化 PPU 寄存器模型（VRAM/OAM/手柄/帧中断），用 nestest 日志校验 CPU。
- **Rationale**: 断言汇编文本很脆弱；执行语义才是用户关心的。编译器与模拟器分开实现，相互交叉验证。

## R6. ROM 格式
- **Decision**: iNES 1.0，Mapper 0，PRG 2×16KB，CHR 1×8KB，垂直镜像（便于水平卷轴）。
- **Rationale**: 所有模拟器和烧录卡都支持；无需 bank 切换。

## R7. 字体与图块
- **Decision**: 内置 8×8 ASCII 字体（`0x20-0x5F`，颜色 1）放在图块索引 = ASCII 码处；背景和精灵共用图案表 0；
  `tile N = [...]` 可覆盖任意索引。图案表 1 复制同样内容。
- **Rationale**: `print("HELLO")` 直接写 ASCII 码到命名表，零配置可用。

## R8. Web 编辑器
- **Decision**: `nesc serve` 用 `std::net::TcpListener` 实现最小 HTTP/1.1；页面 `include_str!` 内嵌；浏览器端用 jsnes（CDN）运行。
- **Rationale**: 无需 Node/打包工具；单个二进制即可提供完整编辑器。
- **Alternatives**: TUI 编辑器（无法直接显示游戏画面）；Electron/Tauri（依赖重）。

## R9. VRAM 写入与滚动
- **Decision**: `vram_addr` 写完 `$2006` 后立即用影子寄存器重写 `$2000/$2005×2`，只修改 PPU 的 t 寄存器，不影响后续 `$2007` 写入的地址 v。
- **Rationale**: 避免用户在 vblank 中写 VRAM 后画面滚动错乱，用户无需了解 loopy 寄存器。

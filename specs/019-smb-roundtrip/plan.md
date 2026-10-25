# Feature 019 实现计划

## 架构

```
crates/nesl/src/asm.rs            # 已有：Mode 改为 pub，新增 opcode_table()（256 项反查表）
crates/nesl/src/disasm/roundtrip.rs  # 新增：iNES 解析/重建、追踪式反汇编器、asm 文本生成
crates/nesl/src/disasm/asm_text.rs   # 新增：文本 6502 汇编器（两遍，输出 iNES）
crates/nesc/src/cmd/roundtrip.rs     # 新增：disasm-full / asm 两个 CLI 命令
crates/nesl/tests/smb_roundtrip.rs   # 新增：验收测试（见 spec FR-4）
```

## 关键设计决策

### 1. 指令表来源

`asm.rs::opcode(op, mode)` 已覆盖全部官方指令的编码。新增
`opcode_table()`：遍历所有 (Op, Mode) 组合调用 `opcode()` 反向生成
`[Option<(Op, Mode)>; 256]`。反汇编与汇编共用同一张表 ⇒ 互逆性由构造保证。

### 2. 代码/数据分离 = 可读性问题，而非正确性问题

输出按地址线性扫描生成：

```
pos = 0
while pos < prg.len():
    若 pos 处有标签 → 输出 "Lxxxx:"
    若 pos 处有追踪到的指令，且指令体内部无标签、不越过 PRG 末尾
        → 输出指令文本，pos += 长度
    否则 → 输出 .byte（至下一个标签/指令起点，每行 ≤16 字节）
```

任何追踪误判（数据被当代码、表扫描过头）都只影响文本形式，
重汇编后的字节不变。

### 3. JumpEngine 启发式

SMB 大量使用：

```
    JSR JumpEngine      ; $8E04：pla/pla 取返回地址，按 A 查表跳转
    .word L1, L2, ...   ; 紧随 JSR 之后的地址表
```

识别方式：JSR 目标例程的前几个字节匹配 JumpEngine 特征
（asl / tay / pla / sta zp / pla / sta zp / iny）。命中后：
- 该 JSR 视为控制流终点（不会返回到表上）；
- 从 JSR 之后逐字（word）读取：值在 `$8000..$FFFA` 内则作为代码入口加入
  工作队列，直到遇到越界值或已追踪为代码的位置。

### 4. 文本格式的字节级无歧义性

- 零页寻址打印 `$XX`（两位）；绝对寻址打印 `$XXXX`（四位，哪怕高字节为 0）。
- 分支/JMP/JSR 的 PRG 内目标用标签；layout 固定（单一 `.org $8000` 顺序输出），
  相对偏移必然复原。
- 数据引用（如 `LDA $9FAA,X`）用四位字面量。
- 向量区 `$FFFA-$FFFF` 作为普通字节流中的 `.word NMI, Reset, IRQ` 输出。

### 5. 帧级验证

nes-sim 已具备：NROM、sprite-0 hit 时序、逐像素渲染（含逐行水平卷轴，
SMB 状态栏分屏依赖此）。冒烟测试已确认标题画面正确渲染。

测试辅助：`fn run_script(nes, &[(frame, buttons)])` 逐帧设置手柄并运行；
检查点处 `render::frame()` 对比两台机器输出向量。
按键位：A=$80 B=$40 Select=$20 Start=$10 U=$08 D=$04 L=$02 R=$01。

演示/操作检查点帧号在实现期用冒烟脚本实测后写死进测试（附注释）。

## 任务拆分

见 tasks.md。红（test: 提交失败测试+桩）→ 绿（feat: 实现）→ 工件与文档。

## 风险

| 风险 | 缓解 |
|------|------|
| 追踪误入数据导致指令跨标签 | 发射阶段检测并退化 .byte（不变式见 spec FR-1） |
| 文本格式歧义导致重汇编漂移 | 测试 2 逐字节对比整个 .nes 文件 |
| nes-sim 行为与真机差异 | 不影响本 feature 的"一致性"定义：两台机器同为 nes-sim，
  对比的是原版 vs 重建版 |

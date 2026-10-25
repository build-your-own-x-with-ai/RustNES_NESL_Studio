# Feature 020 实现计划

## 架构

```
crates/nesl/src/disasm/roundtrip.rs   # 重构：trace 阶段抽出 pub(crate) trace_rom()，
                                      #       Trace 增加 jump_tables（JSR 点 → 表项列表）
crates/nesl/src/decompile/c_codegen.rs # 新增：C 代码生成器（翻译器 + 运行时模板）
crates/nesc/src/cmd/roundtrip.rs       # 新增 decompile-c 子命令实现
crates/nesl/tests/smb_c_decompile.rs   # 新增：验收测试（调用 cc 编译、运行、对比帧）
```

## 关键设计

### 1. 函数划分（翻译单元）

- 入口集合 = {reset, nmi, irq} ∪ JSR 目标 ∪ 跳转表项。
- 每个入口从其地址做块内可达遍历：分支/JMP/顺序延伸；
  JSR = "调用后继续"；`JMP/分支 → 其他入口` = （条件）尾调用；
  RTS/RTI/JumpEngine 调用点 = 终点。
- 到达非入口地址的跨函数跳转：目标块复制进本函数（函数内标签
  `l_XXXX:` + goto）。正确性由共享全局状态保证。

### 2. 指令翻译要点

- 标志：n/z/c/v 独立 uint8_t；`sz(v)` 宏设 n/z。
- ADC/SBC/CMP/BIT/移位/ROL/ROR 按 cpu.rs 语义逐条对应（SBC = ADC(~m)）。
- PHA/PLA/PHP/PLP 操作 `ram[0x100|sp]`（真实栈）；JSR/RTS 用 C 调用栈
  （6502 返回地址不入 ram——SMB 除 JumpEngine 外不读返回地址，
  JumpEngine 调用点已翻译为 switch，不执行其本体）。
  NMI 入口 sp-=3 / 返回 sp+=3，保持 sp 值一致。
- 静态地址分派：<$2000 → `ram[a & 0x7FF]`；$8000+ → `prg[]`（只读）；
  I/O → 专用助手；动态地址（索引/间接）→ `rd()/wr()` 运行时分派。
- 周期：基本块累计一次性 `cyc += N`，在任何 I/O 访问语句前结清；
  分支命中路径加常量（+1，跨页 +2，翻译期可算）；索引寻址跨页 +1
  在读助手内判定（`(base^ea)&0xFF00`）。

### 3. C 运行时（生成文件内嵌模板）

移植 nes-sim 的（mapper 0 子集，逐行对应）：
`vram_index/nt_index` 折叠、$2007 缓冲读、$2002 toggle 复位 +
sprite-0 hit（`bg_opaque`/`sprite0_hit`/`pixel_cycle` 整体移植，
首次轮询时惰性锁存本帧命中周期）、$4014 DMA（+513 cyc）、
$4016 strobe/移位、reg_log 记录 (cyc, kind, val)。

驱动循环（与 nes-sim `TestNes::step` 的帧边界语义对齐）：

```
reset();                     // 执行到主线程停机（JMP self），cyc 从 7 起
frames = cyc / CPF;（期间 NMI 关闭）
循环直到 end：
    处理本帧按键脚本 / 检查点快照（快照 = NMI 体执行前，
        对应 nes-sim run_frames 返回点，渲染索引 = frames-1）
    若 PPUCTRL bit7：sp-=3; cyc+=7; nmi_handler(); sp+=3; cyc+=6(RTI)
    停机空转：while (cyc < (frames+1)*CPF) cyc += 3;   // JMP self
    frames++（NMI 体内跨界的滞后帧：按 cyc 补增 frames，不补发 NMI，
        与 nes-sim "ppuctrl bit7 当时已清" 行为一致）
```

### 4. 快照桥（C → Rust 渲染对比）

快照二进制：magic + frames + cyc + ram + vram(0x4000) + oam +
ppuctrl/ppumask + reg_log 条目数组。Rust 测试：
`TestNes::from_ines(原 ROM)` 取 bus（镜像/CHR 就位），覆写
vram/oam/ppuctrl/ppumask/reg_log，`render::frame(&bus, frames-1)`
与原版同帧渲染结果逐像素比较。

### 5. 测试驱动的脚本协议

```
btn <frame> <mask>    # 从该帧起手柄 = mask
dump <frame> <path>   # 该帧（NMI 体前）写快照
end <frame>
```

## 风险对照 spec"已知风险"；全部由帧对比测试兜底。

## 任务拆分见 tasks.md

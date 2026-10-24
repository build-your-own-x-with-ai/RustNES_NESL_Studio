# Research: NESL Decompiler

**Date**: 2026-10-24

**Purpose**: 解决技术选型中的未决问题，调研反编译器实现的最佳实践

---

## 1. JSON 库选择：serde_json vs 手写 parser

### Decision
**接受 `serde_json` 作为依赖**

### Rationale
1. **符号表格式选择**：
   - JSON 是符号表的最佳格式：人类可读、可扩展、语言无关、工具链支持好
   - 开发者可以直接查看 `.syms` 文件调试问题（如 `cat hello.syms | jq .variables`）
   - 未来可能需要其他语言（Python/JS）读取符号表，JSON 是通用格式

2. **实现成本对比**：
   - `serde_json`：添加 2 行 `Cargo.toml`，派生 `#[derive(Serialize, Deserialize)]`，零实现成本
   - 手写 parser：需要 500+ 行代码（词法分析、语法分析、错误处理），增加维护负担和 bug 风险

3. **与宪章的一致性**：
   - 宪章说"简单"和"零依赖"，但真正的原则是 YAGNI（只做需要的）
   - 符号表需要结构化存储，JSON 是行业标准
   - 手写 parser 不是"简单"，而是增加不必要的复杂度

4. **生态成熟度**：
   - `serde_json` 是 Rust 生态的事实标准（42M+ 下载，活跃维护 8 年+）
   - 已被 cargo、rustc 等官方工具使用
   - 零 unsafe 代码，依赖少（只依赖 `serde`）

### Alternatives Considered
- **手写 JSON parser**：违背 YAGNI，增加维护成本
- **二进制格式（如 bincode）**：不可读，调试困难，不符合"可验证的输出"
- **S-expression 或自定义 DSL**：非标准格式，工具链支持差

---

## 2. 符号表存储位置：嵌入 ROM vs 独立文件

### Decision
**优先生成独立 `.syms` 文件，仅当用户明确要求时才嵌入 ROM**

### Rationale
1. **ROM 空间限制**：
   - NROM（32KB PRG）：符号表可能占 4-8KB，压缩空间
   - UNROM/MMC3：有空余 bank 时可嵌入，但需要动态判断
   - 独立文件：无大小限制，可包含完整源码映射和注释

2. **分发场景**：
   - 反编译主要用于开发调试，不是最终用户功能
   - 分发 `.nes` 时不需要符号表（减小文件大小）
   - 嵌入方案增加复杂度（需要 ROM 解析器支持魔数识别、解压缩）

3. **实现简单性**：
   - 独立文件：编译器在输出 `.nes` 同时写 `.syms`，反编译器读取两个文件
   - 嵌入 ROM：需要在 ROM 末尾预留区域、写入魔数和长度、可能需要压缩

### Alternatives Considered
- **总是嵌入 ROM**：增加 ROM 大小，分发不便
- **可选嵌入**：增加编译器和反编译器的复杂度，YAGNI

**CLI 接口**:
```bash
nesc build hello.nesl --debug-symbols    # 生成 hello.nes + hello.syms
nesc decompile hello.nes -o hello.dec.nesl  # 自动找 hello.syms
nesc decompile hello.nes --symbols custom.syms -o out.nesl  # 手动指定
```

---

## 3. 控制流重建策略

### Decision
**使用模式匹配重建高级控制流，回退到 `loop { if !cond { break; } ... }` 形式**

### Rationale
1. **NESL 控制流编译后的特征**：
   - `if cond { A } else { B }` → 编译为 `BEQ .else; A; JMP .end; .else: B; .end:`
   - `while cond { body }` → `JMP .test; .loop: body; .test: BEQ .loop`
   - `for i in a..b` → 循环变量初始化 + 比较 + 递增的固定模式

2. **重建精度权衡**：
   - 完美重建需要控制流分析（CFG、dominator tree），实现复杂
   - 基于模式的重建可以覆盖 90% 的常见情况（NESL 编译器输出固定模式）
   - 对于复杂嵌套或优化后的代码，回退到等价但不完美的形式

3. **可接受的差异**：
   - 原始：`while x < 10 { x += 1; }`
   - 反编译：`loop { if !(x < 10) { break; } x += 1; }`（语义相同）

### Alternatives Considered
- **完全精确重建**：需要 SSA 分析、数据流分析，违背 YAGNI
- **输出汇编注释**：失去高级语言的可读性

**实现要点**：
- 识别常见的分支模式（if/else、while、for）
- 对于无法识别的模式，生成带注释的 `loop { break }` 形式
- 符号表包含源码行号映射，辅助识别控制流边界

---

## 4. 内建函数调用识别

### Decision
**维护 JSR 地址到内建函数的映射表，从符号表读取**

### Rationale
1. **编译器知道内建函数的地址**：
   - 每个内建函数（如 `__sprite`、`__print`）在运行时库中有固定地址
   - 符号表可包含 `builtins` 字段：`{ "__sprite": 0xC000, "__print": 0xC030, ... }`

2. **反编译器查表还原**：
   - 遇到 `JSR 0xC000` → 查符号表 → 输出 `sprite(i, x, y, tile, attr)`
   - 参数从调用前的寄存器/栈操作推断（固定模式）

3. **无需启发式分析**：
   - 不需要猜测"这个 JSR 是什么"
   - 符号表是真实来源（source of truth）

### Alternatives Considered
- **启发式识别**：容易误判，不可靠
- **输出 JSR 汇编**：失去高级 API 的可读性

**符号表扩展**：
```json
{
  "builtins": {
    "__sprite": { "addr": "0xC000", "signature": "sprite(i: u8, x: u8, y: u8, tile: u8, attr: u8)" },
    "__print": { "addr": "0xC030", "signature": "print(x: u8, y: u8, text: &[u8])" }
  }
}
```

---

## 5. 资源还原（tile/map/music）

### Decision
**符号表标记资源类型和位置，反编译器专门处理**

### Rationale
1. **tile**：
   - CHR-ROM 数据（$0000-$1FFF）可直接转换为 8x8 像素矩阵
   - 符号表记录 `{ "tile_1": { "type": "tile", "chr_offset": 0x10 } }`
   - 反编译器读取 16 字节（2 planes），转换为 `.123` 字符串

2. **map**：
   - 地图数据在 PRG-ROM 中（符号表给出地址和尺寸）
   - 反编译器读取字节序列，转换为字符串数组

3. **music**：
   - MML 字符串存储在 PRG-ROM 的常量数组中
   - 符号表标记类型为 `music`，反编译器输出 `music THEME = "...";`

### Alternatives Considered
- **纯启发式识别**：无法区分普通数组和资源
- **不还原资源**：输出为 `const DATA: [u8] = [...]`（可读性差）

---

## 6. 往返测试策略

### Decision
**测试语义等价性（ROM 行为），而非逐字节相同**

### Rationale
1. **编译器的非确定性因素**：
   - 变量地址可能重新分配（不影响逻辑）
   - 符号表中的顺序可能不同
   - 注释和空白会丢失

2. **语义等价性验证方法**：
   - 编译原始 `hello.nesl` → `hello.nes`
   - 反编译 `hello.nes` → `hello.dec.nesl`
   - 编译 `hello.dec.nesl` → `hello2.nes`
   - 在 `nes-sim` 中运行两个 ROM，执行相同操作，断言内存/VRAM 状态相同

3. **测试覆盖**：
   - 每个 `examples/*.nesl` 都有往返测试
   - 特殊情况：UNROM（多 bank）、MMC3（IRQ handler）

### Alternatives Considered
- **逐字节比较**：过于严格，编译器优化/地址分配的微小变化会导致失败
- **只测反编译不崩溃**：不验证正确性

---

## 7. 错误处理

### Decision
**反编译器必须优雅处理不完整或损坏的输入**

### Rationale
1. **常见错误场景**：
   - 符号表丢失或损坏
   - ROM 不是 NESL 生成的（没有魔数或符号表引用）
   - 符号表版本不匹配

2. **处理策略**：
   - 检测 ROM 是否包含 NESL 符号表引用（PRG-ROM 固定位置的魔数）
   - 如果符号表不存在，报错："此 ROM 不包含调试符号，无法反编译"
   - 如果符号表损坏，尝试部分还原，输出警告："符号表不完整，部分代码用占位符"

3. **用户体验**：
   - 清晰的错误信息，指出问题和解决方法
   - 不输出错误的 NESL 代码（宁可拒绝，也不要误导）

### Alternatives Considered
- **启发式反编译无符号表的 ROM**：不可靠，容易产生错误代码
- **静默失败**：用户体验差

---

## Summary

| 决策项 | 选择 | 关键理由 |
|--------|------|---------|
| JSON 库 | `serde_json` | 行业标准，零实现成本，可读性 |
| 符号表位置 | 独立 `.syms` 文件 | 简单，无大小限制，分发友好 |
| 控制流重建 | 模式匹配 + 回退 | 覆盖常见情况，接受等价输出 |
| 内建函数识别 | 符号表映射表 | 精确，无需启发式 |
| 资源还原 | 符号表标记 + 专门处理 | 保留高级抽象（tile/map） |
| 往返测试 | 语义等价性 | 实用，适应编译器优化 |
| 错误处理 | 优雅拒绝 | 清晰错误信息，不误导用户 |

所有决策已解决，无剩余 NEEDS CLARIFICATION。

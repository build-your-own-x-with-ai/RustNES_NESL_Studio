# Feature 015 实现计划

## Phase 1: 变量识别 (Zero Page 分析)

### 任务 1.1: 扫描 Zero Page 访问
- [ ] 实现 `scan_zeropage_accesses(prg_rom) -> HashMap<u8, Vec<Access>>`
- [ ] 记录每个 ZP 地址的读写模式
- [ ] 追踪访问该地址的指令类型

### 任务 1.2: 类型推断
- [ ] 检测 u16 模式（连续两字节访问）
- [ ] 检测数组模式（索引访问 ZP,X / ZP,Y）
- [ ] 检测计数器模式（INC/DEC 频繁）

### 任务 1.3: 变量命名
- [ ] 基于地址的默认命名（var_02）
- [ ] 基于模式的启发式命名（player_x, counter, flags）
- [ ] NES 约定的特殊地址命名（$00-$01 = ptr）

## Phase 2: 函数识别 (JSR/RTS 分析)

### 任务 2.1: 识别函数入口
- [ ] 扫描所有 JSR 指令
- [ ] 记录调用图（caller → callee）
- [ ] 识别中断向量函数（NMI, RESET, IRQ）

### 任务 2.2: 识别函数边界
- [ ] 从 JSR 目标开始追踪到 RTS/RTI
- [ ] 处理尾调用（JMP）
- [ ] 处理多出口函数（多个 RTS）

### 任务 2.3: 函数命名
- [ ] 基于地址命名（sub_8056）
- [ ] 基于调用位置命名（nmi_handler, update_player）
- [ ] 检测库函数模式（memcpy, multiply）

## Phase 3: 伪代码生成

### 任务 3.1: 指令翻译
- [ ] LDA/STA → 变量赋值（var = value）
- [ ] INC/DEC → 自增自减（var++）
- [ ] ADC/SBC → 算术运算（var += value）
- [ ] AND/ORA/EOR → 位运算（var &= mask）

### 任务 3.2: 寄存器抽象
- [ ] A/X/Y 寄存器使用追踪
- [ ] 临时变量生成（temp_a, temp_x）
- [ ] 寄存器重命名（A = player_x）

### 任务 3.3: 函数调用翻译
- [ ] JSR → call func()
- [ ] 参数识别（调用前的 LDA/LDX/LDY）
- [ ] 返回值识别（调用后的 A）

### 任务 3.4: 控制流翻译（简化版）
- [ ] BEQ/BNE → if (condition)
- [ ] 向后跳转 → while loop
- [ ] JMP 表 → switch case（可选）

## Phase 4: 命令行集成

### 任务 4.1: 新增命令
- [ ] `nesc pseudocode <file.nes>` 命令
- [ ] `-o <output>` 输出文件选项
- [ ] `--format <c|nesl>` 格式选择

### 任务 4.2: 输出格式化
- [ ] C-like 伪代码格式
- [ ] NESL-like 格式（可选）
- [ ] 注释和元数据

## Phase 5: 测试验证

### 任务 5.1: 单元测试
- [ ] 变量识别测试
- [ ] 函数边界测试
- [ ] 指令翻译测试

### 任务 5.2: 集成测试
- [ ] NESL 示例 ROM 测试
- [ ] 简单商业 ROM 测试（Donkey Kong, Balloon Fight）
- [ ] 复杂商业 ROM 测试（Super Mario Bros, Mega Man）

### 任务 5.3: 质量评估
- [ ] 函数识别准确率统计
- [ ] 变量识别准确率统计
- [ ] 可读性主观评分

## 时间估算

| Phase | 任务数 | 预计时间 |
|-------|--------|---------|
| Phase 1 | 3 | 2-3 小时 |
| Phase 2 | 3 | 2-3 小时 |
| Phase 3 | 4 | 3-4 小时 |
| Phase 4 | 2 | 1-2 小时 |
| Phase 5 | 3 | 2-3 小时 |
| **总计** | **15** | **10-15 小时** |

## MVP 定义

最小可行产品包括：
- ✅ Phase 1: 变量识别
- ✅ Phase 2: 函数识别
- ✅ Phase 3.1-3.3: 基础伪代码生成
- ✅ Phase 4: 命令行集成

**交付标准：**
- 可以处理任意 NES ROM
- 识别 60%+ 函数和变量
- 生成可读的伪代码（比汇编好 2-3 倍）

## 下一步

从 **Phase 1.1** 开始实现变量识别。

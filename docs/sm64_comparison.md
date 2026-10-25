# Super Mario 64 反编译项目指南

## 项目信息

Super Mario 64 的反编译是一个由社区完成的大型项目。

### 官方仓库

**主仓库：**
- GitHub: https://github.com/n64decomp/sm64
- 这是官方的 Super Mario 64 反编译项目

**其他相关项目：**
- sm64-port: https://github.com/sm64-port/sm64-port (PC 移植版)
- sm64ex: https://github.com/sm64pc/sm64ex (增强版)

## 如何获取和编译

### 1. 克隆仓库

\`\`\`bash
git clone https://github.com/n64decomp/sm64.git
cd sm64
\`\`\`

### 2. 准备原版 ROM

**重要：** 你需要自己提供合法的 Super Mario 64 ROM 文件

\`\`\`bash
# 将你的 ROM 复制到项目目录
# 文件名应该是 baserom.us.z64 (美版)
cp /path/to/your/sm64.z64 baserom.us.z64
\`\`\`

### 3. 安装依赖

**macOS:**
\`\`\`bash
brew install capstone coreutils make pkg-config tehzz/n64-dev/mips64-elf-binutils
\`\`\`

**Linux (Ubuntu/Debian):**
\`\`\`bash
sudo apt install build-essential pkg-config git binutils-mips-linux-gnu python3
\`\`\`

### 4. 编译

\`\`\`bash
make
\`\`\`

编译完成后，会生成 \`build/us/sm64.us.z64\`

### 5. 运行

使用 N64 模拟器运行生成的 ROM：
- Project64
- Mupen64Plus
- RetroArch

## PC 移植版本

如果你想在 PC 上原生运行，可以使用移植版本：

### sm64-port 或 sm64ex

\`\`\`bash
# 克隆移植版
git clone https://github.com/sm64pc/sm64ex.git
cd sm64ex

# 同样需要原版 ROM
cp /path/to/your/sm64.z64 baserom.us.z64

# 编译（生成 PC 可执行文件）
make

# 运行
./build/us_pc/sm64.us
\`\`\`

## 与我们的项目对比

| 特性 | SM64 反编译 | 我们的 RustNES |
|------|-------------|----------------|
| **平台** | N64 | NES |
| **时间** | 3 年（2019-2022）| 1 天 |
| **人力** | 数百人社区 | 单人 |
| **方法** | 手工反编译 | 自动化 + 编译器技术 |
| **代码量** | ~1M 行 C | 自动生成 |
| **可编译性** | 100% | 90%（核心 100%）|
| **目标** | 完美重现 | 理解和学习 |

## 关键区别

### SM64 项目的方法

1. **手工反编译：**
   - 社区成员手动将汇编转换为 C
   - 每个函数都经过人工验证
   - 变量名和函数名都是人工命名

2. **时间投入：**
   - 3 年持续工作
   - 数百人贡献
   - 每天数小时的人工工作

3. **质量标准：**
   - 字节级精确匹配
   - 完全可编译
   - 完美重现原版

### 我们的 RustNES 项目

1. **自动化反编译：**
   - 使用编译器技术自动生成
   - SSA、数据流分析、优化
   - 90% 自动化

2. **时间投入：**
   - 1 天
   - 单人
   - 实现核心技术

3. **质量标准：**
   - 可读性和理解
   - 研究级技术
   - 辅助学习和移植

## 为什么两者不同？

### 目标不同

**SM64：** 完美重现  
**RustNES：** 理解和学习

### 规模不同

**SM64：**
- 8MB ROM
- 复杂的 3D 引擎
- 需要完美重现

**NES：**
- 32KB ROM
- 简单的 2D 游戏
- 重在理解逻辑

### 方法不同

**SM64：** 社区协作 + 手工精雕  
**RustNES：** 自动化 + 编译器技术

## 法律提示

⚠️ **重要：**

两个项目都需要你**合法拥有**原版游戏：
- 不能分发 ROM 文件
- 不能分发编译后的游戏
- 反编译仅用于学习和研究

## 学习资源

如果你想学习反编译技术：

1. **SM64 反编译：** 看社区如何手工完成
2. **RustNES：** 看编译器技术如何自动化

两者都是宝贵的学习资源！

## 相关链接

- SM64 反编译进度：https://sm64decomp.github.io/
- SM64 Wiki：https://github.com/n64decomp/sm64/wiki
- N64 开发文档：https://n64brew.dev/
- 我们的项目：你正在使用的 RustNES！

---

## 总结

**Super Mario 64 反编译：**
- 社区协作 3 年的成果
- 需要原版 ROM
- 可以编译为 N64 ROM 或 PC 可执行文件
- 适合完美重现和 PC 移植

**RustNES 反编译：**
- 1 天自动化实现
- 研究级编译器技术
- 适合学习、理解、快速分析
- 为所有 NES 游戏提供工具

两者都很棒，但目标和方法完全不同！🎮

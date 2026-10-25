# 如何让 Super Mario Bros 反编译代码可玩

## 问题

反编译的代码只有程序逻辑（PRG），没有图形数据（CHR），所以会黑屏。

## 解决方案

有几种方法可以让游戏可玩：

---

## 方案 1: 使用原版 ROM（推荐，最简单）

**直接使用原版 ROM 玩游戏：**

```bash
# 用模拟器运行原版
open "roms/Super Mario Bros (W) [!].nes"
```

**用反编译代码学习逻辑：**

```bash
# 查看反编译代码
cat output/decompiled/super_mario_bros.nesl

# 理解游戏如何工作
# 然后在原版中验证
```

---

## 方案 2: 混合方案（技术上可行）

**概念：** 将反编译代码编译的 PRG + 原版 ROM 的 CHR 合并

### 需要的步骤：

1. **编译反编译代码**
   ```bash
   nesc build output/decompiled/super_mario_bros.nesl -o mario_prg.bin
   ```

2. **从原版提取 CHR**
   ```bash
   # CHR ROM 在偏移 16 + PRG_SIZE
   # Mario 有 32KB PRG + 8KB CHR
   dd if="mario_original.nes" of="mario_chr.bin" bs=1 skip=32784 count=8192
   ```

3. **合并成新 ROM**
   ```bash
   # 创建新 ROM 头
   # NES 头 (16 字节) + 新 PRG + 原版 CHR
   cat nes_header.bin mario_prg.bin mario_chr.bin > mario_new.nes
   ```

### 问题：

- ⚠️ 反编译代码不完整（只有部分函数）
- ⚠️ 缺少数据段（关卡、调色板等）
- ⚠️ 地址可能不匹配

**结论：技术上可行，但需要大量手工工作**

---

## 方案 3: 完整反编译项目（长期）

参考 Super Mario 64 反编译：

### 需要：

1. **完整的代码反编译**
   - ✅ 我们有 586 行（35 个函数）
   - ❌ 需要所有 310 个函数
   - ❌ 需要数据段

2. **图形数据提取**
   - 从原版 ROM 提取 CHR
   - 转换为 NESL tile 定义
   - 约 256 个 8x8 图块

3. **关卡数据重建**
   - 世界 1-1 到 8-4
   - 敌人位置
   - 物品位置

4. **音乐和音效**
   - APU 音乐数据
   - 音效定义

### 时间估计：

- **社区协作**: 6-12 个月
- **单人**: 1-2 年

---

## 方案 4: 快速演示（实用）

创建一个简化的可玩版本：

### 步骤：

1. **使用原版 CHR**
   ```nesl
   // 在 NESL 中引用原版 CHR
   chr_rom "mario_original.chr"
   ```

2. **简化逻辑**
   ```nesl
   // 只实现基本功能
   fn main() {
       load_chr("mario.chr");
       show_title_screen();
       game_loop();
   }
   ```

3. **重建关键数据**
   ```nesl
   // 定义调色板
   palette bg_palette {
       0x0F, 0x30, 0x10, 0x00,  // 背景
       ...
   }
   ```

---

## 当前最佳实践

### 学习和研究：

✅ **查看反编译代码** - 理解逻辑
✅ **运行原版 ROM** - 看实际效果
✅ **对照学习** - 代码 ↔ 游戏

### 开发新游戏：

✅ **使用其他示例** - sprite.nesl, game.nesl
✅ **创建自己的图形** - 使用 tile 定义
✅ **编写新逻辑** - 基于 Mario 的模式

---

## 技术细节

### NES ROM 结构：

```
Offset  Size    Content
------  -----   ---------
0       16      NES Header
16      32KB    PRG ROM (程序)
32784   8KB     CHR ROM (图形) ← 我们需要这个
```

### CHR ROM 格式：

- 256 个 8x8 像素图块
- 每个图块 16 字节
- 2 位色深（4 色）
- 总共 8192 字节

### 提取 CHR：

```python
# Python 脚本
with open('mario.nes', 'rb') as f:
    f.seek(16 + 32768)  # 跳过头和 PRG
    chr_data = f.read(8192)
    
with open('mario.chr', 'wb') as f:
    f.write(chr_data)
```

---

## 总结

### 现在可以做的：

✅ **学习** - 查看反编译代码理解逻辑
✅ **玩** - 使用原版 ROM
✅ **开发** - 基于理解创建新游戏

### 将来可以做的：

🔄 **完整反编译** - 社区项目，6-12 个月
🔄 **混合 ROM** - 技术实验，需要手工工作
🔄 **NESL 版 Mario** - 完全重写，基于反编译的理解

---

## 推荐方案

**对于学习：**
1. 查看反编译代码
2. 运行原版 ROM
3. 理解两者的对应关系

**对于开发：**
1. 使用有图形的示例（sprite, game）
2. 创建自己的游戏
3. 借鉴 Mario 的代码模式

**对于完整 Mario：**
- 这是一个长期社区项目
- 需要多人协作
- 参考 SM64 反编译项目

---

**反编译的价值在于理解，而非复制！** 📚

用原版玩游戏，用反编译学习代码！🎮

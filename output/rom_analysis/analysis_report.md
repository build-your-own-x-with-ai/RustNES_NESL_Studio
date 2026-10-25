# NES ROM 批量分析报告

生成时间: $(date)

## 目标

分析所有 NES ROM，生成：
1. 伪代码（用于学习）
2. NESL 框架（用于移植）

---

## 分析结果

| ROM | 大小 | 函数数 | 变量数 | 状态 |
|-----|------|--------|--------|------|
| Megaman IV (E) [!].nes | 512KB | 25 | 68 | ✅ |
| Rockman 4 - Aratanaru Yabou!! (J) (PRG1).nes | 512KB | 25 | 68 | ✅ |
| Rockman 4 - Aratanaru Yabou!! (J) (PRG0) [!].nes | 512KB | 25 | 68 | ✅ |
| Megaman IV (U) (PRG0) [!].nes | 512KB | 25 | 68 | ✅ |
| Megaman IV (U) (PRG1) [!].nes | 512KB | 25 | 68 | ✅ |
| Megaman IV (W) (MMLC).nes | 512KB | 25 | 68 | ✅ |
| Takahashi Meijin no Boukenjima II (J) [!].nes | 256KB | 6 | 117 | ✅ |
| Adventure Island II (U) [!].nes | 256KB | 6 | 119 | ✅ |
| Adventure Island II (E) [!].nes | 256KB | 6 | 119 | ✅ |
| Exciting Boxing (J).nes | 256KB | 14 | 61 | ✅ |
| Dusty Diamond's All-Star Softball (U) [!].nes | 256KB | 277 | 150 | ✅ |
| Softball Tengoku (J).nes | 256KB | 268 | 142 | ✅ |
| Oeka Kids - Anpanman no Hiragana Daisuki (J).nes | 128KB | 153 | 147 | ✅ |
| Dragon Power (U) [!].nes | 160KB | 346 | 128 | ✅ |
| Dragon Ball - Shinryuu no Nazo (J) [!].nes | 160KB | 345 | 132 | ✅ |
| Dragon Ball - Le Secret Du Dragon (F) (PRG0) [!].nes | 160KB | 346 | 123 | ✅ |
| Dragon Ball - Le Secret Du Dragon (F) (PRG1) [!].nes | 160KB | 345 | 122 | ✅ |
| Hammerin' Harry (E) [!].nes | 256KB | 80 | 196 | ✅ |
| Daiku no Gen-san (J) [!].nes | 256KB | 88 | 194 | ✅ |
| Sansuu 5 & 6 Nen (J).nes | 64KB | 272 | 169 | ✅ |

---

## 统计

- **总计**: 0 个 ROM
- **成功**: 0 个
- **失败**: 0 个

## 输出文件

- 伪代码: `output/rom_analysis/pseudocode/`
- NESL 框架: `output/rom_analysis/nesl_frameworks/`

## 使用方法

### 学习游戏逻辑

```bash
# 查看伪代码
cat output/rom_analysis/pseudocode/mario.c
```

### 移植游戏

```bash
# 1. 查看 NESL 框架
cat output/rom_analysis/nesl_frameworks/mario.nesl

# 2. 人工完善 TODO 部分
# 3. 编译
nesc build mario.nesl -o mario_port.nes
```

---

生成时间: 2026年10月25日 星期日 15时31分49秒 CST

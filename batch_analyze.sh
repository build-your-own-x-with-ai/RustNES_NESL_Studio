#!/bin/bash
# 批量分析所有 NES ROM 并生成报告
# 这个版本只分析，不尝试编译

set -e

ROMS_DIR="${1:-roms/FC_ROMS-main/ROM}"
OUTPUT_DIR="output/rom_analysis"
REPORT="$OUTPUT_DIR/analysis_report.md"

# 创建输出目录
mkdir -p "$OUTPUT_DIR/pseudocode"
mkdir -p "$OUTPUT_DIR/nesl_frameworks"

# 初始化报告
cat > "$REPORT" << 'EOF'
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
EOF

echo
echo "════════════════════════════════════════════════════════"
echo "   📊 批量 ROM 分析工具"
echo "════════════════════════════════════════════════════════"
echo
echo "目标: 分析所有 ROM 并生成报告"
echo "输出: $OUTPUT_DIR"
echo

# 统计
TOTAL=0
SUCCESS=0
FAILED=0

# 限制数量（可以修改）
MAX_ROMS=20
echo "处理最多 $MAX_ROMS 个 ROM..."
echo

# 查找所有 .nes 文件
find "$ROMS_DIR" -name "*.nes" -type f | head -n $MAX_ROMS | while read rom; do
    TOTAL=$((TOTAL + 1))
    filename=$(basename "$rom")
    name="${filename%.nes}"

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "[$TOTAL] $filename"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # 获取 ROM 大小
    size=$(stat -f%z "$rom" 2>/dev/null || stat -c%s "$rom")
    size_kb=$((size / 1024))

    # 生成伪代码
    echo "  [1/2] 生成伪代码..."
    if result=$(target/release/nesc pseudocode "$rom" 2>&1); then
        # 保存伪代码
        echo "$result" > "$OUTPUT_DIR/pseudocode/${name}.c"

        # 统计
        functions=$(echo "$result" | grep -c "^void.*{.*//.*\$" || echo "0")
        variables=$(echo "$result" | grep -c "^u8\|^u16" || echo "0")

        echo "    ✅ 函数: $functions, 变量: $variables"

        # 生成 NESL 框架
        echo "  [2/2] 生成 NESL 框架..."
        if target/release/nesc generate-nesl "$rom" -o "$OUTPUT_DIR/nesl_frameworks/${name}.nesl" > /dev/null 2>&1; then
            echo "    ✅ NESL 框架已生成"
            status="✅"
            SUCCESS=$((SUCCESS + 1))
        else
            echo "    ⚠️  NESL 生成失败"
            status="⚠️"
        fi

        # 添加到报告
        echo "| $filename | ${size_kb}KB | $functions | $variables | $status |" >> "$REPORT"
    else
        echo "    ❌ 分析失败"
        echo "| $filename | ${size_kb}KB | - | - | ❌ |" >> "$REPORT"
        FAILED=$((FAILED + 1))
    fi

    echo
done

# 添加统计到报告
cat >> "$REPORT" << EOF

---

## 统计

- **总计**: $TOTAL 个 ROM
- **成功**: $SUCCESS 个
- **失败**: $FAILED 个

## 输出文件

- 伪代码: \`$OUTPUT_DIR/pseudocode/\`
- NESL 框架: \`$OUTPUT_DIR/nesl_frameworks/\`

## 使用方法

### 学习游戏逻辑

\`\`\`bash
# 查看伪代码
cat output/rom_analysis/pseudocode/mario.c
\`\`\`

### 移植游戏

\`\`\`bash
# 1. 查看 NESL 框架
cat output/rom_analysis/nesl_frameworks/mario.nesl

# 2. 人工完善 TODO 部分
# 3. 编译
nesc build mario.nesl -o mario_port.nes
\`\`\`

---

生成时间: $(date)
EOF

echo
echo "════════════════════════════════════════════════════════"
echo "   ✅ 分析完成"
echo "════════════════════════════════════════════════════════"
echo
echo "统计:"
echo "  总计: $TOTAL"
echo "  成功: $SUCCESS"
echo "  失败: $FAILED"
echo
echo "输出:"
echo "  报告: $REPORT"
echo "  伪代码: $OUTPUT_DIR/pseudocode/"
echo "  NESL: $OUTPUT_DIR/nesl_frameworks/"
echo
echo "查看报告: cat $REPORT"

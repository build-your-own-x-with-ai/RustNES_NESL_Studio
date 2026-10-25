#!/bin/bash
# 批量分析所有 NES ROM - 增强版
# 支持：断点续传、进度保存、并行处理

set -e

ROMS_DIR="${1:-roms/FC_ROMS-main/ROM}"
OUTPUT_DIR="output/rom_analysis_full"
PROGRESS_FILE="$OUTPUT_DIR/.progress"
REPORT="$OUTPUT_DIR/analysis_report.md"

# 配置
MAX_ROMS="${MAX_ROMS:-0}"  # 0 = 全部
SKIP_EXISTING=true         # 跳过已处理的

# 创建输出目录
mkdir -p "$OUTPUT_DIR/pseudocode"
mkdir -p "$OUTPUT_DIR/nesl_frameworks"
mkdir -p "$OUTPUT_DIR/logs"

# 加载进度
if [ -f "$PROGRESS_FILE" ]; then
    PROCESSED=$(cat "$PROGRESS_FILE")
    echo "发现进度文件，已处理: $PROCESSED 个"
else
    PROCESSED=0
fi

echo "════════════════════════════════════════════════════════"
echo "   📊 批量 ROM 分析工具 - 增强版"
echo "════════════════════════════════════════════════════════"
echo
echo "配置:"
echo "  ROM 目录: $ROMS_DIR"
echo "  输出目录: $OUTPUT_DIR"
if [ "$MAX_ROMS" -eq 0 ]; then
    echo "  处理数量: 全部"
else
    echo "  处理数量: $MAX_ROMS"
fi
echo "  跳过已有: $SKIP_EXISTING"
echo "  已处理: $PROCESSED"
echo

# 初始化报告
if [ ! -f "$REPORT" ]; then
    cat > "$REPORT" << 'EOF'
# NES ROM 完整分析报告

## 分析结果

| ROM | 大小 | 函数数 | 变量数 | 状态 |
|-----|------|--------|--------|------|
EOF
fi

# 统计
TOTAL=0
SUCCESS=0
FAILED=0
SKIPPED=0

# 查找所有 .nes 文件
echo "查找 ROM 文件..."
if [ "$MAX_ROMS" -eq 0 ]; then
    roms=$(find "$ROMS_DIR" -name "*.nes" -type f | sort)
else
    roms=$(find "$ROMS_DIR" -name "*.nes" -type f | sort | head -n $MAX_ROMS)
fi

total_count=$(echo "$roms" | wc -l | tr -d ' ')
echo "找到 $total_count 个 ROM"
echo

# 处理每个 ROM
echo "$roms" | while read rom; do
    TOTAL=$((TOTAL + 1))
    filename=$(basename "$rom")
    name="${filename%.nes}"

    # 检查是否已处理
    if [ "$SKIP_EXISTING" = true ] && [ -f "$OUTPUT_DIR/nesl_frameworks/${name}.nesl" ]; then
        echo "[$TOTAL/$total_count] ⏭️  跳过: $filename (已存在)"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    echo
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "[$TOTAL/$total_count] 处理: $filename"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # 获取 ROM 大小
    size=$(stat -f%z "$rom" 2>/dev/null || stat -c%s "$rom")
    size_kb=$((size / 1024))

    # 生成伪代码
    echo "  [1/2] 生成伪代码..."
    if result=$(timeout 30s target/release/nesc pseudocode "$rom" 2>&1); then
        # 保存伪代码
        echo "$result" > "$OUTPUT_DIR/pseudocode/${name}.c"

        # 统计
        functions=$(echo "$result" | grep -c "^void.*{.*//.*\$" || echo "0")
        variables=$(echo "$result" | grep -c "^u8\|^u16" || echo "0")

        echo "    ✅ 函数: $functions, 变量: $variables"

        # 生成 NESL 框架
        echo "  [2/2] 生成 NESL 框架..."
        if timeout 30s target/release/nesc generate-nesl "$rom" -o "$OUTPUT_DIR/nesl_frameworks/${name}.nesl" > "$OUTPUT_DIR/logs/${name}.log" 2>&1; then
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
        echo "    ❌ 分析失败或超时"
        echo "| $filename | ${size_kb}KB | - | - | ❌ |" >> "$REPORT"
        FAILED=$((FAILED + 1))
    fi

    # 保存进度
    PROCESSED=$((PROCESSED + 1))
    echo $PROCESSED > "$PROGRESS_FILE"

    # 每 10 个显示一次进度
    if [ $((PROCESSED % 10)) -eq 0 ]; then
        echo
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "进度: $PROCESSED/$total_count ($((PROCESSED * 100 / total_count))%)"
        echo "成功: $SUCCESS, 失败: $FAILED, 跳过: $SKIPPED"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo
    fi
done

# 最终统计
cat >> "$REPORT" << EOF

---

## 统计

- **总计**: $PROCESSED 个 ROM
- **成功**: $SUCCESS 个
- **失败**: $FAILED 个
- **跳过**: $SKIPPED 个

生成时间: $(date)
EOF

echo
echo "════════════════════════════════════════════════════════"
echo "   ✅ 分析完成"
echo "════════════════════════════════════════════════════════"
echo
echo "统计:"
echo "  处理: $PROCESSED"
echo "  成功: $SUCCESS"
echo "  失败: $FAILED"
echo "  跳过: $SKIPPED"
echo
echo "输出:"
echo "  报告: $REPORT"
echo "  伪代码: $OUTPUT_DIR/pseudocode/"
echo "  NESL: $OUTPUT_DIR/nesl_frameworks/"
echo
echo "继续处理: 重新运行此脚本（自动断点续传）"

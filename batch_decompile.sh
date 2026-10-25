#!/bin/bash
# 批量反编译 NES ROM 并重新编译
# 用途：测试反编译 → NESL → ROM 的完整循环

set -e

ROMS_DIR="${1:-roms/FC_ROMS-main/ROM}"
OUTPUT_DIR="output/batch_decompile"
LOG_FILE="$OUTPUT_DIR/batch.log"

# 创建输出目录
mkdir -p "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/nesl"
mkdir -p "$OUTPUT_DIR/roms"
mkdir -p "$OUTPUT_DIR/logs"

# 初始化日志
echo "================================================================" > "$LOG_FILE"
echo "  批量反编译测试" >> "$LOG_FILE"
echo "  开始时间: $(date)" >> "$LOG_FILE"
echo "================================================================" >> "$LOG_FILE"
echo >> "$LOG_FILE"

# 统计
TOTAL=0
SUCCESS_ANALYZE=0
SUCCESS_GENERATE=0
SUCCESS_COMPILE=0
FAILED=0

# 查找所有 .nes 文件（限制数量以避免过长时间）
MAX_ROMS=10
echo "查找 ROM 文件（最多 $MAX_ROMS 个）..."

find "$ROMS_DIR" -name "*.nes" -type f | head -n $MAX_ROMS | while read rom; do
    TOTAL=$((TOTAL + 1))
    filename=$(basename "$rom")
    name="${filename%.nes}"

    echo
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "[$TOTAL] 处理: $filename"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Step 1: 生成 NESL 框架
    echo "  [1/3] 生成 NESL 框架..."
    if target/release/nesc generate-nesl "$rom" -o "$OUTPUT_DIR/nesl/${name}.nesl" 2>&1 | tee "$OUTPUT_DIR/logs/${name}_generate.log"; then
        echo "  ✅ NESL 框架生成成功" | tee -a "$LOG_FILE"
        SUCCESS_GENERATE=$((SUCCESS_GENERATE + 1))

        # Step 2: 尝试编译 NESL（可能失败，因为是框架）
        echo "  [2/3] 尝试编译 NESL..."
        if target/release/nesc build "$OUTPUT_DIR/nesl/${name}.nesl" -o "$OUTPUT_DIR/roms/${name}_rebuilt.nes" 2>&1 | tee "$OUTPUT_DIR/logs/${name}_compile.log"; then
            echo "  ✅ 编译成功！" | tee -a "$LOG_FILE"
            SUCCESS_COMPILE=$((SUCCESS_COMPILE + 1))

            # Step 3: 对比 ROM 大小
            original_size=$(stat -f%z "$rom" 2>/dev/null || stat -c%s "$rom")
            rebuilt_size=$(stat -f%z "$OUTPUT_DIR/roms/${name}_rebuilt.nes" 2>/dev/null || stat -c%s "$OUTPUT_DIR/roms/${name}_rebuilt.nes")

            echo "  📊 大小对比:" | tee -a "$LOG_FILE"
            echo "     原始: $original_size bytes" | tee -a "$LOG_FILE"
            echo "     重建: $rebuilt_size bytes" | tee -a "$LOG_FILE"
        else
            echo "  ⚠️  编译失败（预期，需要人工完善）" | tee -a "$LOG_FILE"
        fi
    else
        echo "  ❌ NESL 框架生成失败" | tee -a "$LOG_FILE"
        FAILED=$((FAILED + 1))
    fi

    echo | tee -a "$LOG_FILE"
done

# 最终统计
echo >> "$LOG_FILE"
echo "================================================================" | tee -a "$LOG_FILE"
echo "  最终统计" | tee -a "$LOG_FILE"
echo "================================================================" | tee -a "$LOG_FILE"
echo "  总计: $TOTAL" | tee -a "$LOG_FILE"
echo "  NESL 生成成功: $SUCCESS_GENERATE" | tee -a "$LOG_FILE"
echo "  编译成功: $SUCCESS_COMPILE" | tee -a "$LOG_FILE"
echo "  失败: $FAILED" | tee -a "$LOG_FILE"
echo | tee -a "$LOG_FILE"
echo "  输出目录: $OUTPUT_DIR" | tee -a "$LOG_FILE"
echo "  日志文件: $LOG_FILE" | tee -a "$LOG_FILE"
echo "================================================================" | tee -a "$LOG_FILE"
echo
echo "完成！查看 $LOG_FILE 了解详情。"

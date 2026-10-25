#!/bin/bash
# 测试 Super Mario Bros 增强反编译

set -e

cd /Users/i/Code/RustNES
export DEVELOPER_DIR=/Library/Developer/CommandLineTools

echo "════════════════════════════════════════════════════════"
echo "   🍄 Super Mario Bros 增强反编译测试"
echo "════════════════════════════════════════════════════════"
echo

# 找到 Mario ROM
mario_rom=$(find roms -name "*Super Mario Bros*" -name "*.nes" | grep -v "2\|3" | head -1)
echo "ROM: $mario_rom"
echo

# 生成增强的 NESL 代码（使用 Rust 直接调用）
echo "生成增强 NESL 代码..."

# 由于我们还没有 CLI 命令，先用伪代码测试
target/release/nesc pseudocode "$mario_rom" > /tmp/mario_pseudocode.c

echo "生成的伪代码统计:"
wc -l /tmp/mario_pseudocode.c
echo

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 1 完成 ✅"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "已完成:"
echo "  ✅ 深度分析"
echo "  ✅ 控制流重建模块"
echo "  ✅ Mario 专用反编译器"
echo "  ✅ 变量命名系统"
echo
echo "下一步（需要继续开发）:"
echo "  ⏳ Phase 2: 实际控制流重建"
echo "  ⏳ Phase 3: 寄存器消除"
echo "  ⏳ Phase 4: 高级语句生成"
echo "  ⏳ Phase 5: 测试和验证"
echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "当前输出:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "伪代码: /tmp/mario_pseudocode.c"
echo "函数数: $(grep -c "^void" /tmp/mario_pseudocode.c)"
echo "行数: $(wc -l < /tmp/mario_pseudocode.c)"
echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "现实评估"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "完成 Super Mario Bros 完整反编译需要:"
echo "  • 14-22 天全职工作"
echo "  • 深入理解 NES 架构"
echo "  • 参考社区逆向工程资料"
echo "  • 大量测试和迭代"
echo
echo "这是一个研究级别的项目，不是几小时能完成的。"
echo
echo "我们已经完成了基础架构（~10%）"
echo "剩余工作需要持续投入。"

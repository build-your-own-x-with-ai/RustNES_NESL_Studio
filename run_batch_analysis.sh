#!/bin/bash
# 批量分析启动器 - 让用户选择处理方式

echo "════════════════════════════════════════════════════════"
echo "   🎮 NES ROM 批量分析工具"
echo "════════════════════════════════════════════════════════"
echo
echo "你有 1933 个 NES ROM 文件"
echo
echo "请选择处理方式："
echo
echo "  1. 快速测试 (20 个 ROM, ~1 分钟)"
echo "  2. 小批量 (100 个 ROM, ~5 分钟)"
echo "  3. 中批量 (500 个 ROM, ~25 分钟)"
echo "  4. 全部处理 (1933 个 ROM, ~96 分钟) ⚠️"
echo "  5. 自定义数量"
echo "  6. 继续未完成的处理（断点续传）"
echo
read -p "请选择 [1-6]: " choice

case $choice in
    1)
        echo "处理 20 个 ROM..."
        ./batch_analyze.sh
        ;;
    2)
        echo "处理 100 个 ROM..."
        MAX_ROMS=100 ./batch_analyze_all.sh
        ;;
    3)
        echo "处理 500 个 ROM..."
        MAX_ROMS=500 ./batch_analyze_all.sh
        ;;
    4)
        echo "⚠️  警告: 这将处理全部 1933 个 ROM，需要约 96 分钟"
        read -p "确认继续? [y/N]: " confirm
        if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
            MAX_ROMS=0 ./batch_analyze_all.sh
        else
            echo "已取消"
        fi
        ;;
    5)
        read -p "输入要处理的 ROM 数量: " custom
        MAX_ROMS=$custom ./batch_analyze_all.sh
        ;;
    6)
        echo "继续未完成的处理..."
        MAX_ROMS=0 ./batch_analyze_all.sh
        ;;
    *)
        echo "无效选择"
        exit 1
        ;;
esac

echo
echo "完成！查看输出:"
echo "  output/rom_analysis/     (快速测试)"
echo "  output/rom_analysis_full/ (完整处理)"

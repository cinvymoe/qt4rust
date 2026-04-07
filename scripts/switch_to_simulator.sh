#!/bin/bash
# 切换到模拟传感器模式脚本

set -e

echo "========================================="
echo "  切换到模拟传感器模式"
echo "========================================="
echo ""

# 检查配置文件是否存在
if [ ! -f "config/pipeline_config.toml" ]; then
    echo "❌ 错误: 找不到 config/pipeline_config.toml"
    exit 1
fi

# 备份配置文件
echo "📦 备份配置文件..."
cp config/pipeline_config.toml config/pipeline_config.toml.bak
echo "   ✅ 已备份到 config/pipeline_config.toml.bak"
echo ""

# 修改 use_simulator 为 true
echo "🔧 修改配置: use_simulator = true"
sed -i 's/use_simulator = false/use_simulator = true/' config/pipeline_config.toml
echo "   ✅ 已启用模拟传感器模式"
echo ""

# 显示模拟器配置
echo "📋 当前模拟器配置:"
echo "-----------------------------------"
grep -A 4 "\[simulator.weight\]" config/pipeline_config.toml | grep -v "^#"
echo "-----------------------------------"
echo ""

# 提示重启应用
echo "========================================="
echo "✅ 配置完成！"
echo "========================================="
echo ""
echo "下一步操作:"
echo "1. 重启应用程序:"
echo "   pkill qt-rust-demo && ./qt-rust-demo"
echo ""
echo "查看日志:"
echo "   tail -f logs/app.log"
echo ""
echo "如需切换到 Modbus TCP 模式，运行:"
echo "   ./scripts/switch_to_modbus.sh"
echo ""

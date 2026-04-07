#!/bin/bash
# 切换到 Modbus TCP 模式脚本

set -e

echo "========================================="
echo "  切换到 Modbus TCP 传感器模式"
echo "========================================="
echo ""

# 检查配置文件是否存在
if [ ! -f "config/pipeline_config.toml" ]; then
    echo "❌ 错误: 找不到 config/pipeline_config.toml"
    exit 1
fi

if [ ! -f "config/modbus_sensors.toml" ]; then
    echo "❌ 错误: 找不到 config/modbus_sensors.toml"
    exit 1
fi

# 备份配置文件
echo "📦 备份配置文件..."
cp config/pipeline_config.toml config/pipeline_config.toml.bak
echo "   ✅ 已备份到 config/pipeline_config.toml.bak"
echo ""

# 修改 use_simulator 为 false
echo "🔧 修改配置: use_simulator = false"
sed -i 's/use_simulator = true/use_simulator = false/' config/pipeline_config.toml
echo "   ✅ 已启用 Modbus TCP 模式"
echo ""

# 显示 Modbus 配置
echo "📋 当前 Modbus 配置:"
echo "-----------------------------------"
grep -A 3 "\[server\]" config/modbus_sensors.toml | grep -v "^#"
echo "-----------------------------------"
echo ""

# 测试网络连通性
MODBUS_HOST=$(grep "^host" config/modbus_sensors.toml | cut -d'"' -f2)
MODBUS_PORT=$(grep "^port" config/modbus_sensors.toml | awk '{print $3}')

echo "🌐 测试网络连通性..."
if ping -c 1 -W 2 "$MODBUS_HOST" > /dev/null 2>&1; then
    echo "   ✅ 主机 $MODBUS_HOST 可达"
else
    echo "   ⚠️  警告: 无法 ping 通 $MODBUS_HOST"
    echo "   请检查网络连接和 IP 地址配置"
fi
echo ""

# 测试端口连通性
echo "🔌 测试 Modbus TCP 端口..."
if command -v nc > /dev/null 2>&1; then
    if nc -zv -w 2 "$MODBUS_HOST" "$MODBUS_PORT" 2>&1 | grep -q "succeeded"; then
        echo "   ✅ 端口 $MODBUS_PORT 可连接"
    else
        echo "   ⚠️  警告: 无法连接到 $MODBUS_HOST:$MODBUS_PORT"
        echo "   请确认 Modbus Slave 设备已启动"
    fi
else
    echo "   ⚠️  未安装 nc 工具，跳过端口测试"
fi
echo ""

# 提示重启应用
echo "========================================="
echo "✅ 配置完成！"
echo "========================================="
echo ""
echo "下一步操作:"
echo "1. 确认 Modbus Slave 设备已启动"
echo "2. 检查 config/modbus_sensors.toml 配置是否正确"
echo "3. 重启应用程序:"
echo "   pkill qt-rust-demo && ./qt-rust-demo"
echo ""
echo "查看日志:"
echo "   tail -f logs/app.log"
echo ""
echo "如需恢复模拟模式，运行:"
echo "   ./scripts/switch_to_simulator.sh"
echo ""

#!/bin/bash
# 安装 qt-rust-demo 自动启动脚本

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== 安装 qt-rust-demo 自动启动 ===${NC}"

# 检查 adb 连接
if ! adb devices | grep -q "device$"; then
    echo -e "${RED}错误: 未检测到设备${NC}"
    exit 1
fi

SCRIPT_DIR="$(dirname "$0")"
INIT_SCRIPT="$SCRIPT_DIR/S01qt-rust-demo"

if [ ! -f "$INIT_SCRIPT" ]; then
    echo -e "${RED}错误: 未找到 S01qt-rust-demo 脚本${NC}"
    exit 1
fi

echo -e "${YELLOW}备份原有启动脚本...${NC}"
adb shell "[ -f /etc/init.d/S01hello_vanxaok ] && cp /etc/init.d/S01hello_vanxaok /etc/init.d/S01hello_vanxaok.bak" || true

echo -e "${YELLOW}停止原有服务...${NC}"
adb shell "/etc/init.d/S01hello_vanxaok stop" 2>/dev/null || true

echo -e "${YELLOW}推送新的启动脚本...${NC}"
adb push "$INIT_SCRIPT" /etc/init.d/S01hello_vanxaok
adb shell "chmod +x /etc/init.d/S01hello_vanxaok"

echo -e "${GREEN}✓ 启动脚本已安装${NC}"
echo ""
echo "现在可以使用以下命令控制应用："
echo -e "${YELLOW}  启动: adb shell '/etc/init.d/S01hello_vanxaok start'${NC}"
echo -e "${YELLOW}  停止: adb shell '/etc/init.d/S01hello_vanxaok stop'${NC}"
echo -e "${YELLOW}  重启: adb shell '/etc/init.d/S01hello_vanxaok restart'${NC}"
echo ""
echo "设备重启后将自动启动 qt-rust-demo"
echo ""
echo -e "${YELLOW}是否现在启动应用? (y/n)${NC}"
read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    adb shell "/etc/init.d/S01hello_vanxaok start"
    echo -e "${GREEN}✓ 应用已启动${NC}"
fi

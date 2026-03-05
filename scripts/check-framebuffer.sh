#!/bin/bash
# 检查设备 Framebuffer 配置

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}=== Framebuffer 配置检查 ===${NC}"
echo ""

# 检查 adb 连接
if ! adb devices | grep -q "device$"; then
    echo -e "${RED}错误: 未检测到设备${NC}"
    exit 1
fi

echo -e "${BLUE}1. 检查 Framebuffer 设备${NC}"
adb shell "ls -l /dev/fb*" 2>/dev/null || echo -e "${YELLOW}未找到 framebuffer 设备${NC}"
echo ""

echo -e "${BLUE}2. Framebuffer 信息${NC}"
echo -e "${YELLOW}虚拟分辨率:${NC}"
adb shell "cat /sys/class/graphics/fb0/virtual_size 2>/dev/null" || echo "无法读取"

echo -e "${YELLOW}物理分辨率:${NC}"
adb shell "cat /sys/class/graphics/fb0/modes 2>/dev/null" || echo "无法读取"

echo -e "${YELLOW}色深:${NC}"
adb shell "cat /sys/class/graphics/fb0/bits_per_pixel 2>/dev/null" || echo "无法读取"

echo -e "${YELLOW}名称:${NC}"
adb shell "cat /sys/class/graphics/fb0/name 2>/dev/null" || echo "无法读取"
echo ""

echo -e "${BLUE}3. 输入设备${NC}"
adb shell "ls -l /dev/input/event*" 2>/dev/null || echo -e "${YELLOW}未找到输入设备${NC}"
echo ""

echo -e "${BLUE}4. 测试 Framebuffer 访问权限${NC}"
adb shell "test -r /dev/fb0 && echo '可读' || echo '不可读'"
adb shell "test -w /dev/fb0 && echo '可写' || echo '不可写'"
echo ""

echo -e "${BLUE}5. 检查 fbset 工具${NC}"
if adb shell "which fbset" 2>/dev/null; then
    echo "fbset 可用，获取详细信息:"
    adb shell "fbset -fb /dev/fb0 -i" 2>/dev/null || echo "无法执行 fbset"
else
    echo -e "${YELLOW}fbset 工具未安装${NC}"
fi
echo ""

echo -e "${GREEN}检查完成${NC}"

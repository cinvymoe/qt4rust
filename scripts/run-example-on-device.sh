#!/bin/bash
# 在 ARM32 设备上通过 ADB 运行示例程序
# 用法: ./scripts/run-example-on-device.sh <package> <example_name> [timeout]
# 示例: ./scripts/run-example-on-device.sh qt-threading-utils blocking_usage
#       ./scripts/run-example-on-device.sh qt-threading-utils sensor_collector 10

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查参数
if [ $# -lt 2 ]; then
    echo -e "${RED}错误: 缺少参数${NC}"
    echo "用法: $0 <package> <example_name> [timeout]"
    echo ""
    echo "示例:"
    echo "  $0 qt-threading-utils blocking_usage"
    echo "  $0 qt-threading-utils sensor_collector 10"
    echo "  $0 sensor-simulator unified_sensor"
    exit 1
fi

PACKAGE=$1
EXAMPLE_NAME=$2
TIMEOUT=${3:-0}  # 默认无超时

# 目标架构
TARGET="armv7-unknown-linux-gnueabihf"

# 设备路径
DEVICE_PATH="/data/local/tmp/${EXAMPLE_NAME}"

echo -e "${BLUE}=== 在 ARM32 设备上运行示例程序 ===${NC}"
echo -e "${YELLOW}Package:${NC} ${PACKAGE}"
echo -e "${YELLOW}Example:${NC} ${EXAMPLE_NAME}"
echo -e "${YELLOW}Target:${NC} ${TARGET}"
echo ""

# 步骤 1: 编译示例
echo -e "${BLUE}[1/4] 编译示例程序...${NC}"
cargo build --release --target ${TARGET} --package ${PACKAGE} --example ${EXAMPLE_NAME}

if [ $? -ne 0 ]; then
    echo -e "${RED}编译失败${NC}"
    exit 1
fi

BINARY_PATH="target/${TARGET}/release/examples/${EXAMPLE_NAME}"

if [ ! -f "${BINARY_PATH}" ]; then
    echo -e "${RED}错误: 找不到编译后的二进制文件: ${BINARY_PATH}${NC}"
    exit 1
fi

echo -e "${GREEN}✓ 编译成功${NC}"
echo ""

# 步骤 2: 检查设备连接
echo -e "${BLUE}[2/4] 检查设备连接...${NC}"
if ! adb devices | grep -q "device$"; then
    echo -e "${RED}错误: 未检测到 ADB 设备${NC}"
    echo "请确保设备已连接并启用 USB 调试"
    exit 1
fi

echo -e "${GREEN}✓ 设备已连接${NC}"
echo ""

# 步骤 3: 推送到设备
echo -e "${BLUE}[3/4] 推送到设备...${NC}"
adb push ${BINARY_PATH} ${DEVICE_PATH}

if [ $? -ne 0 ]; then
    echo -e "${RED}推送失败${NC}"
    exit 1
fi

echo -e "${GREEN}✓ 推送成功${NC}"
echo ""

# 步骤 4: 运行程序
echo -e "${BLUE}[4/4] 运行程序...${NC}"
echo -e "${YELLOW}----------------------------------------${NC}"

if [ ${TIMEOUT} -gt 0 ]; then
    # 带超时运行
    adb shell "chmod +x ${DEVICE_PATH} && timeout ${TIMEOUT} ${DEVICE_PATH} || true"
else
    # 无超时运行
    adb shell "chmod +x ${DEVICE_PATH} && ${DEVICE_PATH}"
fi

EXIT_CODE=$?

echo -e "${YELLOW}----------------------------------------${NC}"

if [ ${EXIT_CODE} -eq 0 ]; then
    echo -e "${GREEN}✓ 程序执行完成${NC}"
else
    echo -e "${YELLOW}⚠ 程序退出码: ${EXIT_CODE}${NC}"
fi

echo ""
echo -e "${GREEN}=== 完成 ===${NC}"

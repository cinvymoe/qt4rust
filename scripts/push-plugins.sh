#!/bin/bash
# 推送 Qt 平台插件和依赖到设备

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== 推送 Qt 平台插件 ===${NC}"

DEVICE_DIR="/data/local/tmp/qt-rust-demo"

# 架构参数，默认 arm32
ARCH="${ARCH:-arm32}"

if [ "$ARCH" = "arm64" ]; then
	QT_PLUGIN_DIR="/usr/lib/aarch64-linux-gnu/qt6/plugins"
	LIB_BASE="/usr/lib/aarch64-linux-gnu"
else
	QT_PLUGIN_DIR="/usr/lib/arm-linux-gnueabihf/qt6/plugins"
	LIB_BASE="/usr/lib/arm-linux-gnueabihf"
fi

echo -e "${GREEN}使用 ${ARCH} 架构${NC}"

# 检查设备连接
if ! adb devices | grep -q "device$"; then
    echo -e "${RED}错误: 未检测到设备${NC}"
    exit 1
fi

# 创建插件目录
echo -e "${YELLOW}创建插件目录...${NC}"
adb shell "mkdir -p $DEVICE_DIR/plugins/platforms"
adb shell "mkdir -p $DEVICE_DIR/lib"

# 推送 linuxfb 插件
echo -e "${YELLOW}推送 linuxfb 平台插件...${NC}"
if [ -f "$QT_PLUGIN_DIR/platforms/libqlinuxfb.so" ]; then
    adb push "$QT_PLUGIN_DIR/platforms/libqlinuxfb.so" "$DEVICE_DIR/plugins/platforms/"
    echo -e "${GREEN}✓ libqlinuxfb.so 已推送${NC}"
else
    echo -e "${RED}✗ 未找到 libqlinuxfb.so${NC}"
    exit 1
fi

# 推送插件依赖
echo -e "${YELLOW}推送插件依赖库...${NC}"
PLUGIN_DEPS=(
    "libudev.so.1"
    "libmtdev.so.1"
    "libts.so.0"
    "libinput.so.10"
    "libdrm.so.2"
    "libevdev.so.2"
    "libwacom.so.9"
)

for lib in "${PLUGIN_DEPS[@]}"; do
    LIB_PATH="$LIB_BASE/$lib"
    if [ -f "$LIB_PATH" ]; then
        echo "  推送 $lib..."
        adb push "$LIB_PATH" "$DEVICE_DIR/lib/"
        
        # 同时推送实际文件
        if [ -L "$LIB_PATH" ]; then
            REAL_LIB=$(readlink -f "$LIB_PATH")
            if [ -f "$REAL_LIB" ]; then
                adb push "$REAL_LIB" "$DEVICE_DIR/lib/"
            fi
        fi
    else
        echo -e "${YELLOW}  ⚠ 未找到 $lib${NC}"
    fi
done

# 推送其他有用的插件
echo -e "${YELLOW}推送其他平台插件...${NC}"
for plugin in libqminimal.so libqoffscreen.so libqvnc.so; do
    if [ -f "$QT_PLUGIN_DIR/platforms/$plugin" ]; then
        echo "  推送 $plugin..."
        adb push "$QT_PLUGIN_DIR/platforms/$plugin" "$DEVICE_DIR/plugins/platforms/" 2>/dev/null || true
    fi
done

echo ""
echo -e "${GREEN}完成！${NC}"
echo ""
echo "验证插件："
echo -e "${YELLOW}  adb shell \"ls -l $DEVICE_DIR/plugins/platforms/\"${NC}"
echo ""
echo "测试运行："
echo -e "${YELLOW}  adb shell \"cd $DEVICE_DIR && ./run.sh\"${NC}"

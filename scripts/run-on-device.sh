#!/bin/bash
# 直接在设备上运行应用（不使用 run.sh）

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 架构参数，默认 arm32
ARCH="${ARCH:-arm32}"

echo -e "${GREEN}=== 在设备上运行 Qt 应用 ($ARCH) ===${NC}"

# 检查设备连接
if ! adb devices | grep -q "device$"; then
    echo -e "${RED}错误: 未检测到设备${NC}"
    exit 1
fi

DEVICE_DIR="/data/local/tmp/qt-rust-demo"

# 检查应用是否存在
if ! adb shell "test -f $DEVICE_DIR/qt-rust-demo" 2>/dev/null; then
    echo -e "${RED}错误: 应用未部署${NC}"
    echo "请先运行: ./deploy-to-device.sh"
    exit 1
fi

echo -e "${YELLOW}设置环境并运行应用...${NC}"
echo ""

# 根据架构选择显示平台
if [ "$ARCH" = "arm64" ]; then
    # ARM64 设备使用 Wayland + 软件渲染 (EGL 硬件加速不可用)
    adb shell << 'SHELL_EOF'
cd /data/local/tmp/qt-rust-demo
export LD_LIBRARY_PATH=/data/local/tmp/qt-rust-demo/lib:/usr/lib
export QT_PLUGIN_PATH=/data/local/tmp/qt-rust-demo/plugins
export XDG_RUNTIME_DIR=/var/run
export WAYLAND_DISPLAY=wayland-0
export WAYLAND_WAIT=1
export QT_QPA_PLATFORM=wayland
export QML2_IMPORT_PATH=/data/local/tmp/qt-rust-demo/qml_modules
export QT_QUICK_BACKEND=software
export QT_IM_MODULE=qtvirtualkeyboard
export QT_VIRTUALKEYBOARD_DESKTOP_DISABLE=0
./qt-rust-demo
SHELL_EOF
else
    # ARM32 设备使用 linuxfb
    adb shell << 'SHELL_EOF'
cd /data/local/tmp/qt-rust-demo
export LD_LIBRARY_PATH=/data/local/tmp/qt-rust-demo/lib:$LD_LIBRARY_PATH
export QT_PLUGIN_PATH=/data/local/tmp/qt-rust-demo/plugins
export QT_QPA_PLATFORM_PLUGIN_PATH=/data/local/tmp/qt-rust-demo/plugins
export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0
export QML2_IMPORT_PATH=/data/local/tmp/qt-rust-demo/qml_modules
export QT_QPA_FB_DISABLE_INPUT=0
export QT_IM_MODULE=qtvirtualkeyboard
export QT_VIRTUALKEYBOARD_DESKTOP_DISABLE=0
./qt-rust-demo
SHELL_EOF
fi

#!/bin/bash
# 诊断设备上的 Qt 应用部署状态

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}=== Qt 应用部署诊断 ===${NC}"
echo ""

DEVICE_DIR="/data/local/tmp/qt-rust-demo"

# 检查设备连接
if ! adb devices | grep -q "device$"; then
    echo -e "${RED}✗ 未检测到设备${NC}"
    exit 1
fi
echo -e "${GREEN}✓ 设备已连接${NC}"

# 检查应用目录
echo ""
echo -e "${BLUE}1. 检查应用目录${NC}"
if adb shell "test -d $DEVICE_DIR" 2>/dev/null; then
    echo -e "${GREEN}✓ 应用目录存在${NC}"
else
    echo -e "${RED}✗ 应用目录不存在${NC}"
    exit 1
fi

# 检查二进制文件
echo ""
echo -e "${BLUE}2. 检查二进制文件${NC}"
if adb shell "test -f $DEVICE_DIR/qt-rust-demo" 2>/dev/null; then
    echo -e "${GREEN}✓ 二进制文件存在${NC}"
    adb shell "ls -lh $DEVICE_DIR/qt-rust-demo"
else
    echo -e "${RED}✗ 二进制文件不存在${NC}"
fi

# 检查库目录
echo ""
echo -e "${BLUE}3. 检查共享库${NC}"
LIB_COUNT=$(adb shell "ls $DEVICE_DIR/lib/ 2>/dev/null | wc -l" || echo "0")
echo "  库文件数量: $LIB_COUNT"
if [ "$LIB_COUNT" -gt "10" ]; then
    echo -e "${GREEN}✓ 库文件已部署${NC}"
else
    echo -e "${YELLOW}⚠ 库文件可能不完整${NC}"
fi

# 检查关键库
echo "  检查关键库:"
CRITICAL_LIBS=(
    "libQt6Core.so.6"
    "libQt6Gui.so.6"
    "libQt6Qml.so.6"
    "libQt6Quick.so.6"
    "libQt6QuickControls2.so.6"
    "libQt6QuickTemplates2.so.6"
    "libGLdispatch.so.0"
)

for lib in "${CRITICAL_LIBS[@]}"; do
    if adb shell "test -f $DEVICE_DIR/lib/$lib" 2>/dev/null; then
        echo -e "    ${GREEN}✓${NC} $lib"
    else
        echo -e "    ${RED}✗${NC} $lib"
    fi
done

# 检查插件
echo ""
echo -e "${BLUE}4. 检查 Qt 平台插件${NC}"
if adb shell "test -d $DEVICE_DIR/plugins/platforms" 2>/dev/null; then
    echo -e "${GREEN}✓ 插件目录存在${NC}"
    
    if adb shell "test -f $DEVICE_DIR/plugins/platforms/libqlinuxfb.so" 2>/dev/null; then
        echo -e "${GREEN}✓ linuxfb 插件存在${NC}"
    else
        echo -e "${RED}✗ linuxfb 插件不存在${NC}"
    fi
    
    echo "  所有插件:"
    adb shell "ls -l $DEVICE_DIR/plugins/platforms/" 2>/dev/null || echo "  无法列出"
else
    echo -e "${RED}✗ 插件目录不存在${NC}"
fi

# 检查 QML 文件
echo ""
echo -e "${BLUE}5. 检查 QML 资源${NC}"
if adb shell "test -d $DEVICE_DIR/qml" 2>/dev/null; then
    echo -e "${GREEN}✓ QML 目录存在${NC}"
    QML_COUNT=$(adb shell "find $DEVICE_DIR/qml -name '*.qml' 2>/dev/null | wc -l" || echo "0")
    echo "  QML 文件数量: $QML_COUNT"
else
    echo -e "${YELLOW}⚠ QML 目录不存在${NC}"
fi

# 检查启动脚本
echo ""
echo -e "${BLUE}6. 检查启动脚本${NC}"
if adb shell "test -f $DEVICE_DIR/run.sh" 2>/dev/null; then
    echo -e "${GREEN}✓ 启动脚本存在${NC}"
    if adb shell "test -x $DEVICE_DIR/run.sh" 2>/dev/null; then
        echo -e "${GREEN}✓ 启动脚本可执行${NC}"
    else
        echo -e "${YELLOW}⚠ 启动脚本不可执行${NC}"
    fi
else
    echo -e "${RED}✗ 启动脚本不存在${NC}"
fi

# 检查 framebuffer
echo ""
echo -e "${BLUE}7. 检查 Framebuffer${NC}"
if adb shell "test -c /dev/fb0" 2>/dev/null; then
    echo -e "${GREEN}✓ /dev/fb0 存在${NC}"
    
    FB_PERMS=$(adb shell "ls -l /dev/fb0" 2>/dev/null | awk '{print $1}')
    echo "  权限: $FB_PERMS"
    
    if adb shell "test -r /dev/fb0 && test -w /dev/fb0" 2>/dev/null; then
        echo -e "${GREEN}✓ Framebuffer 可读写${NC}"
    else
        echo -e "${YELLOW}⚠ Framebuffer 权限不足${NC}"
        echo "  修复: adb shell \"chmod 666 /dev/fb0\""
    fi
    
    FB_SIZE=$(adb shell "cat /sys/class/graphics/fb0/virtual_size 2>/dev/null" || echo "未知")
    echo "  分辨率: $FB_SIZE"
else
    echo -e "${RED}✗ /dev/fb0 不存在${NC}"
fi

# 检查输入设备
echo ""
echo -e "${BLUE}8. 检查输入设备${NC}"
INPUT_COUNT=$(adb shell "ls /dev/input/event* 2>/dev/null | wc -l" || echo "0")
echo "  输入设备数量: $INPUT_COUNT"
if [ "$INPUT_COUNT" -gt "0" ]; then
    echo -e "${GREEN}✓ 输入设备存在${NC}"
    adb shell "ls -l /dev/input/event*" 2>/dev/null | head -5
else
    echo -e "${YELLOW}⚠ 未找到输入设备${NC}"
fi

# 总结
echo ""
echo -e "${GREEN}=== 诊断完成 ===${NC}"
echo ""
echo "建议操作:"
echo ""

# 根据诊断结果给出建议
if [ "$LIB_COUNT" -lt "10" ]; then
    echo -e "${YELLOW}1. 重新部署库文件:${NC}"
    echo "   ./deploy-to-device.sh"
    echo ""
fi

if ! adb shell "test -f $DEVICE_DIR/plugins/platforms/libqlinuxfb.so" 2>/dev/null; then
    echo -e "${YELLOW}2. 推送平台插件:${NC}"
    echo "   ./push-plugins.sh"
    echo ""
fi

if ! adb shell "test -r /dev/fb0 && test -w /dev/fb0" 2>/dev/null; then
    echo -e "${YELLOW}3. 修复 framebuffer 权限:${NC}"
    echo "   adb shell \"chmod 666 /dev/fb0\""
    echo ""
fi

echo "运行应用:"
echo "  adb shell \"cd $DEVICE_DIR && ./run.sh\""

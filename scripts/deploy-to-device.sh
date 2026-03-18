#!/bin/bash
# 部署 Qt Rust 应用到 Android/ARM 设备

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Qt Rust Demo 设备部署脚本 ===${NC}"

# 检查 adb 是否可用
if ! command -v adb &>/dev/null; then
	echo -e "${RED}错误: 未找到 adb 命令${NC}"
	echo "请安装 Android SDK Platform Tools"
	exit 1
fi

# 检查设备连接
echo -e "${YELLOW}检查设备连接...${NC}"
if ! adb devices | grep -q "device$"; then
	echo -e "${RED}错误: 未检测到设备${NC}"
	echo "请确保设备已连接并启用 USB 调试"
	adb devices
	exit 1
fi

echo -e "${GREEN}✓ 设备已连接${NC}"

# 配置
BINARY_PATH="target/armv7-unknown-linux-gnueabihf/release/qt-rust-demo"
DEVICE_DIR="/userdata/local/tmp/qt-rust-demo"
DEVICE_LIB_DIR="$DEVICE_DIR/lib"
QML_DIR="qml"

# 检查二进制文件是否存在
if [ ! -f "$BINARY_PATH" ]; then
	echo -e "${RED}错误: 未找到编译的二进制文件${NC}"
	echo "请先运行: cargo build --target armv7-unknown-linux-gnueabihf"
	exit 1
fi

echo -e "${YELLOW}创建设备目录...${NC}"
adb shell "mkdir -p $DEVICE_DIR"
adb shell "mkdir -p $DEVICE_LIB_DIR"
adb shell "mkdir -p $DEVICE_DIR/qml"

# 推送二进制文件
echo -e "${YELLOW}推送应用程序...${NC}"
adb push "$BINARY_PATH" "$DEVICE_DIR/qt-rust-demo"
adb shell "chmod +x $DEVICE_DIR/qt-rust-demo"
echo -e "${GREEN}✓ 应用程序已推送${NC}"

# 推送 QML 文件
echo -e "${YELLOW}推送 QML 资源...${NC}"
if [ -d "$QML_DIR" ]; then
	adb push "$QML_DIR/." "$DEVICE_DIR/qml/"
	echo -e "${GREEN}✓ QML 文件已推送${NC}"
else
	echo -e "${YELLOW}⚠ 未找到 QML 目录${NC}"
fi

# 收集并推送字体（解决 "Cannot load default config file" 及设备缺字体问题）
echo -e "${YELLOW}收集并推送字体...${NC}"
COLLECT_FONTS_SCRIPT="$(dirname "$0")/collect-fonts.sh"
if [ -f "$COLLECT_FONTS_SCRIPT" ]; then
	bash "$COLLECT_FONTS_SCRIPT"
fi

if [ -d "fonts" ] && [ "$(ls -A fonts 2>/dev/null)" ]; then
	adb shell "mkdir -p $DEVICE_DIR/fonts"
	adb push fonts/. "$DEVICE_DIR/fonts/"
	echo -e "${GREEN}✓ 字体已推送 ($(ls fonts/*.ttf fonts/*.otf 2>/dev/null | wc -l) 个字体文件 + fonts.conf)${NC}"
else
	echo -e "${YELLOW}⚠ 未找到字体文件，跳过字体推送${NC}"
fi

# 推送共享库（可选）
if [ -z "$SKIP_LIBS" ]; then
	echo -e "${YELLOW}推送共享库...${NC}"

	# 运行库收集脚本
	COLLECT_SCRIPT="$(dirname "$0")/collect-libs.sh"
	if [ -f "$COLLECT_SCRIPT" ]; then
		echo "  运行库收集脚本..."
		bash "$COLLECT_SCRIPT" >/dev/null 2>&1

		if [ -d "libs-to-deploy" ] && [ "$(ls -A libs-to-deploy 2>/dev/null)" ]; then
			echo "  推送所有收集的库 (共 $(ls libs-to-deploy | wc -l) 个文件)..."
			adb push libs-to-deploy/. "$DEVICE_LIB_DIR/" 2>&1 |
				grep -E "(pushed|skipped)" || echo "  推送完成"
			echo -e "${GREEN}✓ 所有库已推送${NC}"
		else
			echo -e "${RED}错误: 库收集失败${NC}"
			exit 1
		fi
	else
		echo -e "${RED}错误: 未找到 collect-libs.sh 脚本${NC}"
		exit 1
	fi
else
	echo -e "${YELLOW}⊘ 跳过共享库推送${NC}"
fi

# 推送 Qt 平台插件（可选）
if [ -z "$SKIP_QT_PLUGINS" ]; then
	echo -e "${YELLOW}推送 Qt6 平台插件...${NC}"
	QT_PLUGIN_DIR="/usr/lib/arm-linux-gnueabihf/qt6/plugins"

	if [ -d "$QT_PLUGIN_DIR/platforms" ]; then
		adb shell "mkdir -p $DEVICE_DIR/plugins/platforms"

		# 推送 linuxfb 插件
		if [ -f "$QT_PLUGIN_DIR/platforms/libqlinuxfb.so" ]; then
			echo "  推送 linuxfb 平台插件..."
			adb push "$QT_PLUGIN_DIR/platforms/libqlinuxfb.so" "$DEVICE_DIR/plugins/platforms/"
			echo -e "${GREEN}✓ linuxfb 插件已推送${NC}"
		fi

		# 推送其他可能有用的插件
		for plugin in libqminimal.so libqoffscreen.so; do
			if [ -f "$QT_PLUGIN_DIR/platforms/$plugin" ]; then
				echo "  推送 $plugin..."
				adb push "$QT_PLUGIN_DIR/platforms/$plugin" "$DEVICE_DIR/plugins/platforms/" 2>/dev/null || true
			fi
		done
	fi

	# 推送图像格式插件（SVG 支持）
	if [ -d "$QT_PLUGIN_DIR/imageformats" ]; then
		echo -e "${YELLOW}推送图像格式插件...${NC}"
		adb shell "mkdir -p $DEVICE_DIR/plugins/imageformats"

		if [ -f "$QT_PLUGIN_DIR/imageformats/libqsvg.so" ]; then
			echo "  推送 SVG 图像格式插件..."
			adb push "$QT_PLUGIN_DIR/imageformats/libqsvg.so" "$DEVICE_DIR/plugins/imageformats/"
			echo -e "${GREEN}✓ SVG 插件已推送${NC}"
		fi
	fi

	# 推送图标引擎插件
	if [ -d "$QT_PLUGIN_DIR/iconengines" ]; then
		adb shell "mkdir -p $DEVICE_DIR/plugins/iconengines"

		if [ -f "$QT_PLUGIN_DIR/iconengines/libqsvgicon.so" ]; then
			echo "  推送 SVG 图标引擎..."
			adb push "$QT_PLUGIN_DIR/iconengines/libqsvgicon.so" "$DEVICE_DIR/plugins/iconengines/" 2>/dev/null || true
		fi
	fi

	# 推送虚拟键盘输入法插件（关键！）
	if [ -d "$QT_PLUGIN_DIR/platforminputcontexts" ]; then
		echo -e "${YELLOW}推送虚拟键盘插件...${NC}"
		adb shell "mkdir -p $DEVICE_DIR/plugins/platforminputcontexts"

		if [ -f "$QT_PLUGIN_DIR/platforminputcontexts/libqtvirtualkeyboardplugin.so" ]; then
			echo "  推送 Qt Virtual Keyboard 插件..."
			adb push "$QT_PLUGIN_DIR/platforminputcontexts/libqtvirtualkeyboardplugin.so" "$DEVICE_DIR/plugins/platforminputcontexts/"
			echo -e "${GREEN}✓ 虚拟键盘插件已推送${NC}"
		fi
	fi
else
	echo -e "${YELLOW}⊘ 跳过 Qt 平台插件推送${NC}"
fi

# 推送 QML 模块
echo -e "${YELLOW}推送 QML 模块...${NC}"
QT_QML_DIR="/usr/lib/arm-linux-gnueabihf/qt6/qml"

if [ -d "$QT_QML_DIR" ]; then
	adb shell "mkdir -p $DEVICE_DIR/qml_modules"

	# 推送必需的 QML 模块
	for module in QtQuick QtQml; do
		if [ -d "$QT_QML_DIR/$module" ]; then
			echo "  推送 $module 模块..."
			adb push "$QT_QML_DIR/$module" "$DEVICE_DIR/qml_modules/" 2>&1 | grep -E "(pushed|skipped)" || echo "  完成"
		fi
	done

	# 推送虚拟键盘模块
	if [ -d "$QT_QML_DIR/QtQuick/VirtualKeyboard" ]; then
		echo "  推送 QtQuick.VirtualKeyboard 模块..."
		adb shell "mkdir -p $DEVICE_DIR/qml_modules/QtQuick"
		adb push "$QT_QML_DIR/QtQuick/VirtualKeyboard" "$DEVICE_DIR/qml_modules/QtQuick/" 2>&1 | grep -E "(pushed|skipped)" || echo "  完成"
	fi

	# 推送 QtQuick.Layouts 模块
	if [ -d "$QT_QML_DIR/QtQuick/Layouts" ]; then
		echo "  推送 QtQuick.Layouts 模块..."
		adb shell "mkdir -p $DEVICE_DIR/qml_modules/QtQuick"
		adb push "$QT_QML_DIR/QtQuick/Layouts" "$DEVICE_DIR/qml_modules/QtQuick/" 2>&1 | grep -E "(pushed|skipped)" || echo "  完成"
	fi

	# 推送 Qt.labs.folderlistmodel 模块
	if [ -d "$QT_QML_DIR/Qt/labs/folderlistmodel" ]; then
		echo "  推送 Qt.labs.folderlistmodel 模块..."
		adb shell "mkdir -p $DEVICE_DIR/qml_modules/Qt/labs"
		adb push "$QT_QML_DIR/Qt/labs/folderlistmodel" "$DEVICE_DIR/qml_modules/Qt/labs/" 2>&1 | grep -E "(pushed|skipped)" || echo "  完成"
	fi

	# 推送 builtins 和 jsroot
	for file in builtins.qmltypes jsroot.qmltypes; do
		if [ -f "$QT_QML_DIR/$file" ]; then
			adb push "$QT_QML_DIR/$file" "$DEVICE_DIR/qml_modules/" 2>/dev/null || true
		fi
	done

	echo -e "${GREEN}✓ QML 模块已推送${NC}"
else
	echo -e "${YELLOW}⚠ 未找到 QML 模块目录${NC}"
fi

# 创建启动脚本
echo -e "${YELLOW}创建启动脚本...${NC}"
cat >/tmp/run-qt-rust-demo.sh <<'EOF'
#!/bin/sh
cd /userdata/local/tmp/qt-rust-demo
export LD_LIBRARY_PATH=/userdata/local/tmp/qt-rust-demo/lib:$LD_LIBRARY_PATH
export QT_PLUGIN_PATH=/userdata/local/tmp/qt-rust-demo/plugins
export QT_QPA_PLATFORM_PLUGIN_PATH=/userdata/local/tmp/qt-rust-demo/plugins
export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0
export QML2_IMPORT_PATH=/userdata/local/tmp/qt-rust-demo/qml_modules
export QT_QPA_FB_DISABLE_INPUT=0
export QT_IM_MODULE=qtvirtualkeyboard
export QT_VIRTUALKEYBOARD_DESKTOP_DISABLE=0
export FONTCONFIG_FILE=/userdata/local/tmp/qt-rust-demo/fonts/fonts.conf
export FONTCONFIG_PATH=/userdata/local/tmp/qt-rust-demo/fonts
# 将 Qt VirtualKeyboard 用户数据目录重定向到可写路径
# 避免 "Cannot create directory for user data /root/.config/qtvirtualkeyboard"
export XDG_CONFIG_HOME=/tmp/qt-app-config
mkdir -p /tmp/qt-app-config/qtvirtualkeyboard
./qt-rust-demo
EOF

adb push /tmp/run-qt-rust-demo.sh "$DEVICE_DIR/run.sh"
adb shell "chmod 755 $DEVICE_DIR/run.sh"

# 推送配置文件
echo -e "${YELLOW}推送配置文件...${NC}"
DEVICE_CONFIG="$(dirname "$0")/device-config.sh"
if [ -f "$DEVICE_CONFIG" ]; then
	adb push "$DEVICE_CONFIG" "$DEVICE_DIR/config.sh"
	adb shell "chmod +x $DEVICE_DIR/config.sh"
	echo -e "${GREEN}✓ 配置文件已推送${NC}"
fi

echo ""
echo -e "${GREEN}=== 部署完成! ===${NC}"
echo ""
echo "运行应用的方式："
echo ""
echo "方式 1: 使用便捷脚本（推荐）"
echo -e "${YELLOW}  ./run-on-device.sh${NC}"
echo ""
echo "方式 2: 直接在设备上运行"
echo -e "${YELLOW}  adb shell${NC}"
echo -e "${YELLOW}  cd $DEVICE_DIR${NC}"
echo -e "${YELLOW}  export LD_LIBRARY_PATH=./lib:\$LD_LIBRARY_PATH${NC}"
echo -e "${YELLOW}  export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0${NC}"
echo -e "${YELLOW}  export QT_QPA_PLATFORM_PLUGIN_PATH=./plugins${NC}"
echo -e "${YELLOW}  export FONTCONFIG_FILE=./fonts/fonts.conf${NC}"
echo -e "${YELLOW}  ./qt-rust-demo${NC}"
echo ""
echo "方式 3: 使用设备上的脚本"
echo -e "${YELLOW}  adb shell \"sh $DEVICE_DIR/run.sh\"${NC}"
echo ""
echo "查看日志："
echo -e "${YELLOW}  adb logcat | grep qt-rust-demo${NC}"

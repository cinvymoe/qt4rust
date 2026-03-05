#!/system/bin/sh
# Qt 应用设备配置文件
# 在设备上运行前可以根据需要调整这些环境变量

# ============ 基础路径配置 ============
export APP_DIR=/data/local/tmp/qt-rust-demo
export LD_LIBRARY_PATH=$APP_DIR/lib:$LD_LIBRARY_PATH

# ============ Qt 平台配置 ============
# 使用 Linux Framebuffer 直接输出到 /dev/fb0
export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0

# 其他可选的 linuxfb 参数：
# export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0:size=800x480
# export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0:size=1920x1080
# export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0:mmSize=800x480

# ============ Qt 插件路径 ============
export QT_PLUGIN_PATH=$APP_DIR/plugins
export QT_QPA_PLATFORM_PLUGIN_PATH=$APP_DIR/plugins
export QML2_IMPORT_PATH=$APP_DIR/qml

# ============ 输入设备配置 ============
# 启用输入设备（触摸屏、键盘等）
export QT_QPA_FB_DISABLE_INPUT=0

# 如果有触摸屏，可以指定触摸设备
# export QT_QPA_EVDEV_TOUCHSCREEN_PARAMETERS=/dev/input/event0

# 如果有鼠标
# export QT_QPA_EVDEV_MOUSE_PARAMETERS=/dev/input/event1

# 如果有键盘
# export QT_QPA_EVDEV_KEYBOARD_PARAMETERS=/dev/input/event2

# ============ 显示配置 ============
# 强制使用特定的屏幕分辨率（如果自动检测失败）
# export QT_QPA_FB_FORCE_FULLSCREEN=1

# DPI 设置（影响字体和 UI 缩放）
# export QT_FONT_DPI=96

# ============ 调试选项 ============
# 启用 Qt 调试输出
# export QT_DEBUG_PLUGINS=1
# export QT_LOGGING_RULES="qt.qpa.*=true"

# 显示 FPS
# export QT_QPA_EGLFS_DEBUG=1

# ============ 性能优化 ============
# 禁用垂直同步（可能提高性能但会有撕裂）
# export QT_QPA_EGLFS_SWAPINTERVAL=0

# 使用软件渲染（如果硬件加速有问题）
# export QT_QUICK_BACKEND=software

# ============ Framebuffer 信息检查 ============
# 在运行应用前，可以检查 framebuffer 信息：
# cat /sys/class/graphics/fb0/virtual_size
# cat /sys/class/graphics/fb0/bits_per_pixel
# fbset -fb /dev/fb0 -i

echo "Qt 环境配置完成"
echo "Framebuffer: /dev/fb0"
echo "库路径: $LD_LIBRARY_PATH"
echo "平台: $QT_QPA_PLATFORM"

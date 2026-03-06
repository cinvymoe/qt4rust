#!/bin/bash
# 安装 ARM32 Qt6 交叉编译所需的库

set -e

echo "正在添加 armhf 架构支持..."
dpkg --add-architecture armhf

echo "更新软件包列表..."
apt-get update

echo "安装 ARM Qt6 核心库..."
apt-get install -y \
    qt6-base-dev:armhf \
    qt6-declarative-dev:armhf \
    qt6-charts-dev:armhf \
    libqt6core6:armhf \
    libqt6gui6:armhf \
    libqt6qml6:armhf \
    libqt6quick6:armhf \
    libqt6network6:armhf \
    libqt6widgets6:armhf \
    libqt6charts6:armhf

echo "ARM Qt6 库安装完成！"
echo ""
echo "验证安装："
dpkg -l | grep "qt6.*armhf" | head -10

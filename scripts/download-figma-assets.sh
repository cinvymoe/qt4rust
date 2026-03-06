#!/bin/bash
# 下载 Figma 设计资源到本地

ASSETS_DIR="qml/assets/images"
mkdir -p "$ASSETS_DIR"

echo "开始下载 Figma 设计资源..."

# 下载图标和图片
curl -o "$ASSETS_DIR/canvas.png" "https://www.figma.com/api/mcp/asset/78957821-4c2e-48f4-92e3-18ef989f65cb"
curl -o "$ASSETS_DIR/icon-logo.png" "https://www.figma.com/api/mcp/asset/366fe571-f087-43d5-9e6e-f10205581d37"
curl -o "$ASSETS_DIR/icon-alert.png" "https://www.figma.com/api/mcp/asset/eb261eee-8148-4bf2-8577-f4b1ea95a42e"
curl -o "$ASSETS_DIR/icon-gauge.png" "https://www.figma.com/api/mcp/asset/ef47509c-89ae-4116-b124-f02a5b8b86c1"
curl -o "$ASSETS_DIR/icon-weight.png" "https://www.figma.com/api/mcp/asset/537a4768-08ee-476e-9f08-0a06ba5f188d"
curl -o "$ASSETS_DIR/icon-radius.png" "https://www.figma.com/api/mcp/asset/6d32271d-791c-4297-8888-2a36592076d1"
curl -o "$ASSETS_DIR/icon-angle.png" "https://www.figma.com/api/mcp/asset/1e8a7f07-499e-4eda-b834-b51296ffc5d7"
curl -o "$ASSETS_DIR/icon-home.png" "https://www.figma.com/api/mcp/asset/7904ffcc-0ebe-4935-aa37-30161b432b86"
curl -o "$ASSETS_DIR/icon-chart.png" "https://www.figma.com/api/mcp/asset/b6a84855-5522-4862-9d02-e32c14508a19"
curl -o "$ASSETS_DIR/icon-settings.png" "https://www.figma.com/api/mcp/asset/48656579-5f3a-46df-a8dd-ee3a8da32e0a"

echo "资源下载完成！"
echo "文件保存在: $ASSETS_DIR"

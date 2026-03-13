.PHONY: help build push push-qml push-no-plugins run stop clean deploy install-autostart

# 脚本目录
SCRIPTS_DIR := scripts

# 默认目标
help:
	@echo "Qt Rust Demo - Makefile"
	@echo ""
	@echo "可用命令:"
	@echo "  make build             - 编译 ARM 版本应用"
	@echo "  make push              - 推送应用和依赖到设备"
	@echo "  make push-qml          - 仅推送 QML 文件到设备"
	@echo "  make push-no-plugins   - 推送应用但不推送 Qt 插件和共享库"
	@echo "  make run               - 在设备上运行应用"
	@echo "  make stop              - 停止设备上的应用"
	@echo "  make deploy            - 编译并推送到设备"
	@echo "  make install-autostart - 安装自动启动脚本"
	@echo "  make clean             - 清理编译产物"
	@echo ""

# 编译 ARM 版本
build:
	@echo "=== 编译 ARM 版本 ==="
	cargo build --release --target armv7-unknown-linux-gnueabihf

# 推送到设备
push:
	@echo "=== 推送到设备 ==="
	@bash $(SCRIPTS_DIR)/deploy-to-device.sh

# 仅推送 QML 文件
push-qml:
	@echo "=== 推送 QML 文件到设备 ==="
	@adb push qml /data/local/tmp/qt-rust-demo/
	@echo "QML 文件推送完成"

# 推送到设备（不推送 Qt 平台插件和共享库）
push-no-plugins:build run
	@echo "=== 推送到设备（跳过 Qt 平台插件和共享库）==="
	@SKIP_LIBS=1 bash $(SCRIPTS_DIR)/deploy-to-device.sh

# 在设备上运行
run: stop
	@echo "=== 在设备上运行应用 ==="
	@echo "按 Ctrl+C 停止应用"
	@adb shell "sh /data/local/tmp/qt-rust-demo/run.sh"

# 停止设备上的应用
stop:
	@echo "=== 停止设备上的应用 ==="
	@adb shell "killall qt-rust-demo" || echo "应用未运行或已停止"

# 编译并部署
deploy: build push run
	@echo "=== 部署完成 ==="

# 安装自动启动
install-autostart:
	@echo "=== 安装自动启动脚本 ==="
	@bash $(SCRIPTS_DIR)/install-autostart.sh

# 清理
clean:
	@echo "=== 清理编译产物 ==="
	cargo clean
	rm -rf libs-to-deploy

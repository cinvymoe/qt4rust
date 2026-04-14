.PHONY: help build push push-qml push-config push-fonts push-no-plugins run stop clean deploy install-autostart push-no-plugins-with-build pull-db push-db pull-config

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
	@echo "  make push-config       - 仅推送配置文件到设备"
	@echo "  make push-fonts        - 收集并推送字体到设备"
	@echo "  make push-no-plugins   - 推送应用但不推送 Qt 插件和共享库"
	@echo "  make run               - 在设备上运行应用"
	@echo "  make stop              - 停止设备上的应用"
	@echo "  make deploy            - 编译并推送到设备"
	@echo "  make install-autostart - 安装自动启动脚本"
	@echo "  make pull-db           - 从设备拉取数据库文件到 db/ 文件夹"
	@echo "  make pull-config       - 从设备拉取配置文件到 dev/config/ 文件夹"
	@echo "  make push-db           - 推送本地 db/ 文件夹的数据库到设备"
	@echo "  make clean             - 清理编译产物"
	@echo ""

# 编译 ARM 版本
build:
	@echo "=== 编译 ARM 版本 ==="
	cargo build --release --target armv7-unknown-linux-gnueabihf -p qt-app

# 推送到设备
push:
	@echo "=== 推送到设备 ==="
	@bash $(SCRIPTS_DIR)/deploy-to-device.sh

# 仅推送 QML 文件
push-qml:
	@echo "=== 推送 QML 文件到设备 ==="
	@adb push qml /data/local/tmp/qt-rust-demo/
	@echo "QML 文件推送完成"

# 仅推送配置文件
push-config:
	@echo "=== 推送配置文件到设备 ==="
	@adb shell "mkdir -p /data/local/tmp/qt-rust-demo/config"
	@adb push config/. /data/local/tmp/qt-rust-demo/config/
	@echo "配置文件推送完成"

# 收集并推送字体
push-fonts:
	@echo "=== 收集并推送字体到设备 ==="
	@bash $(SCRIPTS_DIR)/collect-fonts.sh
	@adb shell "mkdir -p /data/local/tmp/qt-rust-demo/fonts"
	@adb push fonts/. /data/local/tmp/qt-rust-demo/fonts/
	@echo "字体推送完成"

# 推送到设备（不推送 Qt 平台插件和共享库）
push-no-plugins:
	@echo "=== 推送到设备（跳过 Qt 平台插件和共享库）==="
	@SKIP_LIBS=1 bash $(SCRIPTS_DIR)/deploy-to-device.sh

# 第一执编译，然后推送并运行
push-no-plugins-with-build: build push-no-plugins run
	@echo "=== 第一执编译、推送并运行完成 ==="

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

# 从设备拉取数据库到 db/ 文件夹
pull-db:
	@echo "=== 从设备拉取数据库到 db/ 文件夹 ==="
	@mkdir -p db
	@adb pull /data/local/tmp/qt-rust-demo/crane_data.db ./db/crane_data.db
	@echo "数据库已保存到: db/crane_data.db"

# 推送本地数据库到设备
push-db:
	@if [ ! -f db/crane_data.db ]; then \
		echo "错误: db/crane_data.db 不存在"; \
		echo "请先使用 'make pull-db' 拉取数据库，或手动创建数据库文件"; \
		exit 1; \
	fi
	@adb push db/crane_data.db /data/local/tmp/qt-rust-demo/crane_data.db
	@echo "数据库已推送到设备"

# 从设备拉取配置文件到 dev/config/ 文件夹
pull-config:
	@echo "=== 从设备拉取配置文件到 dev/config/ 文件夹 ==="
	@mkdir -p dev/config
	@adb pull /data/local/tmp/qt-rust-demo/config/ ./dev/config/
	@echo "配置文件已保存到: dev/config/"

# 清理
clean:
	@echo "=== 清理编译产物 ==="
	cargo clean
	rm -rf libs-to-deploy

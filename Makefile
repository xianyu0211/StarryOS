# StarryOS Makefile
# 完整的构建和部署管理

# 配置参数
TARGET = aarch64-unknown-none
CARGO = cargo
RUSTC = rustc
BUILD_MODE = release
KERNEL_IMAGE = kernel8.img
DEPLOY_DIR = deploy
IMAGE_DIR = $(DEPLOY_DIR)/image
CONFIG_FILE = deploy-config.toml

# 构建目标
.PHONY: all build clean deploy test bench doc

# 默认目标
all: build

# 构建整个系统
build: build-kernel build-drivers build-ai build-apps

# 构建内核
build-kernel:
	@echo "构建内核..."
	cd kernel && $(CARGO) build --$(BUILD_MODE)

# 构建驱动模块
build-drivers:
	@echo "构建驱动模块..."
	cd drivers && $(CARGO) build --$(BUILD_MODE) --features "audio environmental communication auxiliary npu"

# 构建AI模块
build-ai:
	@echo "构建AI模块..."
	cd ai && $(CARGO) build --$(BUILD_MODE) --features "yolo_v8 speech npu optimization"

# 构建应用程序
build-apps:
	@echo "构建应用程序..."
	cd apps && $(CARGO) build --$(BUILD_MODE)

# 清理构建文件
clean:
	@echo "清理构建文件..."
	$(CARGO) clean
	rm -rf $(DEPLOY_DIR)

# 运行测试
test: unit-test integration-test

# 单元测试
unit-test:
	@echo "运行单元测试..."
	$(CARGO) test --workspace

# 集成测试
integration-test:
	@echo "运行集成测试..."
	$(CARGO) test --package apps --test integration

# 性能测试
bench:
	@echo "运行性能测试..."
	$(CARGO) bench --workspace

# 生成文档
doc:
	@echo "生成文档..."
	$(CARGO) doc --workspace --no-deps

# 部署系统
deploy: build create-image
	@echo "系统部署完成"

# 创建部署镜像
create-image:
	@echo "创建部署镜像..."
	mkdir -p $(IMAGE_DIR)/{boot,bin,lib,config,etc}
	
	# 复制内核
	cp target/$(TARGET)/$(BUILD_MODE)/kernel $(IMAGE_DIR)/boot/$(KERNEL_IMAGE)
	
	# 复制应用程序
	cp target/$(TARGET)/$(BUILD_MODE)/apps $(IMAGE_DIR)/bin/starryos-app
	
	# 复制配置文件
	cp $(CONFIG_FILE) $(IMAGE_DIR)/config/
	cp scripts/deploy-voice-ai.sh $(IMAGE_DIR)/
	
	# 创建启动脚本
	echo '# StarryOS 启动脚本' > $(IMAGE_DIR)/boot/boot.scr
	echo 'setenv bootargs "console=ttyS2,1500000 root=/dev/mmcblk0p2 rw rootwait"' >> $(IMAGE_DIR)/boot/boot.scr
	echo 'load mmc 0:1 0x1000000 boot/$(KERNEL_IMAGE)' >> $(IMAGE_DIR)/boot/boot.scr
	echo 'booti 0x1000000 - 0x2000000' >> $(IMAGE_DIR)/boot/boot.scr
	
	@echo "部署镜像创建完成: $(IMAGE_DIR)"

# 快速部署到RK3588
rk3588-deploy: build create-image
	@echo "部署到RK3588设备..."
	@if [ -z "$(DEVICE)" ]; then \
		echo "请指定目标设备: make rk3588-deploy DEVICE=/dev/sdX"; \
		exit 1; \
	fi
	sudo ./scripts/deploy-voice-ai.sh $(DEVICE)

# 运行语音交互演示
voice-demo: build
	@echo "运行语音交互演示..."
	cd apps && $(CARGO) run --$(BUILD_MODE) --features "voice-interaction"

# 运行多模态融合演示
multimodal-demo: build
	@echo "运行多模态融合演示..."
	cd apps && $(CARGO) run --$(BUILD_MODE) --features "multimodal-fusion"

# 运行AI性能测试
ai-benchmark: build
	@echo "运行AI性能测试..."
	cd tests && $(CARGO) run --$(BUILD_MODE) --bin ai_bench

# 系统信息
info:
	@echo "=== StarryOS 系统信息 ==="
	@echo "目标架构: $(TARGET)"
	@echo "构建模式: $(BUILD_MODE)"
	@echo "内核镜像: $(KERNEL_IMAGE)"
	@echo "部署目录: $(DEPLOY_DIR)"
	@echo ""
	@echo "可用命令:"
	@echo "  make build        - 构建整个系统"
	@echo "  make test         - 运行所有测试"
	@echo "  make deploy       - 创建部署镜像"
	@echo "  make voice-demo   - 运行语音交互演示"
	@echo "  make clean        - 清理构建文件"

# 帮助信息
help:
	@echo "StarryOS 构建系统"
	@echo ""
	@echo "目标平台: RK3588 (AArch64)"
	@echo "功能特性:"
	@echo "  - 语音交互系统"
	@echo "  - 多模态AI融合"
	@echo "  - YOLO-v8目标识别"
	@echo "  - NPU硬件加速"
	@echo ""
	@echo "使用: make [目标]"
	@echo "运行 'make info' 查看详细信息"

# 默认目标
.DEFAULT_GOAL := help
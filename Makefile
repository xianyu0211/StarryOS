# StarryOS Makefile

# 工具链配置
TARGET = aarch64-unknown-none
RUST_TARGET = target/$(TARGET)/release/kernel
KERNEL = kernel/target/$(TARGET)/release/kernel

# 构建工具
CARGO = cargo
OBJCOPY = rust-objcopy
QEMU = qemu-system-aarch64

# 构建标志
BUILD_FLAGS = --target $(TARGET) --release

.PHONY: all kernel drivers ai apps clean flash boot test

all: kernel drivers ai apps

# 构建内核
kernel:
	@echo "构建内核..."
	cd kernel && $(CARGO) build $(BUILD_FLAGS)

# 构建驱动模块
drivers:
	@echo "构建驱动模块..."
	cd drivers && $(CARGO) build $(BUILD_FLAGS)

# 构建AI模块
ai:
	@echo "构建AI模块..."
	cd ai && $(CARGO) build $(BUILD_FLAGS)

# 构建应用
apps:
	@echo "构建应用模块..."
	cd apps && $(CARGO) build $(BUILD_FLAGS)

# 清理构建
clean:
	@echo "清理构建文件..."
	cd kernel && $(CARGO) clean
	cd drivers && $(CARGO) clean
	cd ai && $(CARGO) clean
	cd apps && $(CARGO) clean
	rm -f kernel8.img

# 生成内核镜像
kernel8.img: kernel
	@echo "生成内核镜像..."
	$(OBJCOPY) $(KERNEL) --strip-all -O binary kernel8.img

# 烧录到开发板
flash: kernel8.img
	@echo "烧录内核到开发板..."
	# 这里需要根据具体开发板配置烧录命令
	@echo "请配置具体的烧录工具和命令"

# 启动QEMU模拟器
boot: kernel8.img
	@echo "启动QEMU模拟器..."
	$(QEMU) -M virt -cpu cortex-a72 -smp 4 -m 2G \
		-kernel kernel8.img \
		-serial stdio \
		-device virtio-gpu-pci \
		-device virtio-net,netdev=net0 \
		-netdev user,id=net0

# 运行测试
test:
	@echo "运行单元测试..."
	cd kernel && $(CARGO) test
	cd drivers && $(CARGO) test
	cd ai && $(CARGO) test
	cd apps && $(CARGO) test

# 性能测试
bench:
	@echo "运行性能测试..."
	cd tests && $(CARGO) bench

# 格式化代码
fmt:
	$(CARGO) fmt --all

# 代码检查
check:
	$(CARGO) check --all-targets

# 交叉编译工具链安装
install-toolchain:
	@echo "安装交叉编译工具链..."
	rustup target add $(TARGET)
	rustup component add rust-src
	rustup component add llvm-tools-preview

help:
	@echo "可用命令:"
	@echo "  all        - 构建所有模块"
	@echo "  kernel     - 构建内核"
	@echo "  drivers    - 构建驱动模块"
	@echo "  ai         - 构建AI模块"
	@echo "  apps       - 构建应用模块"
	@echo "  clean      - 清理构建文件"
	@echo "  flash      - 烧录到开发板"
	@echo "  boot       - 启动QEMU模拟器"
	@echo "  test       - 运行测试"
	@echo "  bench      - 性能测试"
	@echo "  fmt        - 格式化代码"
	@echo "  check      - 代码检查"
	@echo "  install-toolchain - 安装工具链"
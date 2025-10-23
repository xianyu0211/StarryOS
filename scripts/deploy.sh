#!/bin/bash

set -e

echo "=== StarryOS 部署脚本 ==="

# 配置变量
TARGET_DEVICE=${1:-"orangepi-aipro"}
BUILD_TYPE=${2:-"release"}
KERNEL_IMAGE="kernel8.img"

echo "目标设备: $TARGET_DEVICE"
echo "构建类型: $BUILD_TYPE"

# 构建项目
echo "1. 构建项目..."
if [ "$BUILD_TYPE" = "debug" ]; then
    cargo build --target aarch64-unknown-none
else
    cargo build --target aarch64-unknown-none --release
fi

# 生成内核镜像
echo "2. 生成内核镜像..."
rust-objcopy target/aarch64-unknown-none/$BUILD_TYPE/kernel \
    --strip-all -O binary $KERNEL_IMAGE

# 检查镜像大小
if command -v stat &> /dev/null; then
    if stat -f%z $KERNEL_IMAGE &> /dev/null; then
        IMAGE_SIZE=$(stat -f%z $KERNEL_IMAGE)
    else
        IMAGE_SIZE=$(stat -c%s $KERNEL_IMAGE)
    fi
    echo "内核镜像大小: $((IMAGE_SIZE / 1024)) KB"
fi

# 设备特定的部署逻辑
case $TARGET_DEVICE in
    "orangepi-aipro")
        echo "3. 部署到香橙派AIpro..."
        # 这里添加具体的烧录命令
        echo "请手动将 $KERNEL_IMAGE 烧录到SD卡"
        echo "烧录命令参考: dd if=kernel8.img of=/dev/sdX bs=1M conv=fsync"
        ;;
    "qemu")
        echo "3. 启动QEMU模拟器..."
        make boot
        ;;
    *)
        echo "3. 未知设备: $TARGET_DEVICE"
        echo "请指定有效的目标设备"
        exit 1
        ;;
esac

echo "✅ 部署完成"
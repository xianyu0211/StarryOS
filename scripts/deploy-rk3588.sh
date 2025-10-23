#!/bin/bash

set -e

echo "=== StarryOS RK3588专用部署脚本 ==="

# 配置变量
BUILD_TYPE=${1:-"release"}
KERNEL_IMAGE="kernel8-rk3588.img"
BOARD_TYPE=${2:-"orangepi5"}

echo "目标设备: RK3588 ($BOARD_TYPE)"
echo "构建类型: $BUILD_TYPE"

# 检查依赖
echo "1. 检查依赖..."
if ! command -v cargo &> /dev/null; then
    echo "错误: 未找到 cargo 命令"
    exit 1
fi

if ! command -v rust-objcopy &> /dev/null; then
    echo "错误: 未找到 rust-objcopy 命令"
    echo "请安装: rustup component add llvm-tools-preview"
    exit 1
fi

# 构建RK3588专用内核
echo "2. 构建RK3588专用内核..."
if [ "$BUILD_TYPE" = "debug" ]; then
    cargo build --target aarch64-unknown-none --features rockchip-rk3588
else
    cargo build --target aarch64-unknown-none --release --features rockchip-rk3588
fi

# 生成内核镜像
echo "3. 生成RK3588内核镜像..."
rust-objcopy target/aarch64-unknown-none/$BUILD_TYPE/kernel \
    --strip-all -O binary $KERNEL_IMAGE

# 检查镜像大小
if command -v stat &> /dev/null; then
    if stat -f%z $KERNEL_IMAGE &> /dev/null; then
        IMAGE_SIZE=$(stat -f%z $KERNEL_IMAGE)
    else
        IMAGE_SIZE=$(stat -c%s $KERNEL_IMAGE)
    fi
    echo "RK3588内核镜像大小: $((IMAGE_SIZE / 1024)) KB"
fi

# 设备特定的部署逻辑
echo "4. 部署到RK3588开发板 ($BOARD_TYPE)..."

case $BOARD_TYPE in
    "orangepi5")
        echo "  目标板卡: 香橙派5"
        echo "  烧录命令: dd if=$KERNEL_IMAGE of=/dev/sdX bs=1M conv=fsync"
        echo "  注意: 请将 /dev/sdX 替换为实际的SD卡设备"
        ;;
    "rock5b")
        echo "  目标板卡: Radxa Rock 5B"
        echo "  烧录命令: dd if=$KERNEL_IMAGE of=/dev/sdX bs=1M conv=fsync"
        echo "  注意: 请将 /dev/sdX 替换为实际的SD卡设备"
        ;;
    "firefly")
        echo "  目标板卡: Firefly ROC-RK3588S-PC"
        echo "  烧录命令: dd if=$KERNEL_IMAGE of=/dev/sdX bs=1M conv=fsync"
        echo "  注意: 请将 /dev/sdX 替换为实际的SD卡设备"
        ;;
    *)
        echo "  未知板卡类型: $BOARD_TYPE"
        echo "  支持的板卡: orangepi5, rock5b, firefly"
        echo "  通用烧录命令: dd if=$KERNEL_IMAGE of=/dev/sdX bs=1M conv=fsync"
        ;;
esac

# 生成部署说明
echo ""
echo "=== RK3588部署说明 ==="
echo "1. 将 $KERNEL_IMAGE 烧录到SD卡:"
echo "   sudo dd if=$KERNEL_IMAGE of=/dev/sdX bs=1M conv=fsync"
echo ""
echo "2. 插入SD卡到RK3588开发板"
echo "3. 连接串口调试工具 (波特率: 115200)"
echo "4. 上电启动"
echo ""
echo "5. RK3588特性支持:"
echo "   - Cortex-A76/A55 big.LITTLE架构优化"
echo "   - 6TOPS NPU AI加速支持"
echo "   - Mali-G610 GPU驱动"
echo "   - 丰富外设接口支持"

echo "✅ RK3588部署准备完成"
echo "镜像文件: $KERNEL_IMAGE"
#!/bin/bash

echo "=== StarryOS 调试环境设置 ==="

# 检查依赖工具
echo "1. 检查依赖工具..."
check_tool() {
    if command -v $1 &> /dev/null; then
        echo "✅ $1 已安装"
    else
        echo "❌ $1 未安装"
        return 1
    fi
}

check_tool cargo
check_tool rustc
check_tool qemu-system-aarch64
check_tool aarch64-none-elf-gdb

# 安装交叉编译工具链
echo "2. 安装交叉编译工具链..."
rustup target add aarch64-unknown-none
rustup component add llvm-tools-preview

# 构建项目
echo "3. 构建项目..."
make install-toolchain
make all

echo "4. 生成调试符号..."
rust-objcopy target/aarch64-unknown-none/release/kernel --strip-debug -O binary kernel8.img

echo "✅ 调试环境设置完成"
echo "使用 'make boot' 启动QEMU"
echo "使用GDB连接: target remote :1234"
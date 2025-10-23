#!/bin/bash

# StarryOS 语音AI系统部署脚本
# 针对RK3588平台的完整部署流程

set -e

echo "=== StarryOS 语音AI系统部署开始 ==="

# 检查环境
check_environment() {
    echo "检查部署环境..."
    
    # 检查Rust工具链
    if ! command -v rustc &> /dev/null; then
        echo "错误: 未找到Rust编译器"
        exit 1
    fi
    
    # 检查交叉编译工具
    if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
        echo "警告: 未找到aarch64交叉编译工具链"
        echo "将使用本地编译（仅适用于ARM64设备）"
    fi
    
    # 检查必要的工具
    for tool in make git; do
        if ! command -v $tool &> /dev/null; then
            echo "错误: 未找到必要工具 $tool"
            exit 1
    fi
    done
    
    echo "环境检查通过"
}

# 配置构建参数
configure_build() {
    echo "配置构建参数..."
    
    # 设置目标架构
    export CARGO_TARGET="aarch64-unknown-none"
    
    # 设置优化级别
    export RUSTFLAGS="-C opt-level=s -C target-cpu=cortex-a76"
    
    # 设置链接器
    export RUST_LINKER="aarch64-linux-gnu-gcc"
    
    # 创建构建配置
    cat > .cargo/config.toml << EOF
[build]
target = "$CARGO_TARGET"

[target.$CARGO_TARGET]
linker = "$RUST_LINKER"
rustflags = ["$RUSTFLAGS"]

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
EOF
    
    echo "构建配置完成"
}

# 构建内核和驱动
build_system() {
    echo "构建系统组件..."
    
    # 清理之前的构建
    cargo clean
    
    # 构建内核
    echo "构建内核..."
    cd kernel
    cargo build --release
    cd ..
    
    # 构建驱动
    echo "构建驱动模块..."
    cd drivers
    cargo build --release --features "audio environmental communication auxiliary"
    cd ..
    
    # 构建AI模块
    echo "构建AI模块..."
    cd ai
    cargo build --release --features "yolo_v8 speech npu"
    cd ..
    
    # 构建应用
    echo "构建应用程序..."
    cd apps
    cargo build --release
    cd ..
    
    echo "系统构建完成"
}

# 生成部署镜像
create_deployment_image() {
    echo "生成部署镜像..."
    
    # 创建镜像目录
    mkdir -p deploy/image/{boot,lib,bin,config}
    
    # 复制内核镜像
    cp target/$CARGO_TARGET/release/kernel deploy/image/boot/kernel8.img
    
    # 复制应用程序
    cp target/$CARGO_TARGET/release/apps deploy/image/bin/starryos-app
    
    # 复制配置文件
    cp deploy-config.toml deploy/image/config/
    cp scripts/deploy-rk3588.sh deploy/image/
    
    # 创建启动脚本
    cat > deploy/image/boot/boot.scr << EOF
# StarryOS 启动脚本
setenv bootargs "console=ttyS2,1500000 root=/dev/mmcblk0p2 rw rootwait"
load mmc 0:1 0x1000000 boot/kernel8.img
booti 0x1000000 - 0x2000000
EOF
    
    # 创建文件系统结构
    cat > deploy/image/etc/fstab << EOF
# StarryOS 文件系统配置
/dev/mmcblk0p1 /boot vfat defaults 0 2
/dev/mmcblk0p2 / ext4 defaults 0 1
EOF
    
    echo "部署镜像生成完成"
}

# 优化系统性能
optimize_system() {
    echo "优化系统性能..."
    
    # 设置CPU性能模式
    echo "performance" > deploy/image/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
    
    # 配置NPU驱动
    cat > deploy/image/etc/modules-load.d/npu.conf << EOF
# 加载NPU驱动
rockchip_npu
EOF
    
    # 配置音频驱动
    cat > deploy/image/etc/modules-load.d/audio.conf << EOF
# 加载音频驱动
snd_soc_rockchip_i2s
snd_soc_es8316
EOF
    
    # 设置内存优化参数
    cat > deploy/image/etc/sysctl.d/99-starryos.conf << EOF
# StarryOS 内存优化
vm.swappiness=10
vm.dirty_ratio=15
vm.dirty_background_ratio=5
EOF
    
    echo "系统优化完成"
}

# 部署到目标设备
deploy_to_target() {
    local target_device="${1:-/dev/sdb}"
    
    echo "部署到目标设备: $target_device"
    
    # 检查目标设备
    if [ ! -b "$target_device" ]; then
        echo "错误: 目标设备 $target_device 不存在"
        exit 1
    fi
    
    # 确认部署
    read -p "将在 $target_device 上部署系统，确认继续? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "部署取消"
        exit 0
    fi
    
    # 卸载已挂载的分区
    for partition in ${target_device}*; do
        if mountpoint -q "$partition"; then
            sudo umount "$partition"
        fi
    done
    
    # 创建分区
    echo "创建分区..."
    sudo parted "$target_device" mklabel gpt
    sudo parted "$target_device" mkpart primary fat32 1MiB 256MiB
    sudo parted "$target_device" mkpart primary ext4 256MiB 100%
    sudo parted "$target_device" set 1 boot on
    
    # 格式化分区
    echo "格式化分区..."
    sudo mkfs.vfat -F 32 ${target_device}1
    sudo mkfs.ext4 ${target_device}2
    
    # 挂载分区
    sudo mkdir -p /mnt/starryos/{boot,root}
    sudo mount ${target_device}1 /mnt/starryos/boot
    sudo mount ${target_device}2 /mnt/starryos/root
    
    # 复制系统文件
    echo "复制系统文件..."
    sudo cp -r deploy/image/* /mnt/starryos/root/
    sudo cp deploy/image/boot/* /mnt/starryos/boot/
    
    # 安装引导程序
    echo "安装引导程序..."
    sudo dd if=deploy/u-boot.bin of="$target_device" bs=512 seek=64
    
    # 同步并卸载
    sync
    sudo umount /mnt/starryos/boot
    sudo umount /mnt/starryos/root
    
    echo "部署完成!"
}

# 运行系统测试
run_tests() {
    echo "运行系统测试..."
    
    # 单元测试
    echo "运行单元测试..."
    cargo test --workspace
    
    # 集成测试
    echo "运行集成测试..."
    cargo test --package apps --test integration
    
    # 性能测试
    echo "运行性能测试..."
    cargo bench --workspace
    
    echo "系统测试完成"
}

# 主部署流程
main() {
    local target_device="${1:-}"
    
    check_environment
    configure_build
    build_system
    run_tests
    create_deployment_image
    optimize_system
    
    if [ -n "$target_device" ]; then
        deploy_to_target "$target_device"
    else
        echo "部署镜像已生成到 deploy/image/ 目录"
        echo "使用以下命令手动部署:"
        echo "  sudo ./scripts/deploy-voice-ai.sh /dev/sdX"
    fi
    
    echo "=== StarryOS 语音AI系统部署完成 ==="
}

# 执行主函数
main "$@"
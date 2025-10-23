#!/bin/bash

# StarryOS RK3588部署脚本
# 将系统部署到RK3588开发板的完整流程

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查系统依赖..."
    
    local deps=("rustup" "cargo" "arm-none-eabi-gcc" "qemu-system-aarch64" "dd" "parted" "mkfs.vfat")
    local missing_deps=()
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            missing_deps+=("$dep")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "缺少以下依赖: ${missing_deps[*]}"
        log_info "请安装缺失的依赖后重试"
        exit 1
    fi
    
    # 检查Rust工具链
    if ! rustup show | grep -q "aarch64-unknown-none"; then
        log_info "安装aarch64 Rust工具链..."
        rustup target add aarch64-unknown-none
    fi
    
    log_success "所有依赖检查通过"
}

# 构建系统
build_system() {
    log_info "开始构建StarryOS系统..."
    
    # 清理之前的构建
    log_info "清理构建目录..."
    cargo clean
    
    # 构建内核
    log_info "构建内核..."
    cd kernel
    cargo build --target aarch64-unknown-none --release
    cd ..
    
    # 构建驱动
    log_info "构建驱动模块..."
    cd drivers
    cargo build --target aarch64-unknown-none --release
    cd ..
    
    # 构建AI模块
    log_info "构建AI模块..."
    cd ai
    cargo build --target aarch64-unknown-none --release
    cd ..
    
    # 构建应用程序
    log_info "构建应用程序..."
    cd apps
    cargo build --target aarch64-unknown-none --release
    cd ..
    
    # 生成内核镜像
    log_info "生成内核镜像..."
    arm-none-eabi-objcopy -O binary \
        target/aarch64-unknown-none/release/kernel \
        kernel8.img
    
    log_success "系统构建完成"
}

# 运行系统测试
run_tests() {
    log_info "运行系统测试..."
    
    # 单元测试
    log_info "运行单元测试..."
    cargo test --workspace
    
    # 集成测试
    log_info "运行集成测试..."
    cargo test --test integration
    
    # QEMU模拟测试
    log_info "在QEMU中测试系统..."
    qemu-system-aarch64 \
        -machine virt \
        -cpu cortex-a72 \
        -smp 4 \
        -m 2G \
        -kernel kernel8.img \
        -nographic \
        -serial mon:stdio \
        -append "console=ttyAMA0" \
        -d guest_errors \
        &
    
    local QEMU_PID=$!
    sleep 10
    
    # 检查系统是否正常启动
    if ps -p $QEMU_PID > /dev/null; then
        log_success "QEMU测试通过"
        kill $QEMU_PID
    else
        log_error "QEMU测试失败"
        exit 1
    fi
    
    log_success "所有测试通过"
}

# 准备部署镜像
prepare_deployment_image() {
    log_info "准备部署镜像..."
    
    # 创建临时目录
    local temp_dir=$(mktemp -d)
    
    # 创建磁盘镜像
    local image_size="256M"
    local image_file="starryos-rk3588.img"
    
    log_info "创建磁盘镜像: $image_file ($image_size)"
    dd if=/dev/zero of="$image_file" bs=1M count=256 status=progress
    
    # 分区
    log_info "分区磁盘镜像..."
    parted "$image_file" mklabel msdos
    parted "$image_file" mkpart primary fat32 1MiB 100%
    parted "$image_file" set 1 boot on
    
    # 格式化分区
    log_info "格式化分区..."
    local loop_device=$(sudo losetup --find --show --partscan "$image_file")
    sudo mkfs.vfat -F32 "${loop_device}p1"
    
    # 挂载分区
    sudo mount "${loop_device}p1" "$temp_dir"
    
    # 复制文件
    log_info "复制系统文件..."
    sudo cp kernel8.img "$temp_dir/"
    sudo cp deploy-config.toml "$temp_dir/"
    sudo cp README.md "$temp_dir/"
    
    # 创建启动脚本
    log_info "创建启动脚本..."
    cat > "$temp_dir/boot.scr" << 'EOF'
# StarryOS启动脚本
setenv bootargs "console=ttyS2,1500000 root=/dev/mmcblk0p2 rootwait rw"
fatload mmc 0:1 0x80000 kernel8.img
bootm 0x80000
EOF
    
    # 卸载分区
    sudo umount "$temp_dir"
    sudo losetup -d "$loop_device"
    
    # 清理临时目录
    rm -rf "$temp_dir"
    
    log_success "部署镜像准备完成: $image_file"
}

# 部署到开发板
deploy_to_board() {
    local device=$1
    
    if [ -z "$device" ]; then
        log_error "请指定目标设备，例如: /dev/sdX"
        log_info "可用设备:"
        lsblk -d -o NAME,SIZE,MODEL
        exit 1
    fi
    
    # 确认设备存在
    if [ ! -e "$device" ]; then
        log_error "设备 $device 不存在"
        exit 1
    fi
    
    # 警告用户
    log_warning "这将擦除设备 $device 上的所有数据！"
    read -p "确认继续? (y/N): " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "操作已取消"
        exit 0
    fi
    
    # 部署镜像
    log_info "开始部署到设备 $device..."
    
    # 写入镜像
    sudo dd if=starryos-rk3588.img of="$device" bs=4M status=progress
    
    # 同步文件系统
    sudo sync
    
    log_success "系统已成功部署到 $device"
}

# 验证部署
verify_deployment() {
    log_info "验证部署..."
    
    # 检查镜像文件
    if [ ! -f "starryos-rk3588.img" ]; then
        log_error "部署镜像不存在"
        exit 1
    fi
    
    # 检查镜像大小
    local image_size=$(stat -f%z "starryos-rk3588.img" 2>/dev/null || stat -c%s "starryos-rk3588.img")
    if [ "$image_size" -lt 1000000 ]; then
        log_error "镜像文件大小异常"
        exit 1
    fi
    
    # 检查内核镜像
    if [ ! -f "kernel8.img" ]; then
        log_error "内核镜像不存在"
        exit 1
    fi
    
    log_success "部署验证通过"
}

# 显示帮助信息
show_help() {
    cat << EOF
StarryOS RK3588部署脚本

用法: $0 [选项]

选项:
    -c, --check         检查系统依赖
    -b, --build         构建系统
    -t, --test          运行测试
    -p, --prepare       准备部署镜像
    -d, --deploy DEVICE 部署到指定设备
    -a, --all           执行完整部署流程
    -h, --help         显示此帮助信息

示例:
    $0 --check           # 检查依赖
    $0 --build           # 构建系统
    $0 --all             # 完整部署流程
    $0 --deploy /dev/sdb # 部署到SD卡

硬件要求:
    - RK3588开发板（Orange Pi 5/5B/5 Plus等）
    - 至少2GB内存
    - 8GB以上存储空间
    - 网络连接（用于下载依赖）

软件要求:
    - Rust工具链（nightly）
    - aarch64交叉编译工具链
    - QEMU模拟器
    - 必要的系统工具
EOF
}

# 主函数
main() {
    local action=""
    local device=""
    
    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -c|--check)
                action="check"
                shift
                ;;
            -b|--build)
                action="build"
                shift
                ;;
            -t|--test)
                action="test"
                shift
                ;;
            -p|--prepare)
                action="prepare"
                shift
                ;;
            -d|--deploy)
                action="deploy"
                device="$2"
                shift 2
                ;;
            -a|--all)
                action="all"
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                log_error "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # 默认执行完整流程
    if [ -z "$action" ]; then
        action="all"
    fi
    
    # 执行相应操作
    case "$action" in
        "check")
            check_dependencies
            ;;
        "build")
            check_dependencies
            build_system
            ;;
        "test")
            check_dependencies
            build_system
            run_tests
            ;;
        "prepare")
            check_dependencies
            build_system
            run_tests
            prepare_deployment_image
            ;;
        "deploy")
            check_dependencies
            build_system
            run_tests
            prepare_deployment_image
            verify_deployment
            deploy_to_board "$device"
            ;;
        "all")
            check_dependencies
            build_system
            run_tests
            prepare_deployment_image
            verify_deployment
            log_success "完整部署流程完成"
            log_info "请使用以下命令部署到开发板:"
            log_info "  sudo $0 --deploy /dev/your_device"
            ;;
    esac
}

# 脚本入口
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
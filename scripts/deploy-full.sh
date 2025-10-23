#!/bin/bash

# StarryOS 完整部署脚本
# 支持QEMU模拟器和实际硬件部署

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
    
    local deps=("rustc" "cargo" "qemu-system-aarch64" "aarch64-none-elf-gdb")
    local missing=()
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            missing+=("$dep")
        fi
    done
    
    if [ ${#missing[@]} -ne 0 ]; then
        log_error "缺少依赖: ${missing[*]}"
        log_info "请运行: ./scripts/install-dependencies.sh"
        exit 1
    fi
    
    log_success "所有依赖已安装"
}

# 安装工具链
install_toolchain() {
    log_info "安装Rust工具链..."
    
    rustup target add aarch64-unknown-none
    rustup component add rust-src
    rustup component add llvm-tools-preview
    
    log_success "Rust工具链安装完成"
}

# 构建项目
build_project() {
    local build_type=${1:-"release"}
    
    log_info "构建项目 ($build_type)..."
    
    if [ "$build_type" = "debug" ]; then
        cargo build --target aarch64-unknown-none
    else
        cargo build --target aarch64-unknown-none --release
    fi
    
    # 生成内核镜像
    log_info "生成内核镜像..."
    rust-objcopy target/aarch64-unknown-none/$build_type/kernel \
        --strip-all -O binary kernel8.img
    
    local image_size=$(stat -f%z kernel8.img 2>/dev/null || stat -c%s kernel8.img)
    log_success "内核镜像生成完成 (大小: $((image_size / 1024)) KB)"
}

# 运行QEMU测试
run_qemu() {
    local debug=${1:-false}
    
    log_info "启动QEMU模拟器..."
    
    local qemu_args=(
        "-M" "virt"
        "-cpu" "cortex-a72"
        "-smp" "4"
        "-m" "2G"
        "-kernel" "kernel8.img"
        "-serial" "stdio"
        "-device" "virtio-gpu-pci"
        "-netdev" "user,id=net0"
        "-device" "virtio-net-device,netdev=net0"
    )
    
    if [ "$debug" = true ]; then
        qemu_args+=("-S" "-s")
        log_info "QEMU调试模式已启动 (GDB端口: 1234)"
    fi
    
    qemu-system-aarch64 "${qemu_args[@]}"
}

# 运行单元测试
run_tests() {
    log_info "运行单元测试..."
    
    cargo test --workspace
    
    log_success "所有测试通过"
}

# 运行性能测试
run_benchmarks() {
    log_info "运行性能基准测试..."
    
    cargo bench --workspace
    
    log_success "性能测试完成"
}

# 部署到实际硬件
deploy_hardware() {
    local target=$1
    
    log_info "部署到硬件: $target"
    
    case $target in
        "orangepi-aipro")
            log_warning "请手动将kernel8.img烧录到香橙派AIpro的SD卡"
            log_info "烧录命令: dd if=kernel8.img of=/dev/sdX bs=1M status=progress"
            ;;
        "raspberry-pi-4")
            log_warning "请手动将kernel8.img烧录到树莓派4的SD卡"
            log_info "烧录命令: dd if=kernel8.img of=/dev/sdX bs=1M status=progress"
            ;;
        *)
            log_error "不支持的硬件平台: $target"
            exit 1
            ;;
    esac
}

# 显示帮助信息
show_help() {
    echo "StarryOS 部署脚本"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  build [debug|release]    构建项目 (默认: release)"
    echo "  test                     运行所有测试"
    echo "  bench                    运行性能基准测试"
    echo "  qemu [debug]             在QEMU中运行 (可选调试模式)"
    echo "  deploy <target>         部署到硬件 (orangepi-aipro|raspberry-pi-4)"
    echo "  full                     完整部署流程"
    echo "  help                     显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 build debug          构建调试版本"
    echo "  $0 qemu debug           在QEMU调试模式中运行"
    echo "  $0 deploy orangepi-aipro 部署到香橙派AIpro"
    echo "  $0 full                  运行完整部署流程"
}

# 主函数
main() {
    local command=${1:-"help"}
    local arg=${2:-""}
    
    case $command in
        "build")
            check_dependencies
            install_toolchain
            build_project "$arg"
            ;;
        "test")
            run_tests
            ;;
        "bench")
            run_benchmarks
            ;;
        "qemu")
            check_dependencies
            install_toolchain
            build_project "release"
            run_qemu "$arg"
            ;;
        "deploy")
            check_dependencies
            install_toolchain
            build_project "release"
            deploy_hardware "$arg"
            ;;
        "full")
            check_dependencies
            install_toolchain
            build_project "release"
            run_tests
            run_benchmarks
            log_success "完整部署流程完成"
            ;;
        "help"|""|"-h"|"--help")
            show_help
            ;;
        *)
            log_error "未知命令: $command"
            show_help
            exit 1
            ;;
    esac
}

# 脚本入口
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
#!/bin/bash

# StarryOS 语音AI系统部署脚本
# 针对RK3588平台的完整部署流程
# 版本: 2.0.0
# 作者: StarryOS Team

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

# 版本信息
VERSION="2.0.0"
AUTHOR="StarryOS Team"
DESCRIPTION="RK3588嵌入式AIoT操作系统部署脚本"

echo -e "${BLUE}=== StarryOS 语音AI系统部署开始 ===${NC}"
echo "版本: $VERSION"
echo "作者: $AUTHOR"
echo "描述: $DESCRIPTION"
echo ""

# 显示帮助信息
show_help() {
    echo "使用: $0 [选项] [目标设备]

选项:
    -h, --help          显示此帮助信息
    -v, --version       显示版本信息
    -c, --clean         清理构建文件
    -t, --test-only     仅运行测试，不部署
    -d, --dry-run       模拟运行，不实际执行
    -f, --force         强制部署，跳过确认

示例:
    $0 /dev/sdb         部署到指定设备
    $0 --test-only      仅运行测试
    $0 --clean          清理构建文件
    "
}

# 参数解析
parse_arguments() {
    local target_device=""
    local clean_build=false
    local test_only=false
    local dry_run=false
    local force_deploy=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--version)
                echo "StarryOS 部署脚本 v$VERSION"
                exit 0
                ;;
            -c|--clean)
                clean_build=true
                shift
                ;;
            -t|--test-only)
                test_only=true
                shift
                ;;
            -d|--dry-run)
                dry_run=true
                shift
                ;;
            -f|--force)
                force_deploy=true
                shift
                ;;
            -*)
                log_error "未知选项: $1"
                show_help
                exit 1
                ;;
            *)
                if [[ -n "$target_device" ]]; then
                    log_error "只能指定一个目标设备"
                    show_help
                    exit 1
                fi
                target_device="$1"
                shift
                ;;
        esac
    done
    
    # 设置全局变量
    TARGET_DEVICE="$target_device"
    CLEAN_BUILD="$clean_build"
    TEST_ONLY="$test_only"
    DRY_RUN="$dry_run"
    FORCE_DEPLOY="$force_deploy"
}

# 检查环境
check_environment() {
    log_info "检查部署环境..."
    
    # 检查Rust工具链
    if ! command -v rustc &> /dev/null; then
        log_error "未找到Rust编译器"
        log_info "请安装Rust: https://rustup.rs"
        exit 1
    fi
    
    # 检查Rust版本
    local rust_version=$(rustc --version | cut -d' ' -f2)
    log_info "Rust版本: $rust_version"
    
    # 检查交叉编译工具
    if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
        log_warning "未找到aarch64交叉编译工具链"
        log_info "将使用本地编译（仅适用于ARM64设备）"
        
        # 检查是否在ARM64设备上
        if [[ $(uname -m) != "aarch64" ]]; then
            log_error "当前系统不是ARM64架构，需要交叉编译工具链"
            log_info "请安装交叉编译工具:"
            log_info "Ubuntu/Debian: sudo apt install gcc-aarch64-linux-gnu"
            log_info "macOS: brew install aarch64-unknown-linux-gnu"
            exit 1
        fi
    else
        log_success "交叉编译工具链已安装"
    fi
    
    # 检查必要的工具
    local required_tools=("make" "git" "parted" "mkfs.vfat" "mkfs.ext4")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log_error "未找到必要工具: $tool"
            exit 1
        fi
    done
    
    log_success "环境检查通过"
}

# 配置构建参数
configure_build() {
    log_info "配置构建参数..."
    
    # 设置目标架构
    export CARGO_TARGET="aarch64-unknown-none"
    
    # 设置优化级别
    export RUSTFLAGS="-C opt-level=s -C target-cpu=cortex-a76 -C link-arg=-Tlinker.ld"
    
    # 设置链接器
    if command -v aarch64-linux-gnu-gcc &> /dev/null; then
        export RUST_LINKER="aarch64-linux-gnu-gcc"
    else
        export RUST_LINKER="gcc"
    fi
    
    # 创建构建配置目录
    mkdir -p .cargo
    
    # 创建构建配置
    cat > .cargo/config.toml << EOF
[build]
target = "$CARGO_TARGET"

[target.$CARGO_TARGET]
linker = "$RUST_LINKER"
rustflags = ["$RUSTFLAGS"]

[target.aarch64-unknown-linux-gnu]
linker = "$RUST_LINKER"

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"

[profile.dev]
opt-level = 0
panic = "abort"
EOF
    
    # 创建链接器脚本
    cat > linker.ld << EOF
/* StarryOS 链接器脚本 */
ENTRY(_start)

SECTIONS
{
    . = 0x80000;
    
    .text : {
        *(.text.boot)
        *(.text .text.*)
    }
    
    .rodata : {
        *(.rodata .rodata.*)
    }
    
    .data : {
        *(.data .data.*)
    }
    
    .bss : {
        *(.bss .bss.*)
    }
    
    /DISCARD/ : {
        *(.comment)
        *(.gnu*)
        *(.note*)
    }
}
EOF
    
    log_success "构建配置完成"
}

# 清理构建文件
clean_build() {
    log_info "清理构建文件..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "模拟清理: cargo clean"
        return
    fi
    
    cargo clean
    rm -rf deploy .cargo/config.toml linker.ld
    
    log_success "构建文件清理完成"
}

# 构建内核和驱动
build_system() {
    log_info "构建系统组件..."
    
    # 检查是否需要清理
    if [[ "$CLEAN_BUILD" == "true" ]]; then
        clean_build
    fi
    
    # 检查是否在模拟运行
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "模拟构建: cargo build --release"
        return
    fi
    
    # 使用工作空间构建
    log_info "构建内核..."
    if ! cargo build --release --package kernel; then
        log_error "内核构建失败"
        exit 1
    fi
    
    log_info "构建驱动模块..."
    if ! cargo build --release --package drivers --features "audio environmental communication auxiliary"; then
        log_error "驱动模块构建失败"
        exit 1
    fi
    
    log_info "构建AI模块..."
    if ! cargo build --release --package ai --features "yolo_v8 speech npu"; then
        log_error "AI模块构建失败"
        exit 1
    fi
    
    log_info "构建应用程序..."
    if ! cargo build --release --package apps; then
        log_error "应用程序构建失败"
        exit 1
    fi
    
    # 验证构建结果
    local build_target="target/$CARGO_TARGET/release"
    if [[ ! -f "$build_target/kernel" ]]; then
        log_error "内核构建文件不存在"
        exit 1
    fi
    
    if [[ ! -f "$build_target/apps" ]]; then
        log_error "应用程序构建文件不存在"
        exit 1
    fi
    
    log_success "系统构建完成"
}

# 生成部署镜像
create_deployment_image() {
    log_info "生成部署镜像..."
    
    # 检查是否在模拟运行
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "模拟创建部署镜像"
        return
    fi
    
    # 创建镜像目录结构
    local image_dir="deploy/image"
    mkdir -p "$image_dir/{boot,lib,bin,config,etc,usr/lib,var/log}"
    
    # 复制内核镜像
    local build_target="target/$CARGO_TARGET/release"
    if [[ ! -f "$build_target/kernel" ]]; then
        log_error "内核文件不存在: $build_target/kernel"
        exit 1
    fi
    
    cp "$build_target/kernel" "$image_dir/boot/kernel8.img"
    
    # 复制应用程序
    if [[ ! -f "$build_target/apps" ]]; then
        log_error "应用程序文件不存在: $build_target/apps"
        exit 1
    fi
    
    cp "$build_target/apps" "$image_dir/bin/starryos-app"
    chmod +x "$image_dir/bin/starryos-app"
    
    # 复制配置文件
    if [[ -f "deploy-config.toml" ]]; then
        cp deploy-config.toml "$image_dir/config/"
    else
        log_warning "配置文件 deploy-config.toml 不存在"
    fi
    
    # 创建启动脚本
    cat > "$image_dir/boot/boot.scr" << 'EOF'
# StarryOS 启动脚本
setenv bootargs "console=ttyS2,1500000 root=/dev/mmcblk0p2 rw rootwait earlycon"
setenv kernel_addr_r 0x1000000
setenv fdt_addr_r 0x2000000

# 加载内核
load mmc 0:1 ${kernel_addr_r} boot/kernel8.img

# 启动内核
booti ${kernel_addr_r} - ${fdt_addr_r}
EOF
    
    # 创建文件系统配置
    cat > "$image_dir/etc/fstab" << 'EOF'
# StarryOS 文件系统配置
/dev/mmcblk0p1 /boot vfat defaults 0 2
/dev/mmcblk0p2 / ext4 defaults,noatime 0 1
tmpfs /tmp tmpfs defaults,size=64M 0 0
EOF
    
    # 创建系统服务配置
    cat > "$image_dir/etc/systemd/system/starryos.service" << 'EOF'
[Unit]
Description=StarryOS AIoT System
After=network.target

[Service]
Type=simple
ExecStart=/bin/starryos-app
Restart=always
RestartSec=5
User=root

[Install]
WantedBy=multi-user.target
EOF
    
    # 创建网络配置
    cat > "$image_dir/etc/network/interfaces" << 'EOF'
# StarryOS 网络配置
auto lo
iface lo inet loopback

auto eth0
iface eth0 inet dhcp

# WiFi配置（可选）
#auto wlan0
#iface wlan0 inet dhcp
#wpa-ssid "YourWiFi"
#wpa-psk "YourPassword"
EOF
    
    # 创建版本信息文件
    cat > "$image_dir/etc/starryos-version" << EOF
StarryOS v$VERSION
Build Date: $(date)
Target: $CARGO_TARGET
Kernel: $(uname -r)
EOF
    
    log_success "部署镜像生成完成: $image_dir"
}

# 优化系统性能
optimize_system() {
    log_info "优化系统性能..."
    
    # 检查是否在模拟运行
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "模拟系统优化"
        return
    fi
    
    local image_dir="deploy/image"
    
    # 创建性能优化配置目录
    mkdir -p "$image_dir/etc/modules-load.d" "$image_dir/etc/sysctl.d"
    
    # 设置CPU性能模式
    cat > "$image_dir/etc/rc.local" << 'EOF'
#!/bin/bash
# StarryOS 启动优化脚本

# 设置CPU性能模式
echo performance > /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor

# 启用所有CPU核心
for cpu in /sys/devices/system/cpu/cpu[1-7]; do
    echo 1 > "$cpu/online" 2>/dev/null || true
done

# 设置GPU性能模式
echo performance > /sys/class/misc/mali0/device/devfreq/ff9a0000.gpu/governor 2>/dev/null || true

# 设置NPU性能模式
echo performance > /sys/class/misc/rknpu/devfreq/ffbc0000.npu/governor 2>/dev/null || true

exit 0
EOF
    chmod +x "$image_dir/etc/rc.local"
    
    # 配置NPU驱动
    cat > "$image_dir/etc/modules-load.d/npu.conf" << 'EOF'
# 加载NPU驱动
rockchip_npu
EOF
    
    # 配置音频驱动
    cat > "$image_dir/etc/modules-load.d/audio.conf" << 'EOF'
# 加载音频驱动
snd_soc_rockchip_i2s
snd_soc_es8316
snd_soc_rk817
EOF
    
    # 配置视频驱动
    cat > "$image_dir/etc/modules-load.d/video.conf" << 'EOF'
# 加载视频驱动
rockchip_rga
rockchip_iep
rockchip_vpu
EOF
    
    # 设置内存优化参数
    cat > "$image_dir/etc/sysctl.d/99-starryos.conf" << 'EOF'
# StarryOS 内存优化
vm.swappiness=10
vm.dirty_ratio=15
vm.dirty_background_ratio=5
vm.vfs_cache_pressure=50
vm.dirty_writeback_centisecs=1500
vm.dirty_expire_centisecs=3000

# 网络优化
net.core.rmem_max=16777216
net.core.wmem_max=16777216
net.core.rmem_default=65536
net.core.wmem_default=65536
net.ipv4.tcp_rmem=4096 87380 16777216
net.ipv4.tcp_wmem=4096 65536 16777216
EOF
    
    # 创建系统服务优化
    cat > "$image_dir/etc/systemd/system.conf" << 'EOF'
[Manager]
# 系统服务优化
DefaultTimeoutStartSec=30s
DefaultTimeoutStopSec=15s
DefaultRestartSec=100ms
EOF
    
    log_success "系统优化完成"
}

# 部署到目标设备
deploy_to_target() {
    local target_device="${1:-/dev/sdb}"
    
    log_info "部署到目标设备: $target_device"
    
    # 检查目标设备
    if [ ! -b "$target_device" ]; then
        log_error "目标设备 $target_device 不存在"
        log_info "可用的存储设备:"
        lsblk -d -o NAME,SIZE,TYPE,MOUNTPOINT | grep -v "^NAME"
        exit 1
    fi
    
    # 显示设备信息
    local device_info=$(lsblk -d -o NAME,SIZE,TYPE,MOUNTPOINT "$target_device" 2>/dev/null || true)
    if [[ -n "$device_info" ]]; then
        log_info "设备信息:"
        echo "$device_info"
    fi
    
    # 确认部署（除非强制模式）
    if [[ "$FORCE_DEPLOY" != "true" ]]; then
        echo
        read -p "将在 $target_device 上部署系统，确认继续? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "部署取消"
            exit 0
        fi
    fi
    
    # 检查是否在模拟运行
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "模拟部署到设备: $target_device"
        return
    fi
    
    # 卸载已挂载的分区
    log_info "卸载已挂载的分区..."
    for partition in ${target_device}*; do
        if mountpoint -q "$partition" 2>/dev/null; then
            log_info "卸载分区: $partition"
            sudo umount "$partition" 2>/dev/null || true
        fi
    done
    
    # 创建分区表
    log_info "创建分区表..."
    if ! sudo parted "$target_device" mklabel gpt --script; then
        log_error "创建分区表失败"
        exit 1
    fi
    
    # 创建分区
    log_info "创建分区..."
    sudo parted "$target_device" mkpart primary fat32 1MiB 256MiB --script
    sudo parted "$target_device" mkpart primary ext4 256MiB 100% --script
    sudo parted "$target_device" set 1 boot on --script
    
    # 等待分区创建完成
    sleep 2
    
    # 格式化分区
    log_info "格式化分区..."
    if ! sudo mkfs.vfat -F 32 ${target_device}1; then
        log_error "格式化FAT32分区失败"
        exit 1
    fi
    
    if ! sudo mkfs.ext4 -F ${target_device}2; then
        log_error "格式化EXT4分区失败"
        exit 1
    fi
    
    # 挂载分区
    log_info "挂载分区..."
    sudo mkdir -p /mnt/starryos/{boot,root}
    
    if ! sudo mount ${target_device}1 /mnt/starryos/boot; then
        log_error "挂载boot分区失败"
        exit 1
    fi
    
    if ! sudo mount ${target_device}2 /mnt/starryos/root; then
        log_error "挂载root分区失败"
        sudo umount /mnt/starryos/boot 2>/dev/null || true
        exit 1
    fi
    
    # 复制系统文件
    log_info "复制系统文件..."
    if ! sudo cp -r deploy/image/* /mnt/starryos/root/; then
        log_error "复制系统文件失败"
        sudo umount /mnt/starryos/boot /mnt/starryos/root
        exit 1
    fi
    
    if ! sudo cp deploy/image/boot/* /mnt/starryos/boot/; then
        log_error "复制启动文件失败"
        sudo umount /mnt/starryos/boot /mnt/starryos/root
        exit 1
    fi
    
    # 安装引导程序（如果有U-Boot镜像）
    if [[ -f "deploy/u-boot.bin" ]]; then
        log_info "安装引导程序..."
        if ! sudo dd if=deploy/u-boot.bin of="$target_device" bs=512 seek=64; then
            log_warning "安装引导程序失败，系统可能无法启动"
        fi
    else
        log_warning "未找到U-Boot镜像，系统需要手动安装引导程序"
    fi
    
    # 同步并卸载
    log_info "同步文件系统..."
    sync
    
    log_info "卸载分区..."
    sudo umount /mnt/starryos/boot
    sudo umount /mnt/starryos/root
    
    # 清理挂载点
    sudo rmdir /mnt/starryos/boot /mnt/starryos/root /mnt/starryos 2>/dev/null || true
    
    log_success "部署完成!"
    
    # 显示部署信息
    echo
    log_info "部署摘要:"
    echo "  - 目标设备: $target_device"
    echo "  - Boot分区: ${target_device}1 (FAT32)"
    echo "  - Root分区: ${target_device}2 (EXT4)"
    echo "  - 内核版本: StarryOS v$VERSION"
    echo "  - 构建时间: $(date)"
    echo
    log_info "使用说明:"
    echo "  1. 将设备插入RK3588开发板"
    echo "  2. 设置启动顺序为从存储设备启动"
    echo "  3. 系统将自动启动"
    echo "  4. 默认串口: ttyS2, 波特率: 1500000"
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
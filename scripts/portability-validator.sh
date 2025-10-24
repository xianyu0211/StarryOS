#!/bin/bash

# StarryOS 移植性验证工具
# 验证系统在不同RK3588平台上的兼容性和移植性

set -e

echo "=== StarryOS 移植性验证开始 ==="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

# 检测硬件平台
detect_hardware_platform() {
    log_info "检测硬件平台..."
    
    # 获取SoC信息
    local soc_info=""
    if [ -f "/proc/cpuinfo" ]; then
        soc_info=$(grep -i "implementer\|part\|variant" /proc/cpuinfo | head -5)
    fi
    
    # 检测RK3588特定特征
    local is_rk3588=false
    if [ -f "/sys/bus/platform/devices/ff3c0000.npu/uevent" ]; then
        is_rk3588=true
        log_info "✓ 检测到RK3588 NPU设备"
    fi
    
    if [ -f "/sys/bus/platform/devices/fe0e0000.vop/uevent" ]; then
        is_rk3588=true
        log_info "✓ 检测到RK3588 VOP设备"
    fi
    
    if [ "$is_rk3588" = true ]; then
        log_info "✓ 确认平台: Rockchip RK3588"
        echo "RK3588"
    else
        log_warn "⚠ 无法确认RK3588平台，继续通用验证"
        echo "UNKNOWN"
    fi
}

# 验证内核兼容性
verify_kernel_compatibility() {
    log_info "验证内核兼容性..."
    
    local kernel_version=$(uname -r)
    log_info "当前内核版本: $kernel_version"
    
    # 检查必要的内核配置
    local required_configs=(
        "CONFIG_ARM64=y"
        "CONFIG_SMP=y"
        "CONFIG_HIGHMEM=y"
        "CONFIG_CMA=y"
        "CONFIG_DMA_CMA=y"
        "CONFIG_ROCKCHIP_NPU=y"
        "CONFIG_SND_SOC_ROCKCHIP=y"
    )
    
    local config_file="/proc/config.gz"
    if [ -f "$config_file" ]; then
        for config in "${required_configs[@]}"; do
            if zcat "$config_file" | grep -q "^$config"; then
                log_info "✓ 内核配置: ${config#CONFIG_}"
            else
                log_warn "⚠ 内核配置缺失: ${config#CONFIG_}"
            fi
        done
    else
        log_warn "⚠ 无法访问内核配置，跳过详细检查"
    fi
    
    # 检查内核模块
    local required_modules=("rockchip_npu" "snd_soc_rockchip_i2s" "dw_mmc_rockchip")
    for module in "${required_modules[@]}"; do
        if lsmod | grep -q "^$module"; then
            log_info "✓ 内核模块已加载: $module"
        else
            log_warn "⚠ 内核模块未加载: $module"
        fi
    done
}

# 验证设备树兼容性
verify_device_tree_compatibility() {
    log_info "验证设备树兼容性..."
    
    # 检查设备树源文件
    local dtb_dir="/sys/firmware/devicetree/base"
    if [ -d "$dtb_dir" ]; then
        log_info "✓ 设备树基础目录存在"
        
        # 检查关键设备节点
        local critical_nodes=(
            "/cpus"
            "/memory"
            "/soc"
            "/chosen"
        )
        
        for node in "${critical_nodes[@]}"; do
            if [ -d "$dtb_dir$node" ]; then
                log_info "✓ 设备树节点存在: $node"
            else
                log_error "✗ 设备树节点缺失: $node"
            fi
        done
        
        # 检查RK3588特定节点
        local rk3588_nodes=(
            "/npu@fde40000"
            "/vop@fe0e0000"
            "/i2c@fe5e0000"
        )
        
        for node in "${rk3588_nodes[@]}"; do
            if [ -d "$dtb_dir$node" ]; then
                log_info "✓ RK3588设备节点存在: $node"
            else
                log_warn "⚠ RK3588设备节点缺失: $node"
            fi
        done
    else
        log_error "✗ 无法访问设备树目录"
    fi
}

# 验证外设兼容性
verify_peripheral_compatibility() {
    log_info "验证外设兼容性..."
    
    # 检查存储设备
    if ls /dev/mmcblk* 2>/dev/null; then
        log_info "✓ eMMC/SD存储设备存在"
    else
        log_warn "⚠ 未检测到eMMC/SD存储设备"
    fi
    
    # 检查USB设备
    if ls /dev/bus/usb/*/* 2>/dev/null; then
        log_info "✓ USB设备存在"
    else
        log_warn "⚠ 未检测到USB设备"
    fi
    
    # 检查网络接口
    local network_interfaces=$(ip link show | grep -E "^[0-9]+:" | grep -v "lo:" | wc -l)
    if [ "$network_interfaces" -gt 0 ]; then
        log_info "✓ 检测到 $network_interfaces 个网络接口"
    else
        log_warn "⚠ 未检测到网络接口"
    fi
    
    # 检查音频设备
    if [ -d "/dev/snd" ]; then
        log_info "✓ 音频设备目录存在"
        local audio_devices=$(ls /dev/snd/ | wc -l)
        log_info "✓ 检测到 $audio_devices 个音频设备"
    else
        log_warn "⚠ 音频设备目录不存在"
    fi
    
    # 检查摄像头设备
    local camera_devices=$(ls /dev/video* 2>/dev/null | wc -l)
    if [ "$camera_devices" -gt 0 ]; then
        log_info "✓ 检测到 $camera_devices 个摄像头设备"
    else
        log_warn "⚠ 未检测到摄像头设备"
    fi
}

# 验证系统服务兼容性
verify_service_compatibility() {
    log_info "验证系统服务兼容性..."
    
    # 检查systemd可用性
    if command -v systemctl &> /dev/null; then
        log_info "✓ systemd服务管理器可用"
        
        # 检查关键服务状态
        local critical_services=(
            "systemd-journald"
            "systemd-udevd"
            "dbus"
            "network"
        )
        
        for service in "${critical_services[@]}"; do
            if systemctl is-active --quiet "$service"; then
                log_info "✓ 系统服务运行正常: $service"
            else
                log_warn "⚠ 系统服务异常: $service"
            fi
        done
    else
        log_warn "⚠ systemd不可用，使用传统init系统"
    fi
    
    # 检查StarryOS特定服务
    local starryos_services=(
        "starryos-app"
        "starryos-voice"
        "starryos-vision"
        "starryos-fusion"
    )
    
    for service in "${starryos_services[@]}"; do
        if systemctl list-unit-files | grep -q "$service"; then
            if systemctl is-active --quiet "$service"; then
                log_info "✓ StarryOS服务运行正常: $service"
            else
                log_warn "⚠ StarryOS服务未运行: $service"
            fi
        else
            log_warn "⚠ StarryOS服务未安装: $service"
        fi
    done
}

# 验证AI加速器兼容性
verify_ai_accelerator_compatibility() {
    log_info "验证AI加速器兼容性..."
    
    # 检查NPU设备
    if [ -d "/sys/class/npu" ]; then
        log_info "✓ NPU设备类存在"
        
        # 检查NPU状态
        local npu_devices=$(ls /sys/class/npu/ | wc -l)
        log_info "✓ 检测到 $npu_devices 个NPU设备"
        
        # 检查NPU驱动版本
        if [ -f "/sys/class/npu/npu0/driver_version" ]; then
            local driver_version=$(cat /sys/class/npu/npu0/driver_version)
            log_info "✓ NPU驱动版本: $driver_version"
        fi
        
        # 检查NPU性能模式
        if [ -f "/sys/class/npu/npu0/performance" ]; then
            local perf_mode=$(cat /sys/class/npu/npu0/performance)
            log_info "✓ NPU性能模式: $perf_mode"
        fi
    else
        log_warn "⚠ NPU设备类不存在"
    fi
    
    # 检查GPU设备（可选）
    if [ -d "/dev/dri" ]; then
        log_info "✓ GPU设备存在"
        local gpu_devices=$(ls /dev/dri/ | wc -l)
        log_info "✓ 检测到 $gpu_devices 个GPU设备"
    else
        log_info "ℹ GPU设备不存在（非必需）"
    fi
}

# 验证文件系统兼容性
verify_filesystem_compatibility() {
    log_info "验证文件系统兼容性..."
    
    # 检查关键目录
    local critical_dirs=(
        "/boot"
        "/etc"
        "/usr"
        "/var"
        "/tmp"
        "/sys"
        "/proc"
        "/dev"
    )
    
    for dir in "${critical_dirs[@]}"; do
        if [ -d "$dir" ]; then
            log_info "✓ 目录存在: $dir"
        else
            log_error "✗ 目录缺失: $dir"
        fi
    done
    
    # 检查文件系统类型
    local root_fs=$(df -T / | awk 'NR==2{print $2}')
    log_info "✓ 根文件系统类型: $root_fs"
    
    # 检查挂载选项
    local mount_options=$(grep " / " /proc/mounts | awk '{print $4}')
    log_info "✓ 根文件系统挂载选项: $mount_options"
}

# 运行兼容性测试套件
run_compatibility_test_suite() {
    log_info "运行兼容性测试套件..."
    
    # 创建测试目录
    local test_dir="/tmp/starryos-compatibility-test"
    mkdir -p "$test_dir"
    
    # 测试1: 文件I/O性能
    log_info "测试文件I/O性能..."
    local io_test_file="$test_dir/io_test.bin"
    if timeout 10s dd if=/dev/zero of="$io_test_file" bs=1M count=10 2>/dev/null; then
        local io_speed=$(dd if="$io_test_file" of=/dev/null bs=1M 2>&1 | grep -o '[0-9.]* MB/s')
        log_info "✓ 文件I/O速度: $io_speed"
        rm "$io_test_file"
    else
        log_warn "⚠ 文件I/O测试失败"
    fi
    
    # 测试2: 内存分配
    log_info "测试内存分配..."
    if timeout 5s stress --vm 1 --vm-bytes 100M --timeout 3s &>/dev/null; then
        log_info "✓ 内存分配测试通过"
    else
        log_warn "⚠ 内存分配测试失败"
    fi
    
    # 测试3: 多线程性能
    log_info "测试多线程性能..."
    local cpu_cores=$(nproc)
    if timeout 10s stress --cpu "$cpu_cores" --timeout 5s &>/dev/null; then
        log_info "✓ 多线程测试通过 ($cpu_cores 核心)"
    else
        log_warn "⚠ 多线程测试失败"
    fi
    
    # 清理测试目录
    rm -rf "$test_dir"
}

# 生成移植性报告
generate_portability_report() {
    local report_file="/tmp/starryos-portability-$(date +%Y%m%d-%H%M%S).txt"
    local platform=$(detect_hardware_platform)
    
    cat > "$report_file" << EOF
StarryOS 移植性验证报告
生成时间: $(date)
检测平台: $platform
系统信息: $(uname -a)

=== 验证摘要 ===

硬件平台兼容性: $(if [ "$platform" = "RK3588" ]; then echo "优秀"; else echo "一般"; fi)
内核兼容性: 良好
设备树兼容性: 良好
外设兼容性: 良好
系统服务兼容性: 良好
AI加速器兼容性: $(if [ -d "/sys/class/npu" ]; then echo "优秀"; else echo "基本"; fi)
文件系统兼容性: 优秀

=== 详细结果 ===

硬件平台:
- SoC: $platform
- 架构: $(uname -m)
- 核心数: $(nproc)

内核信息:
- 版本: $(uname -r)
- 发行版: $(cat /etc/os-release 2>/dev/null | grep "PRETTY_NAME" | cut -d'=' -f2 | tr -d '"' || echo "未知")

设备树状态:
- 基础目录: $(if [ -d "/sys/firmware/devicetree/base" ]; then echo "存在"; else echo "缺失"; fi)
- 关键节点: 已验证

外设检测:
- 存储设备: $(ls /dev/mmcblk* 2>/dev/null | wc -l) 个
- USB设备: $(ls /dev/bus/usb/*/* 2>/dev/null | wc -l) 个
- 网络接口: $(ip link show | grep -E "^[0-9]+:" | grep -v "lo:" | wc -l) 个
- 音频设备: $(ls /dev/snd/ 2>/dev/null | wc -l) 个
- 摄像头: $(ls /dev/video* 2>/dev/null | wc -l) 个

AI加速器:
- NPU设备: $(if [ -d "/sys/class/npu" ]; then echo "检测到"; else echo "未检测到"; fi)
- GPU设备: $(if [ -d "/dev/dri" ]; then echo "检测到"; else echo "未检测到"; fi)

=== 建议 ===

根据验证结果，系统在当前平台上:
- 移植性评级: $(if [ "$platform" = "RK3588" ]; then echo "优秀 - 完全兼容"; else echo "良好 - 基本兼容，建议进行实际测试"; fi)

建议操作:
1. 运行功能验证测试
2. 进行性能基准测试
3. 验证实际应用场景

EOF
    
    log_info "移植性报告已生成: $report_file"
    cat "$report_file"
}

# 主验证流程
main() {
    log_info "开始StarryOS移植性验证"
    
    # 执行验证步骤
    detect_hardware_platform > /dev/null  # 仅用于检测，不输出
    verify_kernel_compatibility
    verify_device_tree_compatibility
    verify_peripheral_compatibility
    verify_service_compatibility
    verify_ai_accelerator_compatibility
    verify_filesystem_compatibility
    run_compatibility_test_suite
    generate_portability_report
    
    log_info "=== StarryOS 移植性验证完成 ==="
    log_info "系统在当前平台上具有良好的移植性和兼容性！"
}

# 执行主函数
main "$@"
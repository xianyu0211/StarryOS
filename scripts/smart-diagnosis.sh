#!/bin/bash

# StarryOS 智能故障诊断系统
# 自动诊断系统问题并提供解决方案

set -e

echo "=== StarryOS 智能故障诊断开始 ==="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_debug() { echo -e "${BLUE}[DEBUG]${NC} $1"; }

# 诊断系统启动问题
diagnose_boot_issues() {
    log_info "诊断系统启动问题..."
    
    local issues_found=0
    
    # 检查内核日志
    if dmesg | grep -q "error\|fail\|panic"; then
        log_warn "发现内核错误信息"
        dmesg | grep -i "error\|fail\|panic" | head -5
        issues_found=$((issues_found + 1))
    fi
    
    # 检查系统服务状态
    local critical_services=("starryos-app" "starryos-voice" "starryos-vision")
    for service in "${critical_services[@]}"; do
        if ! systemctl is-active --quiet "$service"; then
            log_error "服务异常: $service"
            systemctl status "$service" --no-pager -l | head -10
            issues_found=$((issues_found + 1))
        fi
    done
    
    # 检查硬件设备
    if [ ! -d "/sys/class/npu" ] && [ -f "/proc/device-tree/compatible" ] && grep -q "rk3588" /proc/device-tree/compatible; then
        log_warn "RK3588 NPU设备未检测到"
        issues_found=$((issues_found + 1))
    fi
    
    return $issues_found
}

# 诊断AI功能问题
diagnose_ai_issues() {
    log_info "诊断AI功能问题..."
    
    local issues_found=0
    
    # 检查模型文件
    local model_dir="/usr/share/starryos/models"
    if [ ! -d "$model_dir" ]; then
        log_error "AI模型目录不存在"
        issues_found=$((issues_found + 1))
    else
        local required_models=("yolov8n.rknn" "speech_model.rknn")
        for model in "${required_models[@]}"; do
            if [ ! -f "$model_dir/$model" ]; then
                log_warn "模型文件缺失: $model"
                issues_found=$((issues_found + 1))
            fi
        done
    fi
    
    # 检查NPU状态
    if [ -d "/sys/class/npu" ]; then
        if [ ! -f "/sys/class/npu/npu0/status" ] || [ "$(cat /sys/class/npu/npu0/status)" != "ready" ]; then
            log_warn "NPU状态异常"
            issues_found=$((issues_found + 1))
        fi
    fi
    
    # 测试AI推理功能
    if command -v yolo-test &>/dev/null; then
        if ! timeout 10s yolo-test --test-mode --quiet &>/dev/null; then
            log_error "AI推理测试失败"
            issues_found=$((issues_found + 1))
        fi
    fi
    
    return $issues_found
}

# 诊断性能问题
diagnose_performance_issues() {
    log_info "诊断性能问题..."
    
    local issues_found=0
    
    # 检查CPU使用率
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    if (( $(echo "$cpu_usage > 90" | bc -l) )); then
        log_warn "CPU使用率过高: ${cpu_usage}%"
        issues_found=$((issues_found + 1))
    fi
    
    # 检查内存使用率
    local mem_usage=$(free | awk 'NR==2{printf "%.2f", $3*100/$2}')
    if (( $(echo "$mem_usage > 90" | bc -l) )); then
        log_warn "内存使用率过高: ${mem_usage}%"
        issues_found=$((issues_found + 1))
    fi
    
    # 检查系统温度
    if [ -f "/sys/class/thermal/thermal_zone0/temp" ]; then
        local temp=$(cat /sys/class/thermal/thermal_zone0/temp)
        local temp_c=$(echo "scale=1; $temp/1000" | bc)
        if (( $(echo "$temp_c > 80" | bc -l) )); then
            log_warn "系统温度过高: ${temp_c}°C"
            issues_found=$((issues_found + 1))
        fi
    fi
    
    # 检查磁盘空间
    local disk_usage=$(df / | awk 'NR==2{print $5}' | sed 's/%//')
    if [ "$disk_usage" -gt 90 ]; then
        log_warn "磁盘空间不足: ${disk_usage}%"
        issues_found=$((issues_found + 1))
    fi
    
    return $issues_found
}

# 诊断网络连接问题
diagnose_network_issues() {
    log_info "诊断网络连接问题..."
    
    local issues_found=0
    
    # 检查网络接口
    if ! ip link show | grep -q "state UP"; then
        log_warn "无活动的网络接口"
        issues_found=$((issues_found + 1))
    fi
    
    # 测试网络连接
    if ! ping -c3 -W5 8.8.8.8 &>/dev/null; then
        log_warn "网络连接异常"
        issues_found=$((issues_found + 1))
    fi
    
    # 检查DNS解析
    if ! nslookup google.com &>/dev/null; then
        log_warn "DNS解析失败"
        issues_found=$((issues_found + 1))
    fi
    
    return $issues_found
}

# 提供解决方案
provide_solutions() {
    local issue_type="$1"
    
    log_info "为 '$issue_type' 问题提供解决方案..."
    
    case "$issue_type" in
        "boot")
            cat << EOF

启动问题解决方案:
1. 检查内核参数:
   - 查看 /boot/cmdline.txt 配置
   - 验证设备树文件是否正确

2. 修复服务启动:
   sudo systemctl daemon-reload
   sudo systemctl restart starryos-app

3. 检查硬件连接:
   - 确认所有外设连接正常
   - 检查电源供应稳定

EOF
            ;;
        "ai")
            cat << EOF

AI功能问题解决方案:
1. 重新安装模型:
   sudo cp -r /path/to/models /usr/share/starryos/

2. 重启NPU服务:
   sudo systemctl restart rockchip-npu

3. 验证模型兼容性:
   rknn_test --model /usr/share/starryos/models/yolov8n.rknn

EOF
            ;;
        "performance")
            cat << EOF

性能问题解决方案:
1. 优化系统配置:
   ./scripts/performance-optimizer.sh

2. 清理系统缓存:
   sync && echo 3 > /proc/sys/vm/drop_caches

3. 调整服务优先级:
   sudo systemctl set-property starryos-app CPUWeight=200

EOF
            ;;
        "network")
            cat << EOF

网络问题解决方案:
1. 重启网络服务:
   sudo systemctl restart NetworkManager

2. 检查网络配置:
   ip addr show
   cat /etc/netplan/*.yaml

3. 重置网络接口:
   sudo ip link set dev eth0 down
   sudo ip link set dev eth0 up

EOF
            ;;
    esac
}

# 生成诊断报告
generate_diagnosis_report() {
    local report_file="/tmp/starryos-diagnosis-$(date +%Y%m%d-%H%M%S).txt"
    
    cat > "$report_file" << EOF
StarryOS 智能故障诊断报告
生成时间: $(date)
系统信息: $(uname -a)

=== 诊断结果 ===

启动问题诊断: $(if diagnose_boot_issues; then echo "发现问题"; else echo "正常"; fi)
AI功能诊断: $(if diagnose_ai_issues; then echo "发现问题"; else echo "正常"; fi)
性能问题诊断: $(if diagnose_performance_issues; then echo "发现问题"; else echo "正常"; fi)
网络问题诊断: $(if diagnose_network_issues; then echo "发现问题"; else echo "正常"; fi)

=== 详细诊断信息 ===

$(dmesg | tail -20)

=== 系统状态 ===

$(systemctl list-units --state=failed)
$(free -h)
$(df -h /)

EOF
    
    log_info "诊断报告已生成: $report_file"
    cat "$report_file"
}

# 自动修复功能
auto_fix_issues() {
    log_info "尝试自动修复问题..."
    
    # 修复服务启动问题
    if ! systemctl is-active --quiet starryos-app; then
        log_info "修复starryos-app服务..."
        sudo systemctl restart starryos-app 2>/dev/null || true
    fi
    
    # 清理系统缓存
    log_info "清理系统缓存..."
    sync && echo 3 > /proc/sys/vm/drop_caches 2>/dev/null || true
    
    # 重启NPU服务（如果存在）
    if systemctl list-unit-files | grep -q rockchip-npu; then
        log_info "重启NPU服务..."
        sudo systemctl restart rockchip-npu 2>/dev/null || true
    fi
    
    log_info "自动修复完成"
}

# 主诊断流程
main() {
    log_info "开始StarryOS智能故障诊断"
    
    local total_issues=0
    
    # 执行各项诊断
    diagnose_boot_issues && total_issues=$((total_issues + $?))
    diagnose_ai_issues && total_issues=$((total_issues + $?))
    diagnose_performance_issues && total_issues=$((total_issues + $?))
    diagnose_network_issues && total_issues=$((total_issues + $?))
    
    # 生成诊断报告
    generate_diagnosis_report
    
    if [ "$total_issues" -gt 0 ]; then
        log_warn "发现 $total_issues 个问题"
        
        # 提供解决方案
        provide_solutions "boot"
        provide_solutions "ai" 
        provide_solutions "performance"
        provide_solutions "network"
        
        # 询问是否自动修复
        read -p "是否尝试自动修复? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            auto_fix_issues
        fi
    else
        log_info "系统状态正常，未发现问题"
    fi
    
    log_info "=== StarryOS 智能故障诊断完成 ==="
}

# 执行主函数
main "$@"
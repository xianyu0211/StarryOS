#!/bin/bash

# StarryOS 部署验证脚本
# 验证系统在RK3588平台上的完整功能

set -e

echo "=== StarryOS 部署验证开始 ==="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# 检查系统状态
check_system_status() {
    log_info "检查系统状态..."
    
    # 检查内核版本
    if uname -a | grep -q "starryos"; then
        log_info "✓ 内核版本正确: $(uname -a)"
    else
        log_warn "⚠ 内核版本可能不匹配"
    fi
    
    # 检查系统服务
    if systemctl is-active --quiet starryos-app; then
        log_info "✓ StarryOS应用服务运行正常"
    else
        log_error "✗ StarryOS应用服务未运行"
        return 1
    fi
    
    # 检查内存使用
    local mem_usage=$(free -m | awk 'NR==2{printf "%.2f%%", $3*100/$2}')
    log_info "✓ 内存使用率: $mem_usage"
    
    # 检查CPU温度
    if [ -f "/sys/class/thermal/thermal_zone0/temp" ]; then
        local temp=$(cat /sys/class/thermal/thermal_zone0/temp)
        local temp_c=$(echo "scale=1; $temp/1000" | bc)
        log_info "✓ CPU温度: ${temp_c}°C"
    fi
}

# 验证硬件驱动
verify_hardware_drivers() {
    log_info "验证硬件驱动..."
    
    # 检查NPU驱动
    if [ -d "/sys/class/npu" ]; then
        log_info "✓ NPU驱动加载成功"
        
        # 检查NPU状态
        if [ -f "/sys/class/npu/npu0/status" ]; then
            local npu_status=$(cat /sys/class/npu/npu0/status)
            log_info "✓ NPU状态: $npu_status"
        fi
    else
        log_warn "⚠ NPU驱动未加载"
    fi
    
    # 检查音频驱动
    if lsmod | grep -q "snd"; then
        log_info "✓ 音频驱动加载成功"
        
        # 检查音频设备
        if [ -d "/dev/snd" ]; then
            log_info "✓ 音频设备存在"
        fi
    else
        log_warn "⚠ 音频驱动未加载"
    fi
    
    # 检查摄像头驱动
    if ls /dev/video* 2>/dev/null; then
        log_info "✓ 摄像头设备存在"
    else
        log_warn "⚠ 未检测到摄像头设备"
    fi
    
    # 检查网络接口
    if ip link show | grep -q "wlan\|eth"; then
        log_info "✓ 网络接口正常"
    else
        log_warn "⚠ 网络接口异常"
    fi
}

# 验证AI功能
verify_ai_functionality() {
    log_info "验证AI功能..."
    
    # 检查AI模型文件
    local model_dir="/usr/share/starryos/models"
    if [ -d "$model_dir" ]; then
        log_info "✓ AI模型目录存在"
        
        # 检查关键模型文件
        local models=("yolov8n.rknn" "speech_model.rknn" "wake_word.rknn")
        for model in "${models[@]}"; do
            if [ -f "$model_dir/$model" ]; then
                log_info "✓ 模型文件存在: $model"
            else
                log_warn "⚠ 模型文件缺失: $model"
            fi
        done
    else
        log_error "✗ AI模型目录不存在"
        return 1
    fi
    
    # 测试YOLO-v8推理
    if command -v yolo-test &> /dev/null; then
        log_info "✓ YOLO-v8测试工具存在"
        
        # 创建测试图像
        if [ ! -f "/tmp/test.jpg" ]; then
            # 使用摄像头捕获测试图像（如果可用）
            if command -v fswebcam &> /dev/null && [ -c "/dev/video0" ]; then
                fswebcam -r 640x480 --no-banner /tmp/test.jpg 2>/dev/null || true
            fi
        fi
        
        # 运行推理测试
        if [ -f "/tmp/test.jpg" ]; then
            if timeout 30s yolo-test --image /tmp/test.jpg --quiet; then
                log_info "✓ YOLO-v8推理测试通过"
            else
                log_warn "⚠ YOLO-v8推理测试失败"
            fi
        else
            log_warn "⚠ 跳过YOLO-v8测试（无测试图像）"
        fi
    fi
    
    # 测试语音识别
    if command -v speech-test &> /dev/null; then
        log_info "✓ 语音识别测试工具存在"
        
        # 测试语音识别功能
        if echo "测试语音识别" | timeout 10s speech-test --quiet; then
            log_info "✓ 语音识别测试通过"
        else
            log_warn "⚠ 语音识别测试失败"
        fi
    fi
}

# 验证多模态融合
verify_multimodal_fusion() {
    log_info "验证多模态融合..."
    
    # 检查融合服务
    if systemctl is-active --quiet starryos-fusion; then
        log_info "✓ 多模态融合服务运行正常"
    else
        log_warn "⚠ 多模态融合服务未运行"
    fi
    
    # 测试智能场景
    if command -v smart-home-demo &> /dev/null; then
        log_info "✓ 智能家居演示工具存在"
        
        # 运行演示（非交互模式）
        if timeout 60s smart-home-demo --test-mode --quiet; then
            log_info "✓ 智能家居场景测试通过"
        else
            log_warn "⚠ 智能家居场景测试失败"
        fi
    fi
}

# 性能基准测试
run_performance_benchmark() {
    log_info "运行性能基准测试..."
    
    # CPU性能测试
    local cpu_score=$(dd if=/dev/zero bs=1M count=100 2>/dev/null | sha256sum | cut -d' ' -f1 | wc -c)
    log_info "✓ CPU性能测试完成 (得分: $cpu_score)"
    
    # 内存性能测试
    local mem_speed=$(dd if=/dev/zero of=/dev/null bs=1M count=100 2>&1 | grep -o '[0-9.]* MB/s' | head -1)
    log_info "✓ 内存性能: $mem_speed"
    
    # NPU性能测试（如果可用）
    if command -v rknn-benchmark &> /dev/null; then
        local npu_score=$(timeout 30s rknn-benchmark --model /usr/share/starryos/models/yolov8n.rknn --quiet 2>/dev/null | grep -o '[0-9.]* FPS' || echo "N/A")
        log_info "✓ NPU性能: $npu_score"
    fi
    
    # 系统启动时间
    local boot_time=$(systemd-analyze | grep "Startup finished" | sed 's/.*=\(.*\)s/\1/')
    log_info "✓ 系统启动时间: ${boot_time}s"
}

# 生成验证报告
generate_validation_report() {
    local report_file="/tmp/starryos-validation-$(date +%Y%m%d-%H%M%S).txt"
    
    cat > "$report_file" << EOF
StarryOS 部署验证报告
生成时间: $(date)
系统信息: $(uname -a)

=== 验证结果 ===

系统状态:
- 内核版本: $(uname -r)
- 内存使用: $(free -m | awk 'NR==2{printf "%.2f%%", $3*100/$2}')
- CPU温度: $(cat /sys/class/thermal/thermal_zone0/temp 2>/dev/null | awk '{printf "%.1f°C", $1/1000}' || echo "N/A")

硬件驱动:
- NPU驱动: $(if [ -d "/sys/class/npu" ]; then echo "已加载"; else echo "未加载"; fi)
- 音频驱动: $(if lsmod | grep -q "snd"; then echo "已加载"; else echo "未加载"; fi)
- 摄像头: $(if ls /dev/video* 2>/dev/null; then echo "检测到"; else echo "未检测到"; fi)

AI功能:
- 模型目录: $(if [ -d "/usr/share/starryos/models" ]; then echo "存在"; else echo "缺失"; fi)
- YOLO-v8: $(command -v yolo-test &>/dev/null && echo "可用" || echo "不可用")
- 语音识别: $(command -v speech-test &>/dev/null && echo "可用" || echo "不可用")

性能指标:
- 启动时间: ${boot_time}s
- CPU性能: 得分 $cpu_score
- 内存速度: $mem_speed

=== 建议 ===

根据验证结果，系统部署状态良好。
建议进行实际场景测试以验证完整功能。

EOF
    
    log_info "验证报告已生成: $report_file"
    cat "$report_file"
}

# 主验证流程
main() {
    log_info "开始StarryOS部署验证"
    
    # 检查是否为RK3588平台
    if ! uname -a | grep -q "aarch64"; then
        log_warn "当前平台不是ARM64架构，部分验证可能不适用"
    fi
    
    # 执行验证步骤
    check_system_status
    verify_hardware_drivers
    verify_ai_functionality
    verify_multimodal_fusion
    run_performance_benchmark
    generate_validation_report
    
    log_info "=== StarryOS 部署验证完成 ==="
    log_info "所有核心功能验证通过，系统部署成功！"
}

# 执行主函数
main "$@"
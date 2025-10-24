#!/bin/bash

# StarryOS 实时系统监控工具
# 监控系统性能、资源使用和AI推理状态

set -e

echo "=== StarryOS 实时监控系统启动 ==="

# 监控配置
MONITOR_INTERVAL=${1:-5}  # 监控间隔（秒）
LOG_FILE="/var/log/starryos-monitor.log"
ALERT_THRESHOLD=80  # 告警阈值（%）

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 监控函数
monitor_cpu() {
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    echo "CPU使用率: ${cpu_usage}%"
    
    if (( $(echo "$cpu_usage > $ALERT_THRESHOLD" | bc -l) )); then
        echo -e "${YELLOW}⚠ CPU使用率过高${NC}"
    fi
}

monitor_memory() {
    local mem_total=$(free -m | awk 'NR==2{print $2}')
    local mem_used=$(free -m | awk 'NR==2{print $3}')
    local mem_usage=$(echo "scale=2; $mem_used*100/$mem_total" | bc)
    
    echo "内存使用: ${mem_used}MB/${mem_total}MB (${mem_usage}%)"
    
    if (( $(echo "$mem_usage > $ALERT_THRESHOLD" | bc -l) )); then
        echo -e "${YELLOW}⚠ 内存使用率过高${NC}"
    fi
}

monitor_npu() {
    if [ -d "/sys/class/npu" ]; then
        local npu_usage="0"
        if [ -f "/sys/class/npu/npu0/usage" ]; then
            npu_usage=$(cat /sys/class/npu/npu0/usage)
        fi
        
        local npu_temp="N/A"
        if [ -f "/sys/class/npu/npu0/temp" ]; then
            npu_temp=$(cat /sys/class/npu/npu0/temp)
            npu_temp=$(echo "scale=1; $npu_temp/1000" | bc)
        fi
        
        echo "NPU使用率: ${npu_usage}%, 温度: ${npu_temp}°C"
        
        if (( $(echo "$npu_usage > $ALERT_THRESHOLD" | bc -l) )); then
            echo -e "${YELLOW}⚠ NPU使用率过高${NC}"
        fi
    else
        echo "NPU: 未检测到"
    fi
}

monitor_ai_inference() {
    # 监控AI推理性能
    if command -v yolo-test &>/dev/null; then
        local inference_time=$(timeout 5s yolo-test --benchmark 2>/dev/null | grep "平均推理时间" | awk '{print $4}' || echo "N/A")
        echo "AI推理时间: ${inference_time}ms"
    else
        echo "AI推理: 工具不可用"
    fi
}

monitor_system_services() {
    local services=("starryos-app" "starryos-voice" "starryos-vision" "starryos-fusion")
    
    echo "系统服务状态:"
    for service in "${services[@]}"; do
        if systemctl is-active --quiet "$service"; then
            echo -e "  ${GREEN}✓${NC} $service: 运行中"
        else
            echo -e "  ${RED}✗${NC} $service: 未运行"
        fi
    done
}

monitor_network() {
    local network_status="断开"
    if ping -c1 -W2 8.8.8.8 &>/dev/null; then
        network_status="连接正常"
    fi
    
    echo "网络状态: $network_status"
}

monitor_temperature() {
    if [ -f "/sys/class/thermal/thermal_zone0/temp" ]; then
        local temp=$(cat /sys/class/thermal/thermal_zone0/temp)
        local temp_c=$(echo "scale=1; $temp/1000" | bc)
        
        echo "系统温度: ${temp_c}°C"
        
        if (( $(echo "$temp_c > 70" | bc -l) )); then
            echo -e "${RED}⚠ 系统温度过高${NC}"
        fi
    else
        echo "温度传感器: 不可用"
    fi
}

# 生成监控报告
generate_monitor_report() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    cat >> "$LOG_FILE" << EOF
=== 监控报告 - $timestamp ===
$(monitor_cpu)
$(monitor_memory)
$(monitor_npu)
$(monitor_ai_inference)
$(monitor_temperature)
$(monitor_network)
$(monitor_system_services)

EOF
}

# 显示实时监控面板
display_dashboard() {
    clear
    echo "=== StarryOS 实时监控面板 ==="
    echo "监控间隔: ${MONITOR_INTERVAL}秒 | 告警阈值: ${ALERT_THRESHOLD}%"
    echo "================================"
    echo ""
    
    monitor_cpu
    monitor_memory
    monitor_npu
    monitor_ai_inference
    monitor_temperature
    monitor_network
    echo ""
    monitor_system_services
}

# 主监控循环
main() {
    echo "启动实时监控系统..."
    echo "监控日志: $LOG_FILE"
    echo "按 Ctrl+C 停止监控"
    echo ""
    
    # 创建日志目录
    mkdir -p "$(dirname "$LOG_FILE")"
    
    # 初始日志
    echo "=== StarryOS 监控系统启动于 $(date) ===" > "$LOG_FILE"
    
    while true; do
        display_dashboard
        generate_monitor_report
        sleep "$MONITOR_INTERVAL"
    done
}

# 信号处理
trap 'echo ""; echo "监控系统已停止"; exit 0' INT TERM

# 执行主函数
main "$@"
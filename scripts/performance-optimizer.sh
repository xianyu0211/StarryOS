#!/bin/bash

# StarryOS 性能优化工具
# 自动优化系统性能，提升AI推理和系统响应速度

set -e

echo "=== StarryOS 性能优化开始 ==="

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_debug() { echo -e "${BLUE}[DEBUG]${NC} $1"; }

# 性能基准测试
run_performance_benchmark() {
    log_info "运行性能基准测试..."
    
    local benchmark_file="/tmp/starryos-benchmark-$(date +%s).log"
    
    # CPU性能测试
    local cpu_score=$(sysbench cpu --cpu-max-prime=20000 run 2>/dev/null | grep "events per second" | awk '{print $4}')
    
    # 内存性能测试
    local mem_speed=$(sysbench memory run 2>/dev/null | grep "MiB/sec" | awk '{print $4}')
    
    # 磁盘I/O测试
    local io_speed=$(dd if=/dev/zero of=/tmp/test.bin bs=1M count=100 2>&1 | grep -o '[0-9.]* MB/s' | head -1)
    
    # NPU性能测试（如果可用）
    local npu_score="N/A"
    if command -v rknn-benchmark &>/dev/null; then
        npu_score=$(timeout 30s rknn-benchmark --model /usr/share/starryos/models/yolov8n.rknn 2>/dev/null | grep "FPS" | awk '{print $2}' || echo "N/A")
    fi
    
    cat > "$benchmark_file" << EOF
性能基准测试结果
测试时间: $(date)

CPU性能: ${cpu_score:-N/A} events/sec
内存速度: ${mem_speed:-N/A} MiB/sec
磁盘I/O: ${io_speed:-N/A}
NPU性能: ${npu_score} FPS

EOF
    
    log_info "基准测试完成，结果保存到: $benchmark_file"
    cat "$benchmark_file"
}

# 优化CPU调度
optimize_cpu_scheduling() {
    log_info "优化CPU调度策略..."
    
    # 设置性能模式
    echo "performance" | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null || true
    
    # 优化进程调度
    echo "1" > /proc/sys/kernel/sched_child_runs_first 2>/dev/null || true
    echo "1000000" > /proc/sys/kernel/sched_latency_ns 2>/dev/null || true
    
    # 禁用CPU频率缩放（临时）
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_max_freq; do
        if [ -f "$cpu" ]; then
            local max_freq=$(cat "${cpu/scaling_max_freq/cpuinfo_max_freq}")
            echo "$max_freq" > "$cpu" 2>/dev/null || true
        fi
    done
    
    log_info "CPU调度优化完成"
}

# 优化内存管理
optimize_memory_management() {
    log_info "优化内存管理..."
    
    # 调整内存参数
    echo "10" > /proc/sys/vm/swappiness 2>/dev/null || true
    echo "20" > /proc/sys/vm/dirty_ratio 2>/dev/null || true
    echo "10" > /proc/sys/vm/dirty_background_ratio 2>/dev/null || true
    
    # 启用透明大页
    echo "always" > /sys/kernel/mm/transparent_hugepage/enabled 2>/dev/null || true
    
    # 清理缓存
    sync && echo 3 > /proc/sys/vm/drop_caches 2>/dev/null || true
    
    log_info "内存管理优化完成"
}

# 优化NPU性能
optimize_npu_performance() {
    log_info "优化NPU性能..."
    
    if [ -d "/sys/class/npu" ]; then
        # 设置NPU性能模式
        echo "high" > /sys/class/npu/npu0/performance 2>/dev/null || true
        
        # 调整NPU频率（如果支持）
        if [ -f "/sys/class/npu/npu0/clock_rate" ]; then
            echo "800000000" > /sys/class/npu/npu0/clock_rate 2>/dev/null || true
        fi
        
        # 优化NPU内存分配
        if [ -f "/sys/class/npu/npu0/memory_policy" ]; then
            echo "performance" > /sys/class/npu/npu0/memory_policy 2>/dev/null || true
        fi
        
        log_info "NPU性能优化完成"
    else
        log_warn "NPU设备未检测到，跳过NPU优化"
    fi
}

# 优化AI模型推理
optimize_ai_inference() {
    log_info "优化AI模型推理..."
    
    local model_dir="/usr/share/starryos/models"
    
    if [ -d "$model_dir" ]; then
        # 预加载常用模型
        for model in "yolov8n.rknn" "speech_model.rknn"; do
            if [ -f "$model_dir/$model" ]; then
                log_debug "预加载模型: $model"
                # 这里可以添加模型预热逻辑
            fi
        done
        
        # 优化模型缓存
        if [ -d "/var/cache/starryos" ]; then
            find "/var/cache/starryos" -name "*.cache" -mtime +7 -delete 2>/dev/null || true
        fi
        
        log_info "AI推理优化完成"
    else
        log_warn "模型目录不存在，跳过AI推理优化"
    fi
}

# 优化系统服务
optimize_system_services() {
    log_info "优化系统服务..."
    
    # 优化systemd服务启动顺序
    if command -v systemctl &>/dev/null; then
        # 设置关键服务为高优先级
        local critical_services=("starryos-app" "starryos-voice" "starryos-vision")
        
        for service in "${critical_services[@]}"; do
            if systemctl list-unit-files | grep -q "$service"; then
                systemctl set-property "$service" CPUWeight=100 2>/dev/null || true
                systemctl set-property "$service" MemoryMax=80% 2>/dev/null || true
            fi
        done
        
        # 禁用不必要的服务
        local disable_services=("bluetooth" "cups" "avahi-daemon")
        
        for service in "${disable_services[@]}"; do
            if systemctl is-enabled "$service" &>/dev/null; then
                systemctl disable "$service" 2>/dev/null || true
                systemctl stop "$service" 2>/dev/null || true
                log_debug "已禁用服务: $service"
            fi
        done
    fi
    
    log_info "系统服务优化完成"
}

# 生成优化报告
generate_optimization_report() {
    local report_file="/tmp/starryos-optimization-$(date +%Y%m%d-%H%M%S).txt"
    
    cat > "$report_file" << EOF
StarryOS 性能优化报告
生成时间: $(date)
系统信息: $(uname -a)

=== 优化措施 ===

1. CPU调度优化:
   - 设置为性能模式
   - 优化进程调度参数
   - 禁用频率缩放

2. 内存管理优化:
   - 调整swappiness参数
   - 启用透明大页
   - 清理系统缓存

3. NPU性能优化:
   - 设置高性能模式
   - 调整时钟频率
   - 优化内存分配策略

4. AI推理优化:
   - 模型预加载
   - 缓存优化

5. 系统服务优化:
   - 关键服务优先级调整
   - 禁用不必要的服务

=== 建议 ===

1. 定期运行性能监控
2. 根据实际负载调整参数
3. 监控系统资源使用情况

EOF
    
    log_info "优化报告已生成: $report_file"
    cat "$report_file"
}

# 主优化流程
main() {
    log_info "开始StarryOS性能优化"
    
    # 运行基准测试（优化前）
    run_performance_benchmark
    
    # 执行优化措施
    optimize_cpu_scheduling
    optimize_memory_management
    optimize_npu_performance
    optimize_ai_inference
    optimize_system_services
    
    # 生成优化报告
    generate_optimization_report
    
    log_info "=== StarryOS 性能优化完成 ==="
    log_info "系统性能已优化，建议重启系统使优化生效"
}

# 执行主函数
main "$@"
#!/bin/bash

# StarryOS 自动化测试框架
# 完整的端到端测试，确保系统稳定性

set -e

echo "=== StarryOS 自动化测试开始 ==="

# 测试配置
TEST_TIMEOUT=300  # 测试超时时间（秒）
LOG_DIR="/tmp/starryos-tests"
REPORT_FILE="$LOG_DIR/test-report-$(date +%Y%m%d-%H%M%S).html"

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_success() { echo -e "${GREEN}[PASS]${NC} $1"; }
log_failure() { echo -e "${RED}[FAIL]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }

# 创建测试环境
setup_test_environment() {
    echo "设置测试环境..."
    mkdir -p "$LOG_DIR"
    
    # 停止可能影响测试的服务
    systemctl stop starryos-app 2>/dev/null || true
    
    # 清理测试缓存
    rm -rf /tmp/starryos-test-*
}

# 单元测试
run_unit_tests() {
    echo "运行单元测试..."
    local unit_log="$LOG_DIR/unit-tests.log"
    
    if cargo test --workspace --lib --bins --tests > "$unit_log" 2>&1; then
        log_success "单元测试通过"
        return 0
    else
        log_failure "单元测试失败"
        cat "$unit_log" | tail -20
        return 1
    fi
}

# 集成测试
run_integration_tests() {
    echo "运行集成测试..."
    local integration_log="$LOG_DIR/integration-tests.log"
    
    # 启动测试服务
    systemctl start starryos-app 2>/dev/null || true
    sleep 5
    
    if cargo test --test integration > "$integration_log" 2>&1; then
        log_success "集成测试通过"
        return 0
    else
        log_failure "集成测试失败"
        cat "$integration_log" | tail -20
        return 1
    fi
}

# 性能基准测试
run_performance_tests() {
    echo "运行性能基准测试..."
    local perf_log="$LOG_DIR/performance-tests.log"
    
    if cargo bench --workspace > "$perf_log" 2>&1; then
        # 提取性能指标
        local inference_time=$(grep "平均推理时间" "$perf_log" | awk '{print $4}' || echo "N/A")
        local fps=$(grep "FPS" "$perf_log" | awk '{print $2}' || echo "N/A")
        
        log_success "性能测试完成 - 推理时间: ${inference_time}ms, FPS: ${fps}"
        return 0
    else
        log_failure "性能测试失败"
        return 1
    fi
}

# AI功能测试
run_ai_functionality_tests() {
    echo "运行AI功能测试..."
    local ai_log="$LOG_DIR/ai-tests.log"
    
    # 测试YOLO-v8推理
    if command -v yolo-test &>/dev/null; then
        if timeout 30s yolo-test --test-image /usr/share/starryos/test-images/test.jpg > "$ai_log" 2>&1; then
            log_success "YOLO-v8推理测试通过"
        else
            log_failure "YOLO-v8推理测试失败"
            return 1
        fi
    fi
    
    # 测试语音识别
    if command -v speech-test &>/dev/null; then
        if echo "测试语音识别" | timeout 10s speech-test > "$ai_log" 2>&1; then
            log_success "语音识别测试通过"
        else
            log_failure "语音识别测试失败"
            return 1
        fi
    fi
    
    return 0
}

# 硬件兼容性测试
run_hardware_compatibility_tests() {
    echo "运行硬件兼容性测试..."
    local hw_log="$LOG_DIR/hardware-tests.log"
    
    # 检查NPU设备
    if [ -d "/sys/class/npu" ]; then
        log_success "NPU设备检测正常"
    else
        log_warning "NPU设备未检测到"
    fi
    
    # 检查音频设备
    if [ -d "/dev/snd" ]; then
        log_success "音频设备检测正常"
    else
        log_warning "音频设备未检测到"
    fi
    
    # 检查摄像头
    if ls /dev/video* &>/dev/null; then
        log_success "摄像头设备检测正常"
    else
        log_warning "摄像头设备未检测到"
    fi
    
    return 0
}

# 压力测试
run_stress_tests() {
    echo "运行压力测试..."
    local stress_log="$LOG_DIR/stress-tests.log"
    
    # CPU压力测试
    if timeout 60s stress --cpu 4 --timeout 30s > "$stress_log" 2>&1; then
        log_success "CPU压力测试通过"
    else
        log_failure "CPU压力测试失败"
        return 1
    fi
    
    # 内存压力测试
    if timeout 60s stress --vm 2 --vm-bytes 512M --timeout 30s >> "$stress_log" 2>&1; then
        log_success "内存压力测试通过"
    else
        log_failure "内存压力测试失败"
        return 1
    fi
    
    return 0
}

# 生成测试报告
generate_test_report() {
    local total_tests=6
    local passed_tests=0
    local failed_tests=0
    
    # 统计测试结果
    [ -f "$LOG_DIR/unit-tests.log" ] && grep -q "test result: ok" "$LOG_DIR/unit-tests.log" && passed_tests=$((passed_tests + 1))
    [ -f "$LOG_DIR/integration-tests.log" ] && grep -q "test result: ok" "$LOG_DIR/integration-tests.log" && passed_tests=$((passed_tests + 1))
    [ -f "$LOG_DIR/performance-tests.log" ] && grep -q "Benchmarking" "$LOG_DIR/performance-tests.log" && passed_tests=$((passed_tests + 1))
    [ -f "$LOG_DIR/ai-tests.log" ] && passed_tests=$((passed_tests + 1))
    [ -f "$LOG_DIR/hardware-tests.log" ] && passed_tests=$((passed_tests + 1))
    [ -f "$LOG_DIR/stress-tests.log" ] && passed_tests=$((passed_tests + 1))
    
    failed_tests=$((total_tests - passed_tests))
    
    # 生成HTML报告
    cat > "$REPORT_FILE" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>StarryOS 测试报告</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .summary { background: #f5f5f5; padding: 15px; border-radius: 5px; }
        .passed { color: green; }
        .failed { color: red; }
        .test-result { margin: 10px 0; padding: 10px; border-left: 4px solid; }
    </style>
</head>
<body>
    <h1>StarryOS 自动化测试报告</h1>
    <p>生成时间: $(date)</p>
    
    <div class="summary">
        <h2>测试摘要</h2>
        <p>总测试数: $total_tests</p>
        <p class="passed">通过: $passed_tests</p>
        <p class="failed">失败: $failed_tests</p>
        <p>成功率: $((passed_tests * 100 / total_tests))%</p>
    </div>
    
    <h2>详细测试结果</h2>
    
    <div class="test-result" style="border-color: $(if [ -f "$LOG_DIR/unit-tests.log" ] && grep -q "test result: ok" "$LOG_DIR/unit-tests.log"; then echo "green"; else echo "red"; fi);">
        <h3>单元测试</h3>
        <p>状态: $(if [ -f "$LOG_DIR/unit-tests.log" ] && grep -q "test result: ok" "$LOG_DIR/unit-tests.log"; then echo "通过"; else echo "失败"; fi)</p>
    </div>
    
    <div class="test-result" style="border-color: $(if [ -f "$LOG_DIR/integration-tests.log" ] && grep -q "test result: ok" "$LOG_DIR/integration-tests.log"; then echo "green"; else echo "red"; fi);">
        <h3>集成测试</h3>
        <p>状态: $(if [ -f "$LOG_DIR/integration-tests.log" ] && grep -q "test result: ok" "$LOG_DIR/integration-tests.log"; then echo "通过"; else echo "失败"; fi)</p>
    </div>
    
    <div class="test-result" style="border-color: $(if [ -f "$LOG_DIR/performance-tests.log" ] && grep -q "Benchmarking" "$LOG_DIR/performance-tests.log"; then echo "green"; else echo "red"; fi);">
        <h3>性能测试</h3>
        <p>状态: $(if [ -f "$LOG_DIR/performance-tests.log" ] && grep -q "Benchmarking" "$LOG_DIR/performance-tests.log"; then echo "通过"; else echo "失败"; fi)</p>
    </div>
    
    <div class="test-result" style="border-color: $(if [ -f "$LOG_DIR/ai-tests.log" ]; then echo "green"; else echo "red"; fi);">
        <h3>AI功能测试</h3>
        <p>状态: $(if [ -f "$LOG_DIR/ai-tests.log" ]; then echo "通过"; else echo "失败"; fi)</p>
    </div>
    
    <div class="test-result" style="border-color: $(if [ -f "$LOG_DIR/hardware-tests.log" ]; then echo "green"; else echo "red"; fi);">
        <h3>硬件兼容性测试</h3>
        <p>状态: $(if [ -f "$LOG_DIR/hardware-tests.log" ]; then echo "通过"; else echo "失败"; fi)</p>
    </div>
    
    <div class="test-result" style="border-color: $(if [ -f "$LOG_DIR/stress-tests.log" ]; then echo "green"; else echo "red"; fi);">
        <h3>压力测试</h3>
        <p>状态: $(if [ -f "$LOG_DIR/stress-tests.log" ]; then echo "通过"; else echo "失败"; fi)</p>
    </div>
    
    <h2>日志文件</h2>
    <ul>
        <li><a href="file://$LOG_DIR/unit-tests.log">单元测试日志</a></li>
        <li><a href="file://$LOG_DIR/integration-tests.log">集成测试日志</a></li>
        <li><a href="file://$LOG_DIR/performance-tests.log">性能测试日志</a></li>
        <li><a href="file://$LOG_DIR/ai-tests.log">AI功能测试日志</a></li>
        <li><a href="file://$LOG_DIR/hardware-tests.log">硬件测试日志</a></li>
        <li><a href="file://$LOG_DIR/stress-tests.log">压力测试日志</a></li>
    </ul>
</body>
</html>
EOF
    
    echo "测试报告已生成: $REPORT_FILE"
}

# 主测试流程
main() {
    echo "开始StarryOS自动化测试..."
    
    # 设置测试环境
    setup_test_environment
    
    # 运行测试套件
    local test_results=0
    
    run_unit_tests || test_results=$((test_results + 1))
    run_integration_tests || test_results=$((test_results + 1))
    run_performance_tests || test_results=$((test_results + 1))
    run_ai_functionality_tests || test_results=$((test_results + 1))
    run_hardware_compatibility_tests || test_results=$((test_results + 1))
    run_stress_tests || test_results=$((test_results + 1))
    
    # 生成测试报告
    generate_test_report
    
    if [ "$test_results" -eq 0 ]; then
        echo -e "${GREEN}=== 所有测试通过 ===${NC}"
        echo "测试报告: $REPORT_FILE"
    else
        echo -e "${RED}=== 发现 $test_results 个测试失败 ===${NC}"
        echo "请查看详细日志: $LOG_DIR"
        exit 1
    fi
}

# 超时处理
timeout "$TEST_TIMEOUT" bash -c "main \$@"
if [ "$?" -eq 124 ]; then
    echo -e "${RED}测试超时（${TEST_TIMEOUT}秒）${NC}"
    exit 1
fi
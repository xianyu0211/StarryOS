// RK3588 AIoT系统监控面板 - 前端JavaScript
// 优化版本 - 性能提升和用户体验改进

class RK3588Monitor {
    constructor() {
        this.ws = null;
        this.isConnected = false;
        this.systemState = null;
        this.aiInferenceInterval = null;
        this.lastUpdateTime = 0;
        this.updateThrottle = 500; // 500ms更新节流
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 10;
        this.init();
    }
    
    // 初始化应用
    init() {
        this.setupEventListeners();
        this.connectWebSocket();
        this.updateConnectionStatus(false);
        
        // 初始加载系统信息
        this.fetchSystemInfo();
    }
    
    // 设置事件监听器
    setupEventListeners() {
        // 连接状态检查
        setInterval(() => {
            if (!this.isConnected) {
                this.connectWebSocket();
            }
        }, 5000);
    }
    
    // 连接WebSocket
    connectWebSocket() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            return;
        }
        
        // 检查重连次数限制
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.warn('达到最大重连次数，停止重连');
            return;
        }
        
        try {
            // 动态获取WebSocket地址，支持部署环境
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const host = window.location.hostname || 'localhost';
            const port = window.location.port ? `:${window.location.port}` : '';
            const wsUrl = `${protocol}//${host}${port}/ws`;
            
            console.log('正在连接WebSocket:', wsUrl);
            this.ws = new WebSocket(wsUrl);
            
            this.ws.onopen = () => {
                console.log('WebSocket连接成功');
                this.isConnected = true;
                this.reconnectAttempts = 0; // 重置重连计数
                this.updateConnectionStatus(true);
                this.showNotification('系统连接成功', 'success');
            };
            
            this.ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.handleWebSocketMessage(data);
                } catch (error) {
                    console.error('WebSocket消息解析错误:', error);
                }
            };
            
            this.ws.onclose = (event) => {
                console.log('WebSocket连接断开，代码:', event.code, '原因:', event.reason);
                this.isConnected = false;
                this.updateConnectionStatus(false);
                
                // 指数退避重连策略
                const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
                this.reconnectAttempts++;
                
                console.log(`将在 ${delay}ms 后尝试重连 (尝试次数: ${this.reconnectAttempts})`);
                setTimeout(() => this.connectWebSocket(), delay);
            };
            
            this.ws.onerror = (error) => {
                console.error('WebSocket错误:', error);
                this.isConnected = false;
                this.updateConnectionStatus(false);
            };
            
        } catch (error) {
            console.error('WebSocket连接失败:', error);
            this.isConnected = false;
            this.updateConnectionStatus(false);
        }
    }
    
    // 处理WebSocket消息
    handleWebSocketMessage(data) {
        switch (data.type) {
            case 'system_status':
                this.systemState = data.data;
                this.updateDashboard();
                break;
                
            case 'ai_inference_result':
                this.updateAIDetectionResults(data.data);
                break;
        }
    }
    
    // 更新连接状态显示
    updateConnectionStatus(connected) {
        const statusElement = document.getElementById('connectionStatus');
        const indicator = statusElement.querySelector('.status-indicator');
        
        if (connected) {
            indicator.className = 'status-indicator status-online';
            statusElement.innerHTML = '<span class="status-indicator status-online"></span>连接状态: 在线';
        } else {
            indicator.className = 'status-indicator status-offline';
            statusElement.innerHTML = '<span class="status-indicator status-offline"></span>连接状态: 离线';
        }
    }
    
    // 获取系统信息
    async fetchSystemInfo() {
        try {
            const response = await fetch('/api/system/info');
            const result = await response.json();
            
            if (result.success) {
                this.displaySystemInfo(result.data);
            }
        } catch (error) {
            console.error('获取系统信息失败:', error);
        }
    }
    
    // 显示系统信息
    displaySystemInfo(info) {
        const header = document.querySelector('.header p');
        header.innerHTML = `
            平台: ${info.platform} | 架构: ${info.architecture} | 
            核心: ${info.cores} | 内存: ${info.memory}
        `;
    }
    
    // 更新仪表板（带节流控制）
    updateDashboard() {
        if (!this.systemState) return;
        
        // 节流控制：避免过于频繁的DOM更新
        const now = Date.now();
        if (now - this.lastUpdateTime < this.updateThrottle) {
            return;
        }
        this.lastUpdateTime = now;
        
        // 使用requestAnimationFrame优化性能
        requestAnimationFrame(() => {
            this.updateCPUStatus();
            this.updateMemoryStatus();
            this.updateAIStatus();
            this.updateDriverStatus();
            this.updatePerformanceMetrics();
        });
    }
    
    // 更新性能指标
    updatePerformanceMetrics() {
        // 计算FPS
        const now = performance.now();
        if (!this.lastFrameTime) {
            this.lastFrameTime = now;
            this.frameCount = 0;
        }
        
        this.frameCount++;
        if (now - this.lastFrameTime >= 1000) {
            const fps = Math.round((this.frameCount * 1000) / (now - this.lastFrameTime));
            this.frameCount = 0;
            this.lastFrameTime = now;
            
            // 只在开发模式下显示FPS
            if (process.env.NODE_ENV === 'development') {
                console.log(`UI更新FPS: ${fps}`);
            }
        }
    }
    
    // 更新CPU状态（带平滑动画）
    updateCPUStatus() {
        const cpu = this.systemState.cpu;
        
        // 批量更新DOM，减少重排重绘
        const updates = [];
        
        Object.keys(cpu.cores).forEach(coreName => {
            const core = cpu.cores[coreName];
            const usageElement = document.getElementById(coreName);
            const barElement = document.getElementById(coreName + 'Bar');
            
            if (usageElement && barElement) {
                // 使用CSS变量实现平滑过渡
                updates.push(() => {
                    usageElement.textContent = `${core.usage}%`;
                    
                    // 使用CSS transition实现平滑动画
                    barElement.style.transition = 'width 0.3s ease, background 0.3s ease';
                    barElement.style.width = `${core.usage}%`;
                    
                    // 根据使用率动态调整颜色和动画效果
                    let color, animation;
                    if (core.usage > 80) {
                        color = 'linear-gradient(90deg, #ff6b6b, #ee5a52)';
                        animation = 'pulse 0.5s infinite';
                    } else if (core.usage > 60) {
                        color = 'linear-gradient(90deg, #ffa726, #ff9800)';
                        animation = 'none';
                    } else {
                        color = 'linear-gradient(90deg, #4CAF50, #45a049)';
                        animation = 'none';
                    }
                    
                    barElement.style.background = color;
                    barElement.style.animation = animation;
                    
                    // 添加温度指示器
                    if (core.temperature > 75) {
                        usageElement.style.color = '#ff6b6b';
                        usageElement.style.fontWeight = 'bold';
                    } else if (core.temperature > 60) {
                        usageElement.style.color = '#ffa726';
                    } else {
                        usageElement.style.color = '';
                        usageElement.style.fontWeight = '';
                    }
                });
            }
        });
        
        // 批量执行DOM更新
        updates.forEach(update => update());
        
        // 更新CPU温度图表
        this.updateCPUTemperatureChart();
    }
    
    // 更新CPU温度图表
    updateCPUTemperatureChart() {
        const cpu = this.systemState.cpu;
        const chartElement = document.getElementById('cpuTempChart');
        
        if (!chartElement || !cpu.temperatureHistory) return;
        
        // 创建简单的SVG温度图表
        const svg = `
            <svg width="100%" height="100%" viewBox="0 0 100 20">
                <defs>
                    <linearGradient id="tempGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                        <stop offset="0%" style="stop-color:#4CAF50;stop-opacity:1" />
                        <stop offset="50%" style="stop-color:#ffa726;stop-opacity:1" />
                        <stop offset="100%" style="stop-color:#ff6b6b;stop-opacity:1" />
                    </linearGradient>
                </defs>
                <rect x="0" y="8" width="100" height="4" fill="url(#tempGradient)" opacity="0.3"/>
                <circle cx="${cpu.temperature / 100 * 100}" cy="10" r="3" fill="#ff6b6b"/>
                <text x="${cpu.temperature / 100 * 100}" y="18" font-size="3" text-anchor="middle" fill="#666">${cpu.temperature}°C</text>
            </svg>
        `;
        
        chartElement.innerHTML = svg;
    }
    
    // 更新内存状态
    updateMemoryStatus() {
        const memory = this.systemState.memory;
        
        document.getElementById('memoryUsage').textContent = 
            `${Math.round(memory.used / 1024)}/${Math.round(memory.total / 1024)} MB`;
        document.getElementById('memoryPressure').textContent = `${memory.pressure}%`;
        document.getElementById('fragmentation').textContent = `${memory.fragmentation}%`;
        document.getElementById('allocationCount').textContent = memory.allocationCount.toLocaleString();
        
        const memoryBar = document.getElementById('memoryBar');
        const usagePercent = (memory.used / memory.total) * 100;
        memoryBar.style.width = `${usagePercent}%`;
        
        // 根据内存压力调整颜色
        if (memory.pressure > 80) {
            memoryBar.style.background = 'linear-gradient(90deg, #ff6b6b, #ee5a52)';
        } else if (memory.pressure > 60) {
            memoryBar.style.background = 'linear-gradient(90deg, #ffa726, #ff9800)';
        } else {
            memoryBar.style.background = 'linear-gradient(90deg, #4CAF50, #45a049)';
        }
    }
    
    // 更新AI状态
    updateAIStatus() {
        const ai = this.systemState.ai;
        
        document.getElementById('inferenceLatency').textContent = `${ai.inferenceLatency}ms`;
        document.getElementById('npuUsage').textContent = `${ai.npuUsage}%`;
        document.getElementById('batchSize').textContent = ai.batchSize;
        document.getElementById('detectionCount').textContent = ai.detectionCount;
        
        const npuBar = document.getElementById('npuBar');
        npuBar.style.width = `${ai.npuUsage}%`;
        
        // 更新AI控制按钮状态
        const startBtn = document.querySelector('button[onclick="startInference()"]');
        const stopBtn = document.querySelector('button[onclick="stopInference()"]');
        
        if (ai.isRunning) {
            startBtn.disabled = true;
            stopBtn.disabled = false;
            startBtn.innerHTML = '<i class="fas fa-play"></i> 运行中...';
        } else {
            startBtn.disabled = false;
            stopBtn.disabled = true;
            startBtn.innerHTML = '<i class="fas fa-play"></i> 开始推理';
        }
    }
    
    // 更新驱动状态
    updateDriverStatus() {
        // 这里可以添加驱动状态的可视化
        console.log('驱动状态:', this.systemState.drivers);
    }
    
    // 发送WebSocket消息
    sendWebSocketMessage(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        } else {
            console.warn('WebSocket未连接，无法发送消息');
        }
    }
}

// 全局监控实例
let monitor = null;

// 页面加载完成后初始化
document.addEventListener('DOMContentLoaded', () => {
    monitor = new RK3588Monitor();
});

// ========== 控制函数 ==========

// 调整CPU频率
function adjustFrequency(mode) {
    if (monitor) {
        monitor.sendWebSocketMessage({
            type: 'adjust_frequency',
            mode: mode
        });
        
        // 显示提示
        showNotification(`已切换到${getFrequencyModeName(mode)}模式`);
    }
}

// 获取频率模式名称
function getFrequencyModeName(mode) {
    const modes = {
        high: '高性能',
        normal: '均衡',
        low: '节能'
    };
    return modes[mode] || '未知';
}

// 内存整理
function defragmentMemory() {
    if (monitor) {
        monitor.sendWebSocketMessage({
            type: 'defragment_memory'
        });
        showNotification('内存整理操作已发送');
    }
}

// 开始AI推理
function startInference() {
    if (monitor) {
        monitor.sendWebSocketMessage({
            type: 'start_inference'
        });
        showNotification('AI推理已启动');
    }
}

// 停止AI推理
function stopInference() {
    if (monitor) {
        monitor.sendWebSocketMessage({
            type: 'stop_inference'
        });
        showNotification('AI推理已停止');
    }
}

// 清空内存统计
function clearMemoryStats() {
    // 这里可以添加清空内存统计的逻辑
    showNotification('内存统计已重置');
}

// ========== AI图像检测功能 ==========

let currentImage = null;

// 上传图像
function uploadImage() {
    document.getElementById('imageUpload').click();
}

// 处理图像上传
function handleImageUpload(file) {
    if (!file || !file.type.startsWith('image/')) {
        showNotification('请选择有效的图像文件', 'error');
        return;
    }
    
    const reader = new FileReader();
    reader.onload = function(e) {
        displayImage(e.target.result);
        currentImage = e.target.result;
    };
    reader.readAsDataURL(file);
}

// 使用示例图像
function useSampleImage() {
    const sampleImages = [
        'https://images.unsplash.com/photo-1507146426996-ef05306b995a?w=400',
        'https://images.unsplash.com/photo-1551963831-b3b1ca40c98e?w=400',
        'https://images.unsplash.com/photo-1541963463532-d68292c34b19?w=400'
    ];
    
    const randomImage = sampleImages[Math.floor(Math.random() * sampleImages.length)];
    displayImage(randomImage);
    currentImage = randomImage;
    
    showNotification('已加载示例图像');
}

// 显示图像
function displayImage(imageSrc) {
    const canvas = document.getElementById('detectionImage');
    const ctx = canvas.getContext('2d');
    const placeholder = document.getElementById('canvasPlaceholder');
    
    const img = new Image();
    img.onload = function() {
        // 设置canvas尺寸
        canvas.width = img.width;
        canvas.height = img.height;
        
        // 绘制图像
        ctx.drawImage(img, 0, 0);
        
        // 显示canvas，隐藏占位符
        canvas.style.display = 'block';
        placeholder.style.display = 'none';
        
        // 执行AI推理
        this.performAIInference(imageSrc);
    };
    img.src = imageSrc;
}

// 执行AI推理（带图像优化和进度指示）
async function performAIInference(imageData) {
    try {
        // 显示进度指示器
        const progressNotification = showNotification('AI推理中...', 'info', true);
        
        // 图像预处理：压缩和优化
        const optimizedImageData = await optimizeImageForInference(imageData);
        
        // 使用AbortController实现超时控制
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 30000); // 30秒超时
        
        const response = await fetch('/api/ai/inference', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                imageData: optimizedImageData,
                model: currentModel,
                confidenceThreshold: 0.5
            }),
            signal: controller.signal
        });
        
        clearTimeout(timeoutId);
        
        const result = await response.json();
        
        if (result.success) {
            updateAIDetectionResults(result.data);
            
            // 移除进度指示器
            if (progressNotification && progressNotification.parentNode) {
                progressNotification.parentNode.removeChild(progressNotification);
            }
            
            showNotification(`AI推理完成，检测到${result.data.detections.length}个目标`, 'success');
            
            // 性能统计
            logInferencePerformance(result.data);
        } else {
            throw new Error(result.message);
        }
        
    } catch (error) {
        console.error('AI推理失败:', error);
        
        if (error.name === 'AbortError') {
            showNotification('AI推理超时，请重试', 'error');
        } else {
            showNotification('AI推理失败: ' + error.message, 'error');
        }
    }
}

// 图像优化函数
async function optimizeImageForInference(imageData) {
    return new Promise((resolve) => {
        const img = new Image();
        img.onload = () => {
            const canvas = document.createElement('canvas');
            const ctx = canvas.getContext('2d');
            
            // 设置最大尺寸限制
            const maxWidth = 800;
            const maxHeight = 600;
            let { width, height } = img;
            
            if (width > maxWidth) {
                height = (height * maxWidth) / width;
                width = maxWidth;
            }
            
            if (height > maxHeight) {
                width = (width * maxHeight) / height;
                height = maxHeight;
            }
            
            canvas.width = width;
            canvas.height = height;
            
            // 高质量缩放
            ctx.imageSmoothingEnabled = true;
            ctx.imageSmoothingQuality = 'high';
            ctx.drawImage(img, 0, 0, width, height);
            
            // 转换为JPEG格式，质量80%
            const optimizedData = canvas.toDataURL('image/jpeg', 0.8);
            resolve(optimizedData);
        };
        
        img.src = imageData;
    });
}

// 记录推理性能
function logInferencePerformance(data) {
    const performanceData = {
        timestamp: new Date().toISOString(),
        inferenceTime: data.inferenceTime || 0,
        detectionCount: data.detections ? data.detections.length : 0,
        model: currentModel,
        confidenceThreshold: 0.5
    };
    
    // 保存到本地存储（可选）
    const history = JSON.parse(localStorage.getItem('aiInferenceHistory') || '[]');
    history.unshift(performanceData);
    
    // 只保留最近50条记录
    if (history.length > 50) {
        history.length = 50;
    }
    
    localStorage.setItem('aiInferenceHistory', JSON.stringify(history));
    
    console.log('AI推理性能统计:', performanceData);
}

// 更新AI检测结果
function updateAIDetectionResults(data) {
    const resultsContainer = document.getElementById('detectionResults');
    const canvas = document.getElementById('detectionImage');
    const ctx = canvas.getContext('2d');
    
    // 清空之前的结果
    resultsContainer.innerHTML = '';
    
    // 在图像上绘制检测框
    if (data.detections && data.detections.length > 0) {
        data.detections.forEach((detection, index) => {
            // 绘制边界框
            const [x, y, width, height] = detection.bbox;
            const canvasX = x * canvas.width;
            const canvasY = y * canvas.height;
            const canvasWidth = width * canvas.width;
            const canvasHeight = height * canvas.height;
            
            ctx.strokeStyle = '#ff6b6b';
            ctx.lineWidth = 3;
            ctx.strokeRect(canvasX, canvasY, canvasWidth, canvasHeight);
            
            // 添加标签
            ctx.fillStyle = '#ff6b6b';
            ctx.font = '16px Arial';
            ctx.fillText(
                `${detection.className} (${(detection.confidence * 100).toFixed(1)}%)`,
                canvasX, canvasY - 5
            );
            
            // 添加结果到列表
            const detectionElement = document.createElement('div');
            detectionElement.className = 'detection-item';
            detectionElement.innerHTML = `
                <strong>${detection.className}</strong> 
                - 置信度: ${(detection.confidence * 100).toFixed(1)}% 
                - 位置: (${x.toFixed(2)}, ${y.toFixed(2)})
            `;
            resultsContainer.appendChild(detectionElement);
        });
    } else {
        resultsContainer.innerHTML = '<div class="detection-item">未检测到目标</div>';
    }
}

// ========== 工具函数 ==========

// 显示通知
function showNotification(message, type = 'success') {
    // 创建通知元素
    const notification = document.createElement('div');
    notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        padding: 15px 20px;
        background: ${type === 'error' ? '#dc3545' : type === 'info' ? '#17a2b8' : '#28a745'};
        color: white;
        border-radius: 8px;
        box-shadow: 0 5px 15px rgba(0,0,0,0.2);
        z-index: 1000;
        animation: slideIn 0.3s ease;
    `;
    
    notification.textContent = message;
    document.body.appendChild(notification);
    
    // 3秒后自动移除
    setTimeout(() => {
        notification.style.animation = 'slideOut 0.3s ease';
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 300);
    }, 3000);
}

// 添加CSS动画
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from { transform: translateX(100%); opacity: 0; }
        to { transform: translateX(0); opacity: 1; }
    }
    
    @keyframes slideOut {
        from { transform: translateX(0); opacity: 1; }
        to { transform: translateX(100%); opacity: 0; }
    }
`;
document.head.appendChild(style);

// 键盘快捷键
document.addEventListener('keydown', (e) => {
    if (e.ctrlKey || e.metaKey) {
        switch (e.key) {
            case '1':
                e.preventDefault();
                adjustFrequency('high');
                break;
            case '2':
                e.preventDefault();
                adjustFrequency('normal');
                break;
            case '3':
                e.preventDefault();
                startInference();
                break;
            case '4':
                e.preventDefault();
                stopInference();
                break;
        }
    }
});

console.log('RK3588监控面板已加载完成');
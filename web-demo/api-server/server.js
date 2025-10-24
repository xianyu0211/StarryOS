const express = require('express');
const cors = require('cors');
const WebSocket = require('ws');
const path = require('path');

const app = express();
const PORT = 8080;

// 中间件
app.use(cors());
app.use(express.json());
app.use(express.static(path.join(__dirname, '../dist')));

// 模拟RK3588系统状态数据
let systemState = {
    cpu: {
        cores: {
            'A76-0': { usage: 15, frequency: 1800, temperature: 45 },
            'A76-1': { usage: 22, frequency: 1800, temperature: 47 },
            'A55-0': { usage: 8, frequency: 1400, temperature: 42 },
            'A55-1': { usage: 5, frequency: 1400, temperature: 41 }
        },
        loadAverage: [0.15, 0.18, 0.12],
        frequencyMode: 'normal'
    },
    memory: {
        total: 8192, // 8GB
        used: 2048,
        pressure: 25,
        fragmentation: 12,
        allocationCount: 1567
    },
    ai: {
        npuUsage: 35,
        inferenceLatency: 45,
        batchSize: 1,
        detectionCount: 0,
        isRunning: false
    },
    drivers: {
        usb: { status: 'connected', speed: '3.0', transfers: 1234 },
        mipi: { status: 'active', frames: 567, errors: 0 },
        dma: { status: 'idle', transfers: 890, zeroCopy: true }
    }
};

// WebSocket服务器
const wss = new WebSocket.Server({ port: 8081 });

// WebSocket连接处理
wss.on('connection', (ws) => {
    console.log('WebSocket客户端连接');
    
    // 发送初始状态
    ws.send(JSON.stringify({
        type: 'system_status',
        data: systemState
    }));
    
    ws.on('message', (message) => {
        try {
            const data = JSON.parse(message);
            handleWebSocketMessage(ws, data);
        } catch (error) {
            console.error('WebSocket消息解析错误:', error);
        }
    });
    
    ws.on('close', () => {
        console.log('WebSocket客户端断开连接');
    });
});

// WebSocket消息处理
function handleWebSocketMessage(ws, data) {
    switch (data.type) {
        case 'start_inference':
            systemState.ai.isRunning = true;
            startAISimulation();
            break;
            
        case 'stop_inference':
            systemState.ai.isRunning = false;
            break;
            
        case 'adjust_frequency':
            adjustFrequency(data.mode);
            break;
            
        case 'defragment_memory':
            defragmentMemory();
            break;
    }
    
    // 广播更新
    broadcastSystemStatus();
}

// 广播系统状态给所有客户端
function broadcastSystemStatus() {
    const message = JSON.stringify({
        type: 'system_status',
        data: systemState
    });
    
    wss.clients.forEach(client => {
        if (client.readyState === WebSocket.OPEN) {
            client.send(message);
        }
    });
}

// AI推理模拟
function startAISimulation() {
    if (!systemState.ai.isRunning) return;
    
    const interval = setInterval(() => {
        if (!systemState.ai.isRunning) {
            clearInterval(interval);
            return;
        }
        
        // 模拟AI推理过程
        systemState.ai.npuUsage = Math.min(95, systemState.ai.npuUsage + Math.random() * 10);
        systemState.ai.inferenceLatency = 30 + Math.random() * 40;
        systemState.ai.detectionCount = Math.floor(Math.random() * 10);
        
        // 更新CPU使用率
        Object.keys(systemState.cpu.cores).forEach(core => {
            systemState.cpu.cores[core].usage = Math.min(95, 
                systemState.cpu.cores[core].usage + Math.random() * 5);
        });
        
        broadcastSystemStatus();
    }, 1000);
}

// 频率调整
function adjustFrequency(mode) {
    const frequencies = {
        high: { A76: 2400, A55: 1800 },
        normal: { A76: 1800, A55: 1400 },
        low: { A76: 1200, A55: 1000 }
    };
    
    const freq = frequencies[mode] || frequencies.normal;
    systemState.cpu.frequencyMode = mode;
    
    // 更新A76核心频率
    ['A76-0', 'A76-1'].forEach(core => {
        systemState.cpu.cores[core].frequency = freq.A76;
    });
    
    // 更新A55核心频率
    ['A55-0', 'A55-1'].forEach(core => {
        systemState.cpu.cores[core].frequency = freq.A55;
    });
}

// 内存整理
function defragmentMemory() {
    systemState.memory.fragmentation = Math.max(5, systemState.memory.fragmentation - 8);
    systemState.memory.pressure = Math.max(10, systemState.memory.pressure - 5);
}

// REST API路由

// 获取系统状态
app.get('/api/system/status', (req, res) => {
    res.json({
        success: true,
        data: systemState
    });
});

// 获取CPU状态
app.get('/api/cpu/status', (req, res) => {
    res.json({
        success: true,
        data: systemState.cpu
    });
});

// 获取内存状态
app.get('/api/memory/status', (req, res) => {
    res.json({
        success: true,
        data: systemState.memory
    });
});

// 获取AI状态
app.get('/api/ai/status', (req, res) => {
    res.json({
        success: true,
        data: systemState.ai
    });
});

// 控制AI推理
app.post('/api/ai/control', (req, res) => {
    const { action } = req.body;
    
    if (action === 'start') {
        systemState.ai.isRunning = true;
        startAISimulation();
    } else if (action === 'stop') {
        systemState.ai.isRunning = false;
    }
    
    res.json({
        success: true,
        message: `AI推理${action === 'start' ? '启动' : '停止'}成功`
    });
});

// 调整CPU频率
app.post('/api/cpu/frequency', (req, res) => {
    const { mode } = req.body;
    
    if (['high', 'normal', 'low'].includes(mode)) {
        adjustFrequency(mode);
        res.json({
            success: true,
            message: `频率模式已切换到${mode}`
        });
    } else {
        res.status(400).json({
            success: false,
            message: '无效的频率模式'
        });
    }
});

// 内存整理
app.post('/api/memory/defragment', (req, res) => {
    defragmentMemory();
    res.json({
        success: true,
        message: '内存整理完成'
    });
});

// AI图像推理
app.post('/api/ai/inference', async (req, res) => {
    const { imageData } = req.body;
    
    try {
        // 模拟AI推理过程
        const inferenceResult = await simulateAIInference(imageData);
        
        res.json({
            success: true,
            data: inferenceResult
        });
    } catch (error) {
        res.status(500).json({
            success: false,
            message: 'AI推理失败'
        });
    }
});

// 模拟AI推理
async function simulateAIInference(imageData) {
    // 模拟处理延迟
    await new Promise(resolve => setTimeout(resolve, 500 + Math.random() * 1000));
    
    // 生成模拟检测结果
    const detections = [];
    const classNames = ['person', 'car', 'bicycle', 'dog', 'cat', 'tree', 'building'];
    const detectionCount = Math.floor(Math.random() * 8) + 1;
    
    for (let i = 0; i < detectionCount; i++) {
        detections.push({
            classId: Math.floor(Math.random() * classNames.length),
            className: classNames[Math.floor(Math.random() * classNames.length)],
            confidence: (0.7 + Math.random() * 0.3).toFixed(2),
            bbox: [
                Math.random() * 0.8,
                Math.random() * 0.8,
                Math.random() * 0.2 + 0.1,
                Math.random() * 0.2 + 0.1
            ]
        });
    }
    
    return {
        latency: 45 + Math.random() * 30,
        detections: detections,
        timestamp: new Date().toISOString()
    };
}

// 获取驱动状态
app.get('/api/drivers/status', (req, res) => {
    res.json({
        success: true,
        data: systemState.drivers
    });
});

// 系统信息
app.get('/api/system/info', (req, res) => {
    res.json({
        success: true,
        data: {
            platform: 'RK3588',
            architecture: 'ARM64',
            cores: '4x Cortex-A76 + 4x Cortex-A55',
            npu: '6TOPS NPU',
            memory: '8GB LPDDR4',
            version: 'StarryOS v1.0',
            uptime: Math.floor(process.uptime())
        }
    });
});

// 错误处理中间件
app.use((error, req, res, next) => {
    console.error('API错误:', error);
    res.status(500).json({
        success: false,
        message: '服务器内部错误'
    });
});

// 404处理
app.use('*', (req, res) => {
    res.status(404).json({
        success: false,
        message: '接口不存在'
    });
});

// 启动服务器
app.listen(PORT, () => {
    console.log(`RK3588 Web API服务器运行在 http://localhost:${PORT}`);
    console.log(`WebSocket服务器运行在 ws://localhost:8081`);
    console.log('系统模拟数据已初始化');
    
    // 启动系统状态更新循环
    setInterval(() => {
        // 随机更新系统状态
        Object.keys(systemState.cpu.cores).forEach(core => {
            if (!systemState.ai.isRunning) {
                systemState.cpu.cores[core].usage = Math.max(5, 
                    systemState.cpu.cores[core].usage + (Math.random() - 0.5) * 10);
            }
            systemState.cpu.cores[core].temperature = 40 + Math.random() * 10;
        });
        
        systemState.memory.used = Math.max(512, 
            Math.min(systemState.memory.total - 512, 
                systemState.memory.used + (Math.random() - 0.5) * 200));
        
        systemState.memory.pressure = Math.max(10, 
            Math.min(90, systemState.memory.pressure + (Math.random() - 0.5) * 5));
        
        broadcastSystemStatus();
    }, 3000);
});
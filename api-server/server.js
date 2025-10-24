const express = require('express');
const cors = require('cors');
const WebSocket = require('ws');
const path = require('path');

const app = express();
const PORT = process.env.PORT || 8080;

// 中间件
app.use(cors());
app.use(express.json({ limit: '10mb' }));
app.use(express.static(path.join(__dirname, '..')));

// 模拟系统状态数据
let systemState = {
    cpu: {
        cores: {
            coreA76_0: { usage: Math.random() * 100, frequency: 2400 },
            coreA76_1: { usage: Math.random() * 100, frequency: 2400 },
            coreA55_0: { usage: Math.random() * 30, frequency: 1800 },
            coreA55_1: { usage: Math.random() * 30, frequency: 1800 }
        },
        temperature: 45 + Math.random() * 20
    },
    memory: {
        total: 8192, // 8GB
        used: Math.random() * 4096 + 1024,
        pressure: Math.random() * 100,
        fragmentation: Math.random() * 30,
        allocationCount: Math.floor(Math.random() * 10000)
    },
    ai: {
        inferenceLatency: Math.random() * 50 + 10,
        npuUsage: Math.random() * 100,
        batchSize: Math.floor(Math.random() * 8) + 1,
        detectionCount: Math.floor(Math.random() * 10),
        isRunning: false
    },
    drivers: {
        audio: 'active',
        camera: 'active',
        network: 'active',
        storage: 'active'
    }
};

// API路由
app.get('/api/health', (req, res) => {
    res.json({ 
        status: 'healthy', 
        timestamp: new Date().toISOString(),
        version: '1.0.0'
    });
});

app.get('/api/system/info', (req, res) => {
    res.json({
        success: true,
        data: {
            platform: 'RK3588 AIoT System',
            architecture: 'aarch64',
            cores: 4,
            memory: '8GB LPDDR4',
            npu: '6TOPS',
            version: 'StarryOS v1.0.0'
        }
    });
});

app.post('/api/ai/inference', (req, res) => {
    const { imageData } = req.body;
    
    // 模拟AI推理结果
    const detections = [
        {
            className: 'person',
            confidence: 0.85 + Math.random() * 0.1,
            bbox: [0.1 + Math.random() * 0.1, 0.2 + Math.random() * 0.1, 0.3, 0.4]
        },
        {
            className: 'car',
            confidence: 0.75 + Math.random() * 0.1,
            bbox: [0.5 + Math.random() * 0.1, 0.3 + Math.random() * 0.1, 0.2, 0.3]
        }
    ];
    
    // 随机过滤一些检测结果
    const filteredDetections = detections.filter(d => d.confidence > 0.7);
    
    res.json({
        success: true,
        data: {
            detections: filteredDetections,
            inferenceTime: Math.random() * 100 + 50,
            model: 'YOLOv8n-RK3588'
        }
    });
});

// WebSocket服务器
const wss = new WebSocket.Server({ 
    port: 8081,
    path: '/ws'
});

wss.on('connection', (ws) => {
    console.log('WebSocket客户端连接成功');
    
    // 发送初始系统状态
    ws.send(JSON.stringify({
        type: 'system_status',
        data: systemState
    }));
    
    // 定期更新系统状态
    const interval = setInterval(() => {
        // 更新系统状态数据
        Object.keys(systemState.cpu.cores).forEach(core => {
            systemState.cpu.cores[core].usage = Math.random() * 100;
        });
        
        systemState.memory.used = Math.random() * 4096 + 1024;
        systemState.memory.pressure = Math.random() * 100;
        systemState.ai.inferenceLatency = Math.random() * 50 + 10;
        systemState.ai.npuUsage = Math.random() * 100;
        
        if (systemState.ai.isRunning) {
            systemState.ai.detectionCount = Math.floor(Math.random() * 10);
        }
        
        ws.send(JSON.stringify({
            type: 'system_status',
            data: systemState
        }));
    }, 2000);
    
    ws.on('message', (message) => {
        try {
            const data = JSON.parse(message);
            
            switch (data.type) {
                case 'adjust_frequency':
                    // 模拟调整频率
                    console.log(`调整CPU频率模式: ${data.mode}`);
                    ws.send(JSON.stringify({
                        type: 'frequency_adjusted',
                        mode: data.mode
                    }));
                    break;
                    
                case 'defragment_memory':
                    // 模拟内存整理
                    systemState.memory.fragmentation = Math.random() * 10;
                    console.log('执行内存整理');
                    break;
                    
                case 'start_inference':
                    systemState.ai.isRunning = true;
                    console.log('启动AI推理');
                    break;
                    
                case 'stop_inference':
                    systemState.ai.isRunning = false;
                    systemState.ai.detectionCount = 0;
                    console.log('停止AI推理');
                    break;
            }
        } catch (error) {
            console.error('WebSocket消息处理错误:', error);
        }
    });
    
    ws.on('close', () => {
        console.log('WebSocket客户端断开连接');
        clearInterval(interval);
    });
});

// 静态文件服务
app.get('/', (req, res) => {
    res.sendFile(path.join(__dirname, '..', 'index.html'));
});

// 启动服务器
const server = app.listen(PORT, '0.0.0.0', () => {
    console.log(`🚀 StarryOS Web Demo 服务器启动成功`);
    console.log(`📍 本地访问: http://localhost:${PORT}`);
    console.log(`🌐 网络访问: http://0.0.0.0:${PORT}`);
    console.log(`🔌 WebSocket服务运行在端口 8081`);
    console.log(`💡 系统信息API: http://localhost:${PORT}/api/system/info`);
});

// 优雅关闭
process.on('SIGTERM', () => {
    console.log('收到SIGTERM信号，正在关闭服务器...');
    server.close(() => {
        console.log('服务器已关闭');
        process.exit(0);
    });
});
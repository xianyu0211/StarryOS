# RK3588 AIoT系统Web演示应用
包含了实际的演示地址：http://d037cdb9d5814a77ace383a80cfd604b.codebuddy.cloudstudio.run
这是一个为RK3588嵌入式AIoT系统构建的Web演示应用，通过REST API和WebSocket实时监控系统状态，并展示AI推理功能。

## 功能特性

### 🚀 实时系统监控
- **CPU状态监控**: 实时显示4个Cortex-A76和4个Cortex-A55核心的使用率、频率和温度
- **内存管理**: 监控内存使用情况、内存压力和碎片化程度
- **AI推理引擎**: 实时显示NPU使用率、推理延迟和检测结果
- **驱动状态**: USB 3.0、MIPI-CSI、DMA等驱动状态监控

### 🤖 AI目标检测
- **图像上传**: 支持上传本地图像进行AI目标检测
- **示例图像**: 提供示例图像快速测试AI功能
- **实时检测**: 在图像上实时显示检测框和置信度
- **多目标识别**: 支持同时检测多个目标

### ⚡ 系统控制
- **频率调节**: 支持高性能、均衡、节能三种CPU频率模式
- **内存整理**: 一键内存碎片整理功能
- **AI控制**: 启动/停止AI推理引擎

## 技术架构

### 前端技术栈
- **Vue 3**: 现代化前端框架
- **Vite**: 快速构建工具
- **WebSocket**: 实时双向通信
- **Canvas API**: 图像处理和检测框绘制

### 后端技术栈
- **Node.js + Express**: REST API服务器
- **WebSocket**: 实时数据推送
- **CORS**: 跨域资源共享
- **模拟数据**: 完整的RK3588系统状态模拟

## 快速开始

### 环境要求
- Node.js 16.0+
- npm 或 yarn

### 安装依赖
```bash
cd web-demo
npm install
```

### 启动开发服务器
```bash
# 启动前端开发服务器 (端口3000)
npm run dev

# 启动API服务器 (端口8080)
npm run api
```

### 访问应用
打开浏览器访问: http://localhost:3000

## 项目结构

```
web-demo/
├── index.html              # 主页面
├── app.js                  # 前端JavaScript
├── package.json            # 项目配置
├── vite.config.js          # Vite配置
├── api-server/
│   └── server.js           # REST API服务器
└── README.md               # 项目文档
```

## API接口文档

### 系统状态接口

#### GET /api/system/status
获取完整的系统状态信息

**响应示例:**
```json
{
  "success": true,
  "data": {
    "cpu": {
      "cores": {
        "A76-0": { "usage": 15, "frequency": 1800, "temperature": 45 },
        "A76-1": { "usage": 22, "frequency": 1800, "temperature": 47 },
        "A55-0": { "usage": 8, "frequency": 1400, "temperature": 42 },
        "A55-1": { "usage": 5, "frequency": 1400, "temperature": 41 }
      },
      "loadAverage": [0.15, 0.18, 0.12],
      "frequencyMode": "normal"
    },
    "memory": {
      "total": 8192,
      "used": 2048,
      "pressure": 25,
      "fragmentation": 12,
      "allocationCount": 1567
    },
    "ai": {
      "npuUsage": 35,
      "inferenceLatency": 45,
      "batchSize": 1,
      "detectionCount": 0,
      "isRunning": false
    }
  }
}
```

#### GET /api/cpu/status
获取CPU状态信息

#### GET /api/memory/status
获取内存状态信息

#### GET /api/ai/status
获取AI推理状态信息

### 控制接口

#### POST /api/ai/control
控制AI推理引擎

**请求体:**
```json
{
  "action": "start"  // 或 "stop"
}
```

#### POST /api/cpu/frequency
调整CPU频率模式

**请求体:**
```json
{
  "mode": "high"  // "high", "normal", "low"
}
```

#### POST /api/memory/defragment
执行内存碎片整理

#### POST /api/ai/inference
执行AI图像推理

**请求体:**
```json
{
  "imageData": "base64编码的图像数据"
}
```

### WebSocket接口

连接地址: `ws://localhost:8081`

#### 消息类型
- `system_status`: 系统状态更新
- `ai_inference_result`: AI推理结果

#### 发送消息
- `start_inference`: 启动AI推理
- `stop_inference`: 停止AI推理
- `adjust_frequency`: 调整频率模式
- `defragment_memory`: 内存整理

## 部署到CloudStudio

### 构建项目
```bash
npm run build
```

### 部署配置
项目已配置为可直接部署到CloudStudio的Web应用。

## 开发指南

### 添加新的监控指标
1. 在 `api-server/server.js` 中更新 `systemState` 结构
2. 在 `app.js` 中添加对应的UI更新逻辑
3. 在 `index.html` 中添加对应的HTML元素

### 自定义AI推理逻辑
修改 `simulateAIInference` 函数来实现真实的AI推理逻辑。

### 扩展驱动支持
在 `systemState.drivers` 中添加新的驱动状态信息。

## 性能优化

- **WebSocket连接**: 使用WebSocket实现实时数据推送，减少HTTP请求
- **图像压缩**: 上传图像时自动压缩以减少传输数据量
- **懒加载**: 按需加载系统状态信息
- **缓存策略**: 合理使用浏览器缓存

## 故障排除

### WebSocket连接失败
- 检查API服务器是否正常运行在8080端口
- 检查防火墙设置是否允许WebSocket连接

### 图像上传失败
- 检查图像文件大小是否超过限制
- 确认浏览器支持File API

### API请求失败
- 检查CORS配置是否正确
- 确认服务器端口未被占用

## 许可证

本项目基于MIT许可证开源。

## 贡献

欢迎提交Issue和Pull Request来改进这个项目！

## 联系方式

如有问题或建议，请通过以下方式联系：
- 提交GitHub Issue
- 发送邮件到项目维护者
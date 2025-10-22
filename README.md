# StarryOS - Rust内核工程项目

基于Rust语言开发的嵌入式AIoT操作系统内核，专为香橙派AIpro 20T等高端算力SoC芯片设计。

## 项目概述

StarryOS是一个专为AIoT场景设计的轻量级Rust内核，具备以下特性：

- **安全可靠**: 基于Rust的所有权系统和内存安全特性
- **高性能**: 支持国产SoC芯片的AI加速单元(NPU/TPU/BPU)
- **多功能**: 集成多种外设驱动和AI应用框架
- **易扩展**: 模块化设计，支持快速功能扩展

## 功能特性

### 内核核心功能
- 内存管理模块 (基于Rust所有权系统)
- 多任务进程调度器
- 文件系统支持 (FAT32/EXT4)
- 轻量级网络协议栈

### 外设驱动支持
**环境感知类**:
- 温湿度传感器 (DHT22/AM2302)
- 光线传感器 (BH1750) 
- 运动传感器 (MPU6050)

**通信交互类**:
- LoRa无线通信模块
- 蓝牙5.0 (BLE支持)
- CAN总线通信

**操作辅助类**:
- 舵机控制 (PWM)
- LCD显示屏驱动
- 蜂鸣器/语音提示

### AI加速功能
- 国产SoC AI加速单元适配 (全志V851S、瑞芯微RK3588等)
- Yolo-v8目标识别模型优化移植
- 硬件加速推理接口

## 项目结构

```
RK/
├── kernel/                 # 内核核心代码
├── drivers/               # 外设驱动模块
├── ai/                   # AI应用模块
├── apps/                 # 应用层代码
├── docs/                 # 开发文档
├── tests/                # 测试代码
└── ci/                   # 持续集成
```

## 快速开始

### 环境要求

- Rust工具链 (nightly版本)
- aarch64-unknown-none目标
- 香橙派AIpro 20T开发板
- 交叉编译工具链

### 构建说明

```bash
# 安装Rust工具链
rustup target add aarch64-unknown-none

# 构建内核
cd kernel
cargo build --target aarch64-unknown-none

# 构建完整系统
make all
```

### 部署运行

```bash
# 烧录到开发板
make flash

# 启动系统
make boot
```

## 开发文档

- [设计文档](./docs/design/)
- [API参考](./docs/api/)
- [部署指南](./docs/deployment/)

## 性能测试

详细的性能测试报告请参考 [测试文档](./tests/performance/)。

## 贡献指南

欢迎提交Issue和Pull Request来改进项目。

## 许可证

本项目采用MIT许可证。

## 联系方式

如有问题请联系项目维护团队。
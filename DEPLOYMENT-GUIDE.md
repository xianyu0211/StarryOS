# StarryOS 部署指南

## 概述

本文档提供StarryOS系统在RK3588等AIoT开发板上的完整部署指南。

## 系统要求

### 硬件要求
- **开发板**: RK3588系列（Orange Pi 5, Radxa Rock 5B, Firefly等）
- **内存**: 最小2GB，推荐8GB LPDDR4
- **存储**: 最小8GB，推荐32GB eMMC或SD卡
- **外设**: 
  - 麦克风（用于语音交互）
  - 摄像头（用于视觉识别）
  - 扬声器（用于语音输出）
  - 网络连接（WiFi/以太网）

### 软件要求
- **主机系统**: Ubuntu 20.04+ 或 macOS 12+
- **Rust工具链**: 1.70.0+
- **交叉编译工具**: aarch64-linux-gnu-gcc
- **部署工具**: dd, parted, mkfs

## 快速部署

### 方法一：使用自动化脚本（推荐）

```bash
# 1. 克隆项目
git clone https://github.com/your-org/starryos-rk3588.git
cd starryos-rk3588

# 2. 初始化子模块（如果有）
git submodule update --init --recursive

# 3. 运行系统验证
./scripts/system-integration.sh

# 4. 创建部署镜像
make deploy

# 5. 部署到设备（替换/dev/sdX为实际设备）
sudo ./scripts/deploy-voice-ai.sh /dev/sdX

# 6. 验证部署
./scripts/verify-deployment.sh
```

### 方法二：手动部署

```bash
# 1. 构建系统
make build

# 2. 创建SD卡分区
sudo parted /dev/sdX mklabel gpt
sudo parted /dev/sdX mkpart primary fat32 1MiB 256MiB
sudo parted /dev/sdX mkpart primary ext4 256MiB 100%
sudo parted /dev/sdX set 1 boot on

# 3. 格式化分区
sudo mkfs.vfat -F 32 /dev/sdX1
sudo mkfs.ext4 /dev/sdX2

# 4. 复制系统文件
sudo mkdir -p /mnt/starryos/{boot,root}
sudo mount /dev/sdX1 /mnt/starryos/boot
sudo mount /dev/sdX2 /mnt/starryos/root

sudo cp deploy/image/boot/* /mnt/starryos/boot/
sudo cp -r deploy/image/* /mnt/starryos/root/

# 5. 安装引导程序
sudo dd if=deploy/u-boot.bin of=/dev/sdX bs=512 seek=64

# 6. 卸载分区
sudo umount /mnt/starryos/boot
sudo umount /mnt/starryos/root
```

## RK3588特定配置

### 设备树配置

创建设备树文件 `starryos-rk3588.dts`：

```dts
/dts-v1/;

#include "rk3588.dtsi"

/ {
    model = "StarryOS RK3588";
    compatible = "rockchip,rk3588", "starryos";

    memory@0 {
        device_type = "memory";
        reg = <0x0 0x200000 0x0 0x40000000>; // 1GB内存
    };

    chosen {
        bootargs = "console=ttyS2,1500000 root=/dev/mmcblk0p2 rw rootwait";
        stdout-path = "serial2:1500000n8";
    };

    // NPU配置
    npu: npu@fde40000 {
        compatible = "rockchip,rk3588-npu";
        status = "okay";
    };

    // 音频配置
    audio: audio@fde00000 {
        compatible = "rockchip,rk3588-audio";
        status = "okay";
    };
};
```

### 内核命令行参数

```bash
# 在U-Boot中设置
setenv bootargs "console=ttyS2,1500000 root=/dev/mmcblk0p2 rw rootwait \
    starryos.voice_enabled=1 \
    starryos.ai_acceleration=1 \
    starryos.camera_device=/dev/video0"
```

## 功能验证

### 语音交互验证

系统启动后，测试语音功能：

```bash
# 测试语音识别
echo "测试语音识别功能" | speech-test

# 测试语音合成
tts-test "欢迎使用StarryOS语音系统"

# 测试唤醒词
wake-word-test
```

### 视觉识别验证

```bash
# 测试摄像头
camera-test --device /dev/video0

# 测试YOLO-v8识别
yolo-test --image test.jpg

# 测试实时识别
real-time-detection --camera 0
```

### 多模态融合验证

```bash
# 启动多模态演示
multimodal-demo --voice --vision --fusion

# 测试智能家居场景
smart-home-demo --light-control --temperature-query
```

## 性能优化

### NPU优化配置

编辑 `/etc/starryos/npu.conf`：

```ini
[npu]
# NPU性能模式
performance_mode = high

# 模型优化级别
optimization_level = aggressive

# 内存分配策略
memory_policy = balanced

# 功耗管理
power_management = adaptive
```

### 音频优化

编辑 `/etc/starryos/audio.conf`：

```ini
[audio]
# 采样率
sample_rate = 16000

# 缓冲区大小
buffer_size = 1600

# 语音活动检测
vad_threshold = 0.8

# 噪声抑制
noise_suppression = aggressive
```

## 故障排除

### 常见问题

**问题1**: 系统无法启动
```bash
# 检查引导顺序
uboot> printenv bootcmd

# 检查内核镜像
uboot> iminfo 0x1000000

# 检查设备树
uboot> fdt addr 0x2000000
uboot> fdt print
```

**问题2**: 语音功能异常
```bash
# 检查音频设备
ls -la /dev/snd/

# 测试麦克风
arecord -d 5 test.wav

# 检查音频驱动
lsmod | grep audio
```

**问题3**: AI推理失败
```bash
# 检查NPU状态
cat /sys/class/npu/npu0/status

# 测试模型加载
rknn_test --model model.rknn

# 检查内存使用
dmesg | grep -i memory
```

### 日志调试

启用详细日志：
```bash
# 设置日志级别
echo "debug" > /sys/module/starryos/parameters/log_level

# 查看内核日志
dmesg | grep starryos

# 查看应用日志
journalctl -u starryos-app
```

## 高级配置

### 自定义唤醒词

编辑 `/etc/starryos/voice.conf`：
```ini
[wake_word]
# 自定义唤醒词
custom_word = "小星"

# 唤醒词模型路径
model_path = "/etc/starryos/wake_word.model"

# 灵敏度设置
sensitivity = 0.8
```

### 网络配置

```bash
# 配置WiFi
wifi-config --ssid "YourSSID" --password "YourPassword"

# 配置静态IP
network-config --interface wlan0 --ip 192.168.1.100 --gateway 192.168.1.1

# 启用网络服务
systemctl enable starryos-network
```

## 维护和更新

### 系统更新

```bash
# 从源码更新
cd /opt/starryos
git pull
make clean && make build
make deploy

# 重启系统
reboot
```

### 数据备份

```bash
# 备份配置
tar -czf starryos-backup-$(date +%Y%m%d).tar.gz /etc/starryos /var/lib/starryos

# 备份模型数据
rsync -av /usr/share/starryos/models/ backup/models/
```

## 技术支持

- **文档**: 查看 `docs/` 目录获取详细文档
- **问题**: 提交GitHub Issue
- **社区**: 加入StarryOS开发者社区
- **邮件**: starryos-support@example.com

---

**注意**: 部署前请确保备份重要数据，部署过程中如遇问题请参考故障排除章节。
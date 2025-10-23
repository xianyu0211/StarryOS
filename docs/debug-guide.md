# StarryOS 调试指南

## 调试环境要求

### 软件依赖
- Rust工具链 (nightly)
- QEMU模拟器
- GDB调试器 (aarch64-none-elf-gdb)
- 交叉编译工具链

### 安装命令
```bash
# 安装Rust工具链
rustup target add aarch64-unknown-none
rustup component add llvm-tools-preview

# 安装QEMU (Ubuntu/Debian)
sudo apt-get install qemu-system-aarch64

# 安装GDB
sudo apt-get install gdb-multiarch
```

## 调试流程

### 1. 构建项目
```bash
make all
```

### 2. 启动QEMU调试模式
```bash
make boot-debug
```

### 3. 连接GDB调试器
```bash
aarch64-none-elf-gdb
(gdb) target remote :1234
(gdb) file target/aarch64-unknown-none/release/kernel
(gdb) break main
(gdb) continue
```

## 常见调试场景

### 内核启动调试
```bash
# 设置断点在入口点
break _start
# 单步执行
stepi
# 查看寄存器
info registers
```

### 内存调试
```bash
# 查看内存映射
info mem
# 检查特定地址
x/10x 0x80000
```

### 外设驱动调试
```bash
# 设置断点在驱动初始化
break dht22::init
# 查看变量值
print temperature
```

## 性能分析

### 使用perf工具
```bash
# 编译带调试符号的内核
cargo build --target aarch64-unknown-none --profile=perf

# 使用QEMU性能监控
qemu-system-aarch64 -M virt -cpu cortex-a72 -kernel kernel8.img -serial stdio -d cpu,exec
```

## 故障排除

### 常见问题
1. **GDB连接失败**: 检查QEMU是否在调试模式运行
2. **符号找不到**: 确保使用带调试符号的构建
3. **内存访问错误**: 检查内存映射配置

### 调试技巧
- 使用`layout asm`查看汇编代码
- 使用`watch`设置数据观察点
- 使用`backtrace`查看调用栈

## VS Code调试

### 配置说明
项目已配置完整的VS Code调试环境：

1. **构建内核**: 使用F5选择"构建内核"配置
2. **QEMU调试**: 启动QEMU并连接GDB调试器
3. **单元测试**: 运行所有单元测试
4. **性能测试**: 运行性能基准测试

### 调试快捷键
- F5: 启动调试
- F9: 设置/取消断点
- F10: 单步跳过
- F11: 单步进入
- Shift+F5: 停止调试

## 部署测试

### 部署到QEMU
```bash
# 快速部署测试
make deploy qemu

# 或者使用脚本
./scripts/deploy.sh qemu
```

### 部署到硬件
```bash
# 部署到香橙派AIpro
make deploy orangepi-aipro

# 或者使用脚本
./scripts/deploy.sh orangepi-aipro
```

## 自动化测试

### 持续集成
项目支持GitHub Actions自动化测试：

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Install Rust
      run: rustup update
    - name: Build
      run: make all
    - name: Test
      run: make test
```

## 性能监控

### 内存使用监控
```bash
# 查看内核内存布局
make memory-layout

# 监控内存使用
watch -n 1 'cat /proc/meminfo'
```

### CPU性能监控
```bash
# 监控CPU使用率
top -p $(pgrep qemu)

# 性能分析
perf record -g qemu-system-aarch64
perf report
```

## 硬件调试

### JTAG调试配置
对于香橙派AIpro等实际硬件：

1. **连接JTAG调试器**
2. **配置OpenOCD**
3. **使用GDB远程调试**

```bash
# OpenOCD配置
openocd -f interface/jlink.cfg -f target/orangepi.cfg

# GDB连接
(gdb) target remote :3333
(gdb) monitor reset halt
(gdb) load kernel8.img
(gdb) continue
```

## 调试最佳实践

1. **增量调试**: 从小模块开始，逐步扩大调试范围
2. **日志调试**: 使用串口输出调试信息
3. **断点策略**: 合理设置条件断点和观察点
4. **版本控制**: 使用Git管理调试版本

## 故障排除指南

### 构建失败
- 检查Rust工具链版本
- 验证依赖包完整性
- 清理构建缓存: `make clean`

### 运行失败
- 检查QEMU配置参数
- 验证内核镜像完整性
- 查看串口输出信息

### 调试失败
- 确认调试符号存在
- 检查GDB连接状态
- 验证内存映射配置

---

**注意**: 调试过程中遇到问题，请参考项目README和技术报告文档。
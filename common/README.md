# StarryOS 通用共享库

## 功能概述

通用共享库提供了StarryOS各模块间共享的数据结构、错误处理和工具函数，旨在减少代码重复、统一接口规范并提高系统可维护性。

## 模块结构

```
common/
├── src/
│   ├── lib.rs            # 库主入口
│   ├── error.rs          # 错误处理
│   ├── data_structures.rs # 通用数据结构
│   └── utils.rs          # 工具函数
├── tests/
│   └── common_tests.rs   # 单元测试
└── Cargo.toml            # 依赖管理
```

## 核心组件

### 1. 错误处理系统

统一的错误处理机制，支持跨模块错误传播和转换：

- `Error`: 通用错误枚举，包含系统、驱动、AI和应用错误
- `SystemError`: 系统相关错误类型
- `DriverError`: 驱动相关错误类型
- `AIError`: AI推理相关错误类型
- `AppError`: 应用程序相关错误类型
- `CommonResult<T>`: 通用的Result类型别名

### 2. 共享数据结构

- `BoundingBox`: 边界框结构，用于目标检测
- `Detection`: 检测结果，包含类别、置信度和边界框
- `SensorData`: 传感器数据，包含温度、湿度等信息
- `PerformanceMode`: 性能模式枚举（高性能、平衡、省电）
- `LogLevel`: 日志级别枚举
- `TaskInfo`: 任务信息结构

### 3. 工具函数

- 内存对齐: `align_memory`
- 数学计算: 平均值、标准差计算
- 数据处理: 快速排序、非最大值抑制(NMS)
- 向量运算: 归一化、点积计算

## 使用方法

在项目中添加依赖：

```toml
common = { path = "../common", features = ["alloc-support"] }
```

导入和使用：

```rust
use common::{Error, BoundingBox, calculate_mean, CommonResult};

fn process_data(data: &[f32]) -> CommonResult<f32> {
    let mean = calculate_mean(data);
    if mean > 100.0 {
        Err(Error::Other("数据超出阈值".to_string()))
    } else {
        Ok(mean)
    }
}
```

## 特性标志

- `alloc-support`: 启用堆内存分配支持

## 测试

运行单元测试：

```bash
cargo test --manifest-path common/Cargo.toml
```

## 设计原则

1. **最小依赖**: 保持核心依赖最小化，仅在需要时引入额外功能
2. **零拷贝**: 优先使用引用和所有权转移，避免不必要的数据复制
3. **类型安全**: 利用Rust的类型系统提供编译时安全保障
4. **可扩展**: 设计允许轻松添加新的错误类型和数据结构
5. **跨平台**: 确保代码在不同平台和环境下均可正常工作
//! StarryOS - 增强型外设驱动模块
//! 
//! 基于Rust异步编程模型，支持零拷贝数据传输和硬件加速

#![no_std]
#![feature(async_fn_in_trait)]

// 导入通用库
use common::{DriverError, SensorData, Result as CommonResult};

// 异步运行时支持
pub mod async_runtime;

// 驱动模块
pub mod environmental;
pub mod communication;
pub mod auxiliary;
pub mod npu;
pub mod rk3588_drivers;

// 通用接口
pub mod uart;
pub mod gpio;
pub mod i2c;
pub mod spi;
pub mod usb;
pub mod mipi_csi;

// 驱动管理器
mod manager;

use core::fmt;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

// 异步支持
pub use async_runtime::{AsyncRuntime, Executor, Task};

// DMA支持
pub mod dma;
pub use dma::{DmaBuffer, DmaController, ZeroCopyTransfer, DmaDirection};

/// 异步驱动特征
pub trait AsyncDriver {
    /// 驱动名称
    fn name(&self) -> &'static str;
    
    /// 异步初始化驱动
    async fn init(&mut self) -> Result<(), DriverError>;
    
    /// 检查设备是否就绪
    fn is_ready(&self) -> bool;
    
    /// 异步卸载驱动
    async fn deinit(&mut self) -> Result<(), DriverError>;
    
    /// 获取DMA支持状态
    fn supports_dma(&self) -> bool { false }
    
    /// 获取零拷贝支持状态
    fn supports_zero_copy(&self) -> bool { false }
}

/// 异步传感器驱动特征
pub trait AsyncSensorDriver: AsyncDriver {
    /// 异步读取传感器数据
    async fn read(&mut self) -> Result<SensorData, DriverError>;
    
    /// 使用DMA异步读取传感器数据（零拷贝）
    async fn read_dma(&mut self, buffer: &mut DmaBuffer) -> Result<(), DriverError> {
        Err(DriverError::NotSupported)
    }
}

/// 异步通信驱动特征
pub trait AsyncCommunicationDriver: AsyncDriver {
    /// 异步发送数据
    async fn send(&mut self, data: &[u8]) -> Result<(), DriverError>;
    
    /// 异步接收数据
    async fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, DriverError>;
    
    /// 使用DMA异步发送数据（零拷贝）
    async fn send_dma(&mut self, buffer: &DmaBuffer) -> Result<(), DriverError> {
        Err(DriverError::NotSupported)
    }
    
    /// 使用DMA异步接收数据（零拷贝）
    async fn receive_dma(&mut self, buffer: &mut DmaBuffer) -> Result<usize, DriverError> {
        Err(DriverError::NotSupported)
    }
}

/// 向后兼容的传统驱动特征
pub trait Driver {
    /// 驱动名称
    fn name(&self) -> &'static str;
    
    /// 初始化驱动
    fn init(&mut self) -> Result<(), DriverError>;
    
    /// 检查设备是否就绪
    fn is_ready(&self) -> bool;
    
    /// 卸载驱动
    fn deinit(&mut self) -> Result<(), DriverError>;
}

/// 向后兼容的传感器驱动特征
pub trait SensorDriver: Driver {
    /// 读取传感器数据
    fn read(&mut self) -> Result<SensorData, DriverError>;
}

/// 向后兼容的通信驱动特征
pub trait CommunicationDriver: Driver {
    /// 发送数据
    fn send(&mut self, data: &[u8]) -> Result<(), DriverError>;
    
    /// 接收数据
    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, DriverError>;
}

/// 传感器数据类型
#[derive(Debug, Clone)]
pub enum SensorData {
    Temperature(f32),        // 温度 (°C)
    Humidity(f32),          // 湿度 (%)
    Light(f32),             // 光照强度 (lux)
    Acceleration(f32, f32, f32), // 加速度 (x, y, z)
    Gyroscope(f32, f32, f32),    // 陀螺仪 (x, y, z)
}

/// 驱动管理器
pub struct DriverManager {
    drivers: manager::DriverRegistry,
}

impl DriverManager {
    /// 创建新的驱动管理器
    pub fn new() -> Self {
        Self {
            drivers: manager::DriverRegistry::new(),
        }
    }
    
    /// 注册驱动
    pub fn register_driver<T: Driver + 'static>(&mut self, driver: T) -> Result<(), DriverError> {
        self.drivers.register(Box::new(driver))
            .map_err(|_| DriverError::InvalidParameter)
    }
    
    /// 获取驱动数量
    pub fn driver_count(&self) -> usize {
        self.drivers.count()
    }
    
    /// 初始化所有驱动
    pub fn init_all(&mut self) -> Result<(), DriverError> {
        self.drivers.init_all()
    }
    
    /// 按名称查找驱动
    pub fn find_driver(&self, name: &str) -> Option<&dyn Driver> {
        self.drivers.find_by_name(name)
    }
    
    /// 根据类型查找驱动
    pub fn find_driver_by_type<T: Driver>(&self, name: &str) -> Option<&T> {
        self.drivers.find(name)
    }
}

/// 全局驱动管理器实例
pub static mut DRIVER_MANAGER: Option<DriverManager> = None;

/// 初始化驱动系统
pub fn init() {
    unsafe {
        DRIVER_MANAGER = Some(DriverManager::new());
    }
    
    // 初始化基础硬件驱动
    uart::init_debug_uart();
    gpio::init_gpio();
    i2c::init_i2c();
    spi::init_spi();
    
    println!("StarryOS驱动系统初始化完成");
}
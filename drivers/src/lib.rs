//! StarryOS - 外设驱动模块
//! 
//! 提供环境感知、通信交互、操作辅助等外设驱动支持

#![no_std]

// 驱动模块
pub mod environmental;
pub mod communication;
pub mod auxiliary;
pub mod npu;

// 通用接口
pub mod uart;
pub mod gpio;
pub mod i2c;
pub mod spi;

// 驱动管理器
mod manager;

use core::fmt;

/// 驱动错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    DeviceNotFound,
    CommunicationError,
    Timeout,
    InvalidParameter,
    NotSupported,
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriverError::DeviceNotFound => write!(f, "设备未找到"),
            DriverError::CommunicationError => write!(f, "通信错误"),
            DriverError::Timeout => write!(f, "操作超时"),
            DriverError::InvalidParameter => write!(f, "参数无效"),
            DriverError::NotSupported => write!(f, "不支持的操作"),
        }
    }
}

/// 驱动特征
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

/// 传感器驱动特征
pub trait SensorDriver: Driver {
    /// 读取传感器数据
    fn read(&mut self) -> Result<SensorData, DriverError>;
}

/// 通信驱动特征
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
        self.drivers.register(driver);
    }
    
    /// 初始化所有驱动
    pub fn init_all(&mut self) -> Result<(), DriverError> {
        self.drivers.init_all()
    }
    
    /// 根据名称查找驱动
    pub fn find_driver<T: Driver>(&self, name: &str) -> Option<&T> {
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
}
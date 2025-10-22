//! 环境感知类传感器驱动
//! 
//! 包括温湿度传感器、光线传感器、运动传感器等

mod dht22;
mod bh1750;
mod mpu6050;

use crate::{Driver, SensorDriver, SensorData, DriverError};

// 导出具体驱动
pub use dht22::DHT22Driver;
pub use bh1750::BH1750Driver;
pub use mpu6050::MPU6050Driver;

/// 环境传感器管理器
pub struct EnvironmentalSensorManager {
    sensors: heapless::Vec<Box<dyn SensorDriver>, 8>,
}

impl EnvironmentalSensorManager {
    /// 创建新的传感器管理器
    pub fn new() -> Self {
        Self {
            sensors: heapless::Vec::new(),
        }
    }
    
    /// 注册传感器
    pub fn register_sensor(&mut self, sensor: Box<dyn SensorDriver>) -> Result<(), DriverError> {
        self.sensors.push(sensor)
            .map_err(|_| DriverError::NotSupported)
    }
    
    /// 读取所有传感器数据
    pub fn read_all_sensors(&mut self) -> Result<Vec<SensorData>, DriverError> {
        let mut results = Vec::new();
        
        for sensor in &mut self.sensors.iter_mut() {
            if sensor.is_ready() {
                match sensor.read() {
                    Ok(data) => results.push(data),
                    Err(e) => return Err(e),
                }
            }
        }
        
        Ok(results)
    }
}
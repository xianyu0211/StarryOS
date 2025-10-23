//! 通信交互驱动模块
//! 
//! 提供WiFi、蓝牙、LoRa等通信外设驱动支持

mod wifi_esp32;
mod bluetooth_hc05;
mod lora_sx1276;

use crate::{Driver, DriverError, CommunicationDriver};
use alloc::string::String;

/// 通信设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunicationDevice {
    WiFiESP32,
    BluetoothHC05,
    LoRaSX1276,
}

/// 通信配置参数
#[derive(Debug, Clone)]
pub struct CommunicationConfig {
    pub device_type: CommunicationDevice,
    pub baud_rate: u32,
    pub channel: u8,
    pub power_level: u8,
}

/// 网络连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// 数据包结构
#[derive(Debug, Clone)]
pub struct DataPacket {
    pub source: u32,
    pub destination: u32,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

/// 通信管理器
pub struct CommunicationManager {
    devices: Vec<Box<dyn CommunicationDriver>>,
    current_device: Option<usize>,
}

impl CommunicationManager {
    /// 创建新的通信管理器
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            current_device: None,
        }
    }
    
    /// 注册通信设备
    pub fn register_device(&mut self, device: Box<dyn CommunicationDriver>) {
        self.devices.push(device);
    }
    
    /// 设置当前通信设备
    pub fn set_current_device(&mut self, index: usize) -> Result<(), DriverError> {
        if index < self.devices.len() {
            self.current_device = Some(index);
            Ok(())
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// 发送数据
    pub fn send_data(&mut self, data: &[u8]) -> Result<(), DriverError> {
        if let Some(index) = self.current_device {
            self.devices[index].send(data)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// 接收数据
    pub fn receive_data(&mut self, buffer: &mut [u8]) -> Result<usize, DriverError> {
        if let Some(index) = self.current_device {
            self.devices[index].receive(buffer)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
}

/// 创建通信驱动实例
pub fn create_communication_driver(device: CommunicationDevice) -> Result<Box<dyn CommunicationDriver>, DriverError> {
    match device {
        CommunicationDevice::WiFiESP32 => {
            Ok(Box::new(wifi_esp32::WiFiESP32Driver::new()))
        }
        CommunicationDevice::BluetoothHC05 => {
            Ok(Box::new(bluetooth_hc05::BluetoothHC05Driver::new()))
        }
        CommunicationDevice::LoRaSX1276 => {
            Ok(Box::new(lora_sx1276::LoRaSX1276Driver::new()))
        }
    }
}
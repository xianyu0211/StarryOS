//! 操作辅助驱动模块
//! 
//! 提供显示屏、蜂鸣器、LED等辅助操作外设驱动支持

mod oled_ssd1306;
mod buzzer_pwm;
mod led_rgb;

use crate::{Driver, DriverError};
use alloc::vec::Vec;

/// 辅助设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuxiliaryDevice {
    OLEDSSD1306,
    BuzzerPWM,
    LEDRGB,
}

/// 显示配置参数
#[derive(Debug, Clone, Copy)]
pub struct DisplayConfig {
    pub width: u16,
    pub height: u16,
    pub contrast: u8,
}

/// 声音配置参数
#[derive(Debug, Clone, Copy)]
pub struct SoundConfig {
    pub frequency: u32,
    pub duration: u32,
    pub volume: u8,
}

/// 灯光配置参数
#[derive(Debug, Clone, Copy)]
pub struct LightConfig {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub brightness: u8,
}

/// 辅助驱动特征
pub trait AuxiliaryDriver: Driver {
    /// 显示文本
    fn display_text(&mut self, text: &str) -> Result<(), DriverError>;
    
    /// 播放声音
    fn play_sound(&mut self, config: SoundConfig) -> Result<(), DriverError>;
    
    /// 控制灯光
    fn set_light(&mut self, config: LightConfig) -> Result<(), DriverError>;
}

/// 辅助管理器
pub struct AuxiliaryManager {
    devices: Vec<Box<dyn AuxiliaryDriver>>,
}

impl AuxiliaryManager {
    /// 创建新的辅助管理器
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }
    
    /// 注册辅助设备
    pub fn register_device(&mut self, device: Box<dyn AuxiliaryDriver>) {
        self.devices.push(device);
    }
    
    /// 显示文本到所有设备
    pub fn display_text_all(&mut self, text: &str) -> Result<(), DriverError> {
        for device in &mut self.devices {
            device.display_text(text)?;
        }
        Ok(())
    }
}
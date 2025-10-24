//! StarryOS - 应用程序模块
//! 
//! 提供语音交互、多模态融合等应用功能

#![no_std]

// 导入通用库
use common::{AppError, BoundingBox, Detection, PerformanceMode, LogLevel};

// 应用模块
pub mod voice_interaction;
pub mod multimodal_fusion;
pub mod system_integration;

// 工具模块
mod utils;

use core::fmt;

// AppError已从common库导入

/// 应用管理器特征
pub trait AppManager {
    /// 启动应用
    fn start(&mut self) -> Result<(), AppError>;
    
    /// 停止应用
    fn stop(&mut self) -> Result<(), AppError>;
    
    /// 获取应用状态
    fn get_status(&self) -> AppStatus;
    
    /// 处理事件
    fn handle_event(&mut self, event: AppEvent) -> Result<(), AppError>;
}

/// 应用状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

/// 应用事件
#[derive(Debug, Clone)]
pub enum AppEvent {
    VoiceCommand(String),
    VisualDetection(Vec<DetectionResult>),
    SensorData(SensorData),
    SystemEvent(SystemEvent),
}

// BoundingBox和Detection已从common库导入

/// 检测结果（适配应用层使用）
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub class_id: u32,
    pub class_name: String,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
}

// SensorData已从common库导入，但这里保持应用特定的SensorData以兼容现有代码

/// 系统事件
#[derive(Debug, Clone)]
pub enum SystemEvent {
    LowBattery,
    Overheating,
    NetworkConnected,
    NetworkDisconnected,
    StorageFull,
}

/// 应用配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub voice_enabled: bool,
    pub vision_enabled: bool,
    pub sensor_enabled: bool,
    pub network_enabled: bool,
    pub performance_mode: PerformanceMode,
    pub log_level: LogLevel,
}

/// 性能模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceMode {
    PowerSaving,
    Balanced,
    Performance,
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            voice_enabled: true,
            vision_enabled: true,
            sensor_enabled: true,
            network_enabled: true,
            performance_mode: PerformanceMode::Balanced,
            log_level: LogLevel::Info,
        }
    }
}
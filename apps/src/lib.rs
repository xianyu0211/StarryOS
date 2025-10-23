//! StarryOS - 应用程序模块
//! 
//! 提供语音交互、多模态融合等应用功能

#![no_std]

// 应用模块
pub mod voice_interaction;
pub mod multimodal_fusion;
pub mod system_integration;

// 工具模块
mod utils;

use core::fmt;

/// 应用错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppError {
    InitializationFailed,
    ResourceUnavailable,
    InvalidConfiguration,
    HardwareError,
    CommunicationError,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InitializationFailed => write!(f, "应用初始化失败"),
            AppError::ResourceUnavailable => write!(f, "资源不可用"),
            AppError::InvalidConfiguration => write!(f, "配置无效"),
            AppError::HardwareError => write!(f, "硬件错误"),
            AppError::CommunicationError => write!(f, "通信错误"),
        }
    }
}

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

/// 检测结果
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub class_id: u32,
    pub class_name: String,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
}

/// 边界框
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// 传感器数据
#[derive(Debug, Clone)]
pub struct SensorData {
    pub temperature: f32,
    pub humidity: f32,
    pub light_level: f32,
    pub motion_detected: bool,
}

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
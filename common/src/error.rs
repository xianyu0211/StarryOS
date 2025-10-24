//! 通用错误处理模块
//! 
//! 提供统一的错误类型定义和转换

use core::fmt;

// 条件导入alloc
#[cfg(feature = "alloc-support")]
extern crate alloc;
#[cfg(feature = "alloc-support")]
use alloc::string::String;

/// 通用Result类型定义
pub type CommonResult<T> = core::result::Result<T, Error>;

/// 通用错误类型
#[derive(Debug)]
pub enum Error {
    // 系统错误
    SystemError(SystemError),
    // 驱动错误
    DriverError(DriverError),
    // AI错误
    AIError(AIError),
    // 应用错误
    AppError(AppError),
    // 融合错误
    FusionError(String),
    // 其他错误
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SystemError(e) => write!(f, "系统错误: {}", e),
            Error::DriverError(e) => write!(f, "驱动错误: {}", e),
            Error::AIError(e) => write!(f, "AI错误: {}", e),
            Error::AppError(e) => write!(f, "应用错误: {}", e),
            Error::FusionError(msg) => write!(f, "融合错误: {}", msg),
            Error::Other(msg) => write!(f, "其他错误: {}", msg),
        }
    }
}

// 错误转换实现
impl From<SystemError> for Error {
    fn from(e: SystemError) -> Self {
        Error::SystemError(e)
    }
}

impl From<DriverError> for Error {
    fn from(e: DriverError) -> Self {
        Error::DriverError(e)
    }
}

impl From<AIError> for Error {
    fn from(e: AIError) -> Self {
        Error::AIError(e)
    }
}

impl From<AppError> for Error {
    fn from(e: AppError) -> Self {
        Error::AppError(e)
    }
}

#[cfg(feature = "alloc-support")]
impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Other(msg)
    }
}

#[cfg(feature = "alloc-support")]
impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::Other(msg.to_string())
    }
}

/// 系统错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemError {
    /// 未找到指定的资源
    ResourceNotFound,
    /// 内存分配失败
    MemoryAllocationFailed,
    /// 权限不足
    PermissionDenied,
    /// 操作超时
    Timeout,
    /// 参数无效
    InvalidParameter,
    /// 硬件不支持
    HardwareNotSupported,
    /// 系统繁忙
    SystemBusy,
    /// 未实现的功能
    NotImplemented,
    /// 未知错误
    Unknown,
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemError::ResourceNotFound => write!(f, "资源未找到"),
            SystemError::MemoryAllocationFailed => write!(f, "内存分配失败"),
            SystemError::PermissionDenied => write!(f, "权限不足"),
            SystemError::Timeout => write!(f, "操作超时"),
            SystemError::InvalidParameter => write!(f, "参数无效"),
            SystemError::HardwareNotSupported => write!(f, "硬件不支持"),
            SystemError::SystemBusy => write!(f, "系统繁忙"),
            SystemError::NotImplemented => write!(f, "未实现的功能"),
            SystemError::Unknown => write!(f, "未知错误"),
        }
    }
}

/// 驱动错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    /// 设备未找到
    DeviceNotFound,
    /// 通信错误
    CommunicationError,
    /// 操作超时
    Timeout,
    /// 参数无效
    InvalidParameter,
    /// 不支持的操作
    NotSupported,
    /// 设备忙
    DeviceBusy,
    /// 设备初始化失败
    InitializationFailed,
    /// 数据格式错误
    DataFormatError,
    /// I/O错误
    IoError,
    /// 配置错误
    ConfigurationError,
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriverError::DeviceNotFound => write!(f, "设备未找到"),
            DriverError::CommunicationError => write!(f, "通信错误"),
            DriverError::Timeout => write!(f, "操作超时"),
            DriverError::InvalidParameter => write!(f, "参数无效"),
            DriverError::NotSupported => write!(f, "不支持的操作"),
            DriverError::DeviceBusy => write!(f, "设备忙"),
            DriverError::InitializationFailed => write!(f, "设备初始化失败"),
            DriverError::DataFormatError => write!(f, "数据格式错误"),
            DriverError::IoError => write!(f, "I/O错误"),
            DriverError::ConfigurationError => write!(f, "配置错误"),
        }
    }
}

/// AI错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIError {
    /// 模型文件未找到
    ModelNotFound,
    /// 模型加载失败
    ModelLoadError,
    /// 推理执行错误
    InferenceError,
    /// 硬件不支持
    HardwareNotSupported,
    /// 内存分配错误
    MemoryAllocationError,
    /// 输入数据无效
    InvalidInput,
    /// 模型格式错误
    ModelFormatError,
    /// NPU初始化失败
    NpuInitializationFailed,
    /// 量化错误
    QuantizationError,
    /// 后处理错误
    PostProcessingError,
}

impl fmt::Display for AIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AIError::ModelNotFound => write!(f, "模型文件未找到"),
            AIError::ModelLoadError => write!(f, "模型加载失败"),
            AIError::InferenceError => write!(f, "推理执行错误"),
            AIError::HardwareNotSupported => write!(f, "硬件不支持"),
            AIError::MemoryAllocationError => write!(f, "内存分配错误"),
            AIError::InvalidInput => write!(f, "输入数据无效"),
            AIError::ModelFormatError => write!(f, "模型格式错误"),
            AIError::NpuInitializationFailed => write!(f, "NPU初始化失败"),
            AIError::QuantizationError => write!(f, "量化错误"),
            AIError::PostProcessingError => write!(f, "后处理错误"),
        }
    }
}

/// 应用错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppError {
    /// 应用初始化失败
    InitializationFailed,
    /// 资源不可用
    ResourceUnavailable,
    /// 配置无效
    InvalidConfiguration,
    /// 硬件错误
    HardwareError,
    /// 通信错误
    CommunicationError,
    /// 数据处理错误
    DataProcessingError,
    /// 权限错误
    PermissionError,
    /// 状态错误
    StateError,
    /// 超时错误
    TimeoutError,
    /// 未知错误
    UnknownError,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InitializationFailed => write!(f, "应用初始化失败"),
            AppError::ResourceUnavailable => write!(f, "资源不可用"),
            AppError::InvalidConfiguration => write!(f, "配置无效"),
            AppError::HardwareError => write!(f, "硬件错误"),
            AppError::CommunicationError => write!(f, "通信错误"),
            AppError::DataProcessingError => write!(f, "数据处理错误"),
            AppError::PermissionError => write!(f, "权限错误"),
            AppError::StateError => write!(f, "状态错误"),
            AppError::TimeoutError => write!(f, "超时错误"),
            AppError::UnknownError => write!(f, "未知错误"),
        }
    }
}

/// 错误转换辅助函数
pub trait ErrorConversion<T> {
    /// 转换为系统错误
    fn to_system_error(self) -> Result<T, SystemError>;
    
    /// 转换为驱动错误
    fn to_driver_error(self) -> Result<T, DriverError>;
    
    /// 转换为AI错误
    fn to_ai_error(self) -> Result<T, AIError>;
    
    /// 转换为应用错误
    fn to_app_error(self) -> Result<T, AppError>;
}

/// 错误处理宏，用于简化错误传播
#[macro_export]
macro_rules! try_or_return {
    ($expr:expr, $error_type:ident) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Err($error_type::from(e)),
        }
    };
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Err(e.into()),
        }
    };
}

/// 性能优化的错误处理宏，避免不必要的克隆
#[macro_export]
macro_rules! try_or_return_no_clone {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Err(e),
        }
    };
}
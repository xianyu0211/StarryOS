//! StarryOS - AI模块
//! 
//! 提供AI模型推理、硬件加速、算法优化等功能

#![no_std]

// AI核心模块
pub mod yolo_v8;
pub mod inference;
pub mod optimization;
pub mod npu;

// 工具模块
mod utils;

use core::fmt;

/// AI错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIError {
    ModelNotFound,
    ModelLoadError,
    InferenceError,
    HardwareNotSupported,
    MemoryAllocationError,
    InvalidInput,
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
        }
    }
}

/// AI推理引擎特征
pub trait InferenceEngine {
    /// 加载模型
    fn load_model(&mut self, model_data: &[u8]) -> Result<(), AIError>;
    
    /// 执行推理
    fn infer(&mut self, input: &[f32]) -> Result<Vec<f32>, AIError>;
    
    /// 获取模型信息
    fn model_info(&self) -> ModelInfo;
    
    /// 设置推理参数
    fn set_params(&mut self, params: InferenceParams) -> Result<(), AIError>;
}

/// 模型信息
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub precision: Precision,
}

/// 推理参数
#[derive(Debug, Clone, Copy)]
pub struct InferenceParams {
    pub batch_size: usize,
    pub use_hardware_acceleration: bool,
    pub optimization_level: OptimizationLevel,
}

/// 精度类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    FP32,
    FP16,
    INT8,
}

/// 优化级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
}

/// 检测结果
#[derive(Debug, Clone)]
pub struct Detection {
    pub class_id: usize,
    pub class_name: &'static str,
    pub confidence: f32,
    pub bbox: BoundingBox,
}

/// 边界框
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// AI管理器
pub struct AIManager {
    engines: Vec<Box<dyn InferenceEngine>>,
    current_engine: Option<usize>,
}

impl AIManager {
    /// 创建新的AI管理器
    pub fn new() -> Self {
        Self {
            engines: Vec::new(),
            current_engine: None,
        }
    }
    
    /// 注册推理引擎
    pub fn register_engine(&mut self, engine: Box<dyn InferenceEngine>) {
        self.engines.push(engine);
    }
    
    /// 设置当前使用的引擎
    pub fn set_current_engine(&mut self, index: usize) -> Result<(), AIError> {
        if index < self.engines.len() {
            self.current_engine = Some(index);
            Ok(())
        } else {
            Err(AIError::HardwareNotSupported)
        }
    }
    
    /// 执行推理
    pub fn infer(&mut self, input: &[f32]) -> Result<Vec<f32>, AIError> {
        if let Some(index) = self.current_engine {
            self.engines[index].infer(input)
        } else {
            Err(AIError::InferenceError)
        }
    }
}

/// 全局AI管理器实例
pub static mut AI_MANAGER: Option<AIManager> = None;

/// 初始化AI系统
pub fn init() {
    unsafe {
        AI_MANAGER = Some(AIManager::new());
    }
}
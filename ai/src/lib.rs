//! StarryOS - AI模块
//! 
//! 提供AI模型推理、硬件加速、算法优化等功能

#![no_std]

// 导入通用库
use common::{AIError, BoundingBox, Detection, Result as CommonResult};

// AI核心模块
pub mod yolo_v8;
pub mod inference;
pub mod optimization;
pub mod npu;
pub mod rk3588_npu;

// 工具模块
mod utils;

use core::fmt;

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

// Detection和BoundingBox已从common库导入

/// AI管理器
pub struct AIManager {
    engines: Vec<Box<dyn InferenceEngine>>,
    current_engine: Option<usize>,
}

impl AIManager {
    /// 创建新的AI管理器
    pub fn new() -> Self {
        Self {
            engines: Vec::with_capacity(4), // 预分配容量，减少内存分配
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
    
    /// 批量推理，提高吞吐量
    pub fn infer_batch(&mut self, inputs: &[&[f32]]) -> Result<Vec<Vec<f32>>, AIError> {
        if let Some(index) = self.current_engine {
            let mut results = Vec::with_capacity(inputs.len());
            for input in inputs {
                results.push(self.engines[index].infer(input)?);
            }
            Ok(results)
        } else {
            Err(AIError::InferenceError)
        }
    }
    
    /// 获取引擎数量
    pub fn engine_count(&self) -> usize {
        self.engines.len()
    }
    
    /// 获取当前引擎信息（避免克隆）
    pub fn current_engine_info(&self) -> Option<&ModelInfo> {
        if let Some(index) = self.current_engine {
            Some(&self.engines[index].model_info())
        } else {
            None
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
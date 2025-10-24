//! 通用共享库 - StarryOS
//! 
//! 提供跨模块共享的数据结构、错误处理、性能优化和工具函数

#![no_std]

// 条件导入alloc
#[cfg(feature = "alloc-support")]
extern crate alloc;
#[cfg(feature = "alloc-support")]
use alloc::{string::String, vec::Vec};

// 错误处理模块
mod error;
// 数据结构模块
mod data_structures;
// 工具函数模块
mod utils;
// 性能优化模块
mod performance;

// 公共导出
pub use error::{Error, SystemError, DriverError, AIError, AppError, CommonResult};
pub use data_structures::{BoundingBox, Detection, SensorData, PerformanceMode, LogLevel, TaskInfo};
pub use utils::{align_memory, calculate_mean, calculate_stddev, quick_sort, non_max_suppression, normalize_vector, dot_product};
pub use performance::{PerformanceMonitor, MemoryPool, AlgorithmOptimizer, CacheOptimized, benchmark};
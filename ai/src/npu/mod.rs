//! NPU (神经网络处理单元) 驱动模块
//! 
//! 提供国产SoC芯片AI加速单元的硬件抽象和驱动支持

mod allwinner_v851s;
mod rockchip_rk3588;
mod generic_opencl;

use crate::{AIError, InferenceEngine, ModelInfo, InferenceParams};
use alloc::vec::Vec;

/// NPU设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NPUDevice {
    AllwinnerV851S,
    RockchipRK3588,
    GenericOpenCL,
}

/// NPU配置参数
#[derive(Debug, Clone)]
pub struct NPUConfig {
    pub device_type: NPUDevice,
    pub memory_size: usize,
    pub clock_frequency: u32,
    pub supported_precision: Vec<Precision>,
}

/// NPU驱动特征
pub trait NPUDriver: InferenceEngine {
    /// 获取NPU设备信息
    fn device_info(&self) -> NPUDeviceInfo;
    
    /// 设置NPU工作频率
    fn set_clock_frequency(&mut self, frequency: u32) -> Result<(), AIError>;
    
    /// 获取NPU性能统计
    fn performance_stats(&self) -> NPUPerformanceStats;
}

/// NPU设备信息
#[derive(Debug, Clone)]
pub struct NPUDeviceInfo {
    pub vendor: &'static str,
    pub device_name: &'static str,
    pub compute_units: u32,
    pub memory_bandwidth: f32, // GB/s
    pub peak_performance: f32,  // TOPS
}

/// NPU性能统计
#[derive(Debug, Clone)]
pub struct NPUPerformanceStats {
    pub inference_time: u64,    // 微秒
    pub memory_usage: usize,    // 字节
    pub power_consumption: f32, // 瓦特
    pub utilization: f32,       // 利用率百分比
}

/// 创建NPU驱动实例
pub fn create_npu_driver(device: NPUDevice) -> Result<Box<dyn NPUDriver>, AIError> {
    match device {
        NPUDevice::AllwinnerV851S => {
            Ok(Box::new(allwinner_v851s::AllwinnerV851SDriver::new()))
        }
        NPUDevice::RockchipRK3588 => {
            Ok(Box::new(rockchip_rk3588::RockchipRK3588Driver::new()))
        }
        NPUDevice::GenericOpenCL => {
            Ok(Box::new(generic_opencl::GenericOpenCLDriver::new()))
        }
    }
}

/// 检测可用的NPU设备
pub fn detect_available_npus() -> Vec<NPUDevice> {
    let mut available_devices = Vec::new();
    
    // 这里实现设备检测逻辑
    // 实际实现需要读取硬件寄存器或系统信息
    
    // 模拟检测结果
    available_devices.push(NPUDevice::GenericOpenCL);
    
    available_devices
}
//! RK3588 NPU驱动模块
//! 
//! 支持RK3588内置NPU（神经网络处理单元）的算力调度接口

#![no_std]

use core::arch::asm;
use core::mem::size_of;

/// NPU错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpuError {
    NotInitialized,
    CommunicationError,
    Timeout,
    InvalidModel,
    MemoryAllocationError,
    HardwareError,
}

/// NPU状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpuState {
    Idle,
    Busy,
    Error,
}

/// NPU配置参数
#[derive(Debug, Clone, Copy)]
pub struct NpuConfig {
    pub clock_frequency: u32,    // 时钟频率 (MHz)
    pub power_mode: PowerMode,    // 功耗模式
    pub memory_size: usize,       // 内存大小
    pub batch_size: u32,         // 批处理大小
}

/// 功耗模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerMode {
    LowPower,    // 低功耗模式
    Balanced,    // 平衡模式
    Performance, // 高性能模式
}

/// NPU管理器
pub struct NpuManager {
    base_address: u64,
    state: NpuState,
    config: NpuConfig,
    current_model: Option<ModelInfo>,
}

/// 模型信息
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: &'static str,
    pub input_shape: [usize; 4],  // [batch, height, width, channels]
    pub output_shape: [usize; 4], // [batch, classes, height, width]
    pub precision: Precision,
    pub memory_usage: usize,
}

/// 精度类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    FP32,
    FP16,
    INT8,
}

impl NpuManager {
    /// 创建新的NPU管理器
    pub const fn new() -> Self {
        Self {
            base_address: 0xFDBC_0000, // NPU基地址
            state: NpuState::Idle,
            config: NpuConfig {
                clock_frequency: 800, // 800MHz
                power_mode: PowerMode::Balanced,
                memory_size: 64 * 1024 * 1024, // 64MB
                batch_size: 1,
            },
            current_model: None,
        }
    }
    
    /// 初始化NPU
    pub unsafe fn init(&mut self) -> Result<(), NpuError> {
        // 重置NPU
        self.reset()?;
        
        // 配置时钟
        self.configure_clock()?;
        
        // 配置内存
        self.configure_memory()?;
        
        // 启用NPU
        self.enable()?;
        
        self.state = NpuState::Idle;
        Ok(())
    }
    
    /// 重置NPU
    unsafe fn reset(&self) -> Result<(), NpuError> {
        let npu_reg = self.base_address as *mut u32;
        
        // 发送重置信号
        npu_reg.add(0x00).write_volatile(0x1);
        
        // 等待重置完成
        let mut timeout = 100000;
        while npu_reg.add(0x04).read_volatile() & 0x1 == 0 {
            timeout -= 1;
            if timeout == 0 {
                return Err(NpuError::Timeout);
            }
        }
        
        Ok(())
    }
    
    /// 配置时钟
    unsafe fn configure_clock(&self) -> Result<(), NpuError> {
        let npu_reg = self.base_address as *mut u32;
        
        // 设置时钟频率
        match self.config.clock_frequency {
            400 => npu_reg.add(0x10).write_volatile(0x0), // 400MHz
            600 => npu_reg.add(0x10).write_volatile(0x1), // 600MHz
            800 => npu_reg.add(0x10).write_volatile(0x2), // 800MHz
            _ => return Err(NpuError::InvalidModel),
        }
        
        Ok(())
    }
    
    /// 配置内存
    unsafe fn configure_memory(&self) -> Result<(), NpuError> {
        let npu_reg = self.base_address as *mut u32;
        
        // 设置内存大小
        npu_reg.add(0x20).write_volatile((self.config.memory_size / 1024 / 1024) as u32);
        
        Ok(())
    }
    
    /// 启用NPU
    unsafe fn enable(&self) -> Result<(), NpuError> {
        let npu_reg = self.base_address as *mut u32;
        
        // 启用NPU
        npu_reg.add(0x08).write_volatile(0x1);
        
        // 等待启用完成
        let mut timeout = 100000;
        while npu_reg.add(0x0C).read_volatile() & 0x1 == 0 {
            timeout -= 1;
            if timeout == 0 {
                return Err(NpuError::Timeout);
            }
        }
        
        Ok(())
    }
    
    /// 加载模型
    pub unsafe fn load_model(&mut self, model_data: &[u8], model_info: ModelInfo) -> Result<(), NpuError> {
        if self.state != NpuState::Idle {
            return Err(NpuError::Busy);
        }
        
        // 检查模型大小
        if model_data.len() > self.config.memory_size {
            return Err(NpuError::MemoryAllocationError);
        }
        
        let npu_reg = self.base_address as *mut u32;
        
        // 设置模型地址
        let model_addr = 0x1000_0000; // 模型加载地址
        npu_reg.add(0x30).write_volatile(model_addr as u32);
        
        // 设置模型大小
        npu_reg.add(0x34).write_volatile(model_data.len() as u32);
        
        // 复制模型数据到NPU内存
        let model_ptr = model_addr as *mut u8;
        for (i, &byte) in model_data.iter().enumerate() {
            model_ptr.add(i).write_volatile(byte);
        }
        
        // 发送加载命令
        npu_reg.add(0x38).write_volatile(0x1);
        
        // 等待加载完成
        let mut timeout = 1000000;
        while npu_reg.add(0x3C).read_volatile() & 0x1 == 0 {
            timeout -= 1;
            if timeout == 0 {
                return Err(NpuError::Timeout);
            }
        }
        
        self.current_model = Some(model_info);
        Ok(())
    }
    
    /// 执行推理
    pub unsafe fn infer(&mut self, input_data: &[f32]) -> Result<Vec<f32>, NpuError> {
        if self.state != NpuState::Idle {
            return Err(NpuError::Busy);
        }
        
        if self.current_model.is_none() {
            return Err(NpuError::NotInitialized);
        }
        
        let model_info = self.current_model.as_ref().unwrap();
        let expected_size = model_info.input_shape.iter().product::<usize>();
        
        if input_data.len() != expected_size {
            return Err(NpuError::InvalidModel);
        }
        
        let npu_reg = self.base_address as *mut u32;
        
        // 设置输入数据地址
        let input_addr = 0x2000_0000; // 输入数据地址
        npu_reg.add(0x40).write_volatile(input_addr as u32);
        
        // 复制输入数据
        let input_ptr = input_addr as *mut f32;
        for (i, &value) in input_data.iter().enumerate() {
            input_ptr.add(i).write_volatile(value);
        }
        
        // 设置输出数据地址
        let output_addr = 0x3000_0000; // 输出数据地址
        npu_reg.add(0x44).write_volatile(output_addr as u32);
        
        // 发送推理命令
        npu_reg.add(0x48).write_volatile(0x1);
        
        self.state = NpuState::Busy;
        
        // 等待推理完成
        let mut timeout = 1000000;
        while npu_reg.add(0x4C).read_volatile() & 0x1 == 0 {
            timeout -= 1;
            if timeout == 0 {
                self.state = NpuState::Idle;
                return Err(NpuError::Timeout);
            }
        }
        
        // 读取输出数据
        let output_size = model_info.output_shape.iter().product::<usize>();
        let output_ptr = output_addr as *const f32;
        let mut output_data = Vec::with_capacity(output_size);
        
        for i in 0..output_size {
            output_data.push(output_ptr.add(i).read_volatile());
        }
        
        self.state = NpuState::Idle;
        Ok(output_data)
    }
    
    /// 获取NPU状态
    pub fn get_state(&self) -> NpuState {
        self.state
    }
    
    /// 获取NPU配置
    pub fn get_config(&self) -> &NpuConfig {
        &self.config
    }
    
    /// 设置NPU配置
    pub fn set_config(&mut self, config: NpuConfig) -> Result<(), NpuError> {
        if self.state != NpuState::Idle {
            return Err(NpuError::Busy);
        }
        
        self.config = config;
        Ok(())
    }
    
    /// 获取当前加载的模型信息
    pub fn get_current_model(&self) -> Option<&ModelInfo> {
        self.current_model.as_ref()
    }
    
    /// 卸载模型
    pub unsafe fn unload_model(&mut self) -> Result<(), NpuError> {
        if self.state != NpuState::Idle {
            return Err(NpuError::Busy);
        }
        
        let npu_reg = self.base_address as *mut u32;
        
        // 发送卸载命令
        npu_reg.add(0x50).write_volatile(0x1);
        
        // 等待卸载完成
        let mut timeout = 100000;
        while npu_reg.add(0x54).read_volatile() & 0x1 == 0 {
            timeout -= 1;
            if timeout == 0 {
                return Err(NpuError::Timeout);
            }
        }
        
        self.current_model = None;
        Ok(())
    }
    
    /// 获取NPU性能统计
    pub unsafe fn get_performance_stats(&self) -> PerformanceStats {
        let npu_reg = self.base_address as *mut u32;
        
        PerformanceStats {
            inference_count: npu_reg.add(0x60).read_volatile(),
            total_time_ms: npu_reg.add(0x64).read_volatile(),
            average_time_ms: npu_reg.add(0x68).read_volatile(),
            error_count: npu_reg.add(0x6C).read_volatile(),
        }
    }
}

/// 性能统计
#[derive(Debug, Clone, Copy)]
pub struct PerformanceStats {
    pub inference_count: u32,
    pub total_time_ms: u32,
    pub average_time_ms: u32,
    pub error_count: u32,
}

/// 全局NPU管理器实例
pub static mut NPU_MANAGER: Option<NpuManager> = None;

/// 初始化NPU系统
pub unsafe fn init() -> Result<(), NpuError> {
    if NPU_MANAGER.is_none() {
        NPU_MANAGER = Some(NpuManager::new());
    }
    
    if let Some(npu) = &mut NPU_MANAGER {
        npu.init()
    } else {
        Err(NpuError::HardwareError)
    }
}

/// 获取NPU管理器实例
pub unsafe fn get_npu_manager() -> Option<&'static mut NpuManager> {
    NPU_MANAGER.as_mut()
}

/// NPU驱动实现
impl crate::Driver for NpuManager {
    fn name(&self) -> &'static str {
        "RK3588 NPU Driver"
    }
    
    fn init(&mut self) -> Result<(), crate::DriverError> {
        unsafe {
            match self.init() {
                Ok(()) => Ok(()),
                Err(e) => match e {
                    NpuError::NotInitialized => Err(crate::DriverError::DeviceNotFound),
                    NpuError::CommunicationError => Err(crate::DriverError::CommunicationError),
                    NpuError::Timeout => Err(crate::DriverError::Timeout),
                    NpuError::InvalidModel => Err(crate::DriverError::InvalidParameter),
                    NpuError::MemoryAllocationError => Err(crate::DriverError::NotSupported),
                    NpuError::HardwareError => Err(crate::DriverError::DeviceNotFound),
                },
            }
        }
    }
    
    fn is_ready(&self) -> bool {
        self.state == NpuState::Idle
    }
    
    fn deinit(&mut self) -> Result<(), crate::DriverError> {
        // NPU不需要特殊的卸载操作
        Ok(())
    }
}
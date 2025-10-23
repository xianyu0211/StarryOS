//! CPU核心管理模块
//! 
//! 支持RK3588的四核Cortex-A76 + 四核Cortex-A55异构架构调度

#![no_std]

use core::arch::asm;
use core::sync::atomic::{AtomicU32, Ordering};

/// CPU核心ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreId {
    A76_0 = 0,
    A76_1 = 1,
    A76_2 = 2,
    A76_3 = 3,
    A55_0 = 4,
    A55_1 = 5,
    A55_2 = 6,
    A55_3 = 7,
}

impl CoreId {
    /// 获取当前CPU核心ID
    pub fn current() -> CoreId {
        let mpidr: u64;
        unsafe {
            asm!(
                "mrs {}, mpidr_el1",
                out(reg) mpidr
            );
        }
        
        // 提取Aff0字段（核心ID）
        let core_id = (mpidr & 0xFF) as u8;
        
        match core_id {
            0 => CoreId::A76_0,
            1 => CoreId::A76_1,
            2 => CoreId::A76_2,
            3 => CoreId::A76_3,
            4 => CoreId::A55_0,
            5 => CoreId::A55_1,
            6 => CoreId::A55_2,
            7 => CoreId::A55_2,
            _ => CoreId::A76_0, // 默认
        }
    }
    
    /// 判断是否为高性能核心（A76）
    pub fn is_performance_core(&self) -> bool {
        matches!(*self, CoreId::A76_0 | CoreId::A76_1 | CoreId::A76_2 | CoreId::A76_3)
    }
    
    /// 判断是否为低功耗核心（A55）
    pub fn is_efficiency_core(&self) -> bool {
        matches!(*self, CoreId::A55_0 | CoreId::A55_1 | CoreId::A55_2 | CoreId::A55_3)
    }
}

/// CPU核心状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreState {
    Off,        // 关闭
    Idle,       // 空闲
    Running,    // 运行中
    Suspended,  // 挂起
}

/// CPU核心管理器
pub struct CpuManager {
    core_states: [AtomicU32; 8],
    current_task: [AtomicU32; 8],
}

impl CpuManager {
    /// 创建新的CPU管理器
    pub const fn new() -> Self {
        Self {
            core_states: [
                AtomicU32::new(CoreState::Off as u32),
                AtomicU32::new(CoreState::Off as u32),
                AtomicU32::new(CoreState::Off as u32),
                AtomicU32::new(CoreState::Off as u32),
                AtomicU32::new(CoreState::Off as u32),
                AtomicU32::new(CoreState::Off as u32),
                AtomicU32::new(CoreState::Off as u32),
                AtomicU32::new(CoreState::Off as u32),
            ],
            current_task: [AtomicU32::new(0); 8],
        }
    }
    
    /// 启动指定核心
    pub fn start_core(&self, core_id: CoreId) -> Result<(), &'static str> {
        let state = &self.core_states[core_id as usize];
        
        if state.load(Ordering::Acquire) == CoreState::Off as u32 {
            state.store(CoreState::Running as u32, Ordering::Release);
            
            // 实际启动核心的硬件操作
            unsafe {
                self.actual_start_core(core_id);
            }
            
            Ok(())
        } else {
            Err("核心已经启动")
        }
    }
    
    /// 停止指定核心
    pub fn stop_core(&self, core_id: CoreId) -> Result<(), &'static str> {
        let state = &self.core_states[core_id as usize];
        
        if state.load(Ordering::Acquire) != CoreState::Off as u32 {
            state.store(CoreState::Off as u32, Ordering::Release);
            
            // 实际停止核心的硬件操作
            unsafe {
                self.actual_stop_core(core_id);
            }
            
            Ok(())
        } else {
            Err("核心已经停止")
        }
    }
    
    /// 获取核心状态
    pub fn get_core_state(&self, core_id: CoreId) -> CoreState {
        let state_value = self.core_states[core_id as usize].load(Ordering::Acquire);
        
        match state_value {
            0 => CoreState::Off,
            1 => CoreState::Idle,
            2 => CoreState::Running,
            3 => CoreState::Suspended,
            _ => CoreState::Off,
        }
    }
    
    /// 设置核心任务
    pub fn set_core_task(&self, core_id: CoreId, task_id: u32) {
        self.current_task[core_id as usize].store(task_id, Ordering::Release);
    }
    
    /// 获取核心当前任务
    pub fn get_core_task(&self, core_id: CoreId) -> u32 {
        self.current_task[core_id as usize].load(Ordering::Acquire)
    }
    
    /// 实际启动核心（硬件操作）
    unsafe fn actual_start_core(&self, core_id: CoreId) {
        // RK3588核心启动序列
        match core_id {
            CoreId::A76_0 | CoreId::A76_1 | CoreId::A76_2 | CoreId::A76_3 => {
                // 启动A76高性能核心
                let base_addr = 0xFFFF_0000 + (core_id as u64) * 0x1000;
                let ptr = base_addr as *mut u32;
                
                // 设置启动地址
                ptr.add(0).write_volatile(0x8000_0000); // 内核入口地址
                
                // 发送启动信号
                ptr.add(1).write_volatile(0x1);
            }
            CoreId::A55_0 | CoreId::A55_1 | CoreId::A55_2 | CoreId::A55_3 => {
                // 启动A55低功耗核心
                let base_addr = 0xFFFF_1000 + (core_id as u64 - 4) * 0x1000;
                let ptr = base_addr as *mut u32;
                
                // 设置启动地址
                ptr.add(0).write_volatile(0x8000_0000); // 内核入口地址
                
                // 发送启动信号
                ptr.add(1).write_volatile(0x1);
            }
        }
    }
    
    /// 实际停止核心（硬件操作）
    unsafe fn actual_stop_core(&self, core_id: CoreId) {
        // RK3588核心停止序列
        match core_id {
            CoreId::A76_0 | CoreId::A76_1 | CoreId::A76_2 | CoreId::A76_3 => {
                // 停止A76高性能核心
                let base_addr = 0xFFFF_0000 + (core_id as u64) * 0x1000;
                let ptr = base_addr as *mut u32;
                
                // 发送停止信号
                ptr.add(2).write_volatile(0x1);
            }
            CoreId::A55_0 | CoreId::A55_1 | CoreId::A55_2 | CoreId::A55_3 => {
                // 停止A55低功耗核心
                let base_addr = 0xFFFF_1000 + (core_id as u64 - 4) * 0x1000;
                let ptr = base_addr as *mut u32;
                
                // 发送停止信号
                ptr.add(2).write_volatile(0x1);
            }
        }
    }
}

/// 全局CPU管理器实例
pub static CPU_MANAGER: CpuManager = CpuManager::new();

/// 初始化CPU系统
pub fn init() {
    // 启动主核心（A76_0）
    let _ = CPU_MANAGER.start_core(CoreId::A76_0);
    
    // 根据系统负载决定是否启动其他核心
    // 默认只启动主核心，其他核心按需启动
}

/// CPU调度器
pub struct Scheduler {
    performance_cores: [CoreId; 4],
    efficiency_cores: [CoreId; 4],
    current_performance_index: usize,
    current_efficiency_index: usize,
}

impl Scheduler {
    /// 创建新的调度器
    pub const fn new() -> Self {
        Self {
            performance_cores: [CoreId::A76_0, CoreId::A76_1, CoreId::A76_2, CoreId::A76_3],
            efficiency_cores: [CoreId::A55_0, CoreId::A55_1, CoreId::A55_2, CoreId::A55_3],
            current_performance_index: 0,
            current_efficiency_index: 0,
        }
    }
    
    /// 调度任务到合适的核心
    pub fn schedule_task(&mut self, task_priority: u8) -> CoreId {
        if task_priority >= 80 {
            // 高优先级任务分配到高性能核心
            let core_id = self.performance_cores[self.current_performance_index];
            self.current_performance_index = (self.current_performance_index + 1) % 4;
            core_id
        } else {
            // 普通任务分配到低功耗核心
            let core_id = self.efficiency_cores[self.current_efficiency_index];
            self.current_efficiency_index = (self.current_efficiency_index + 1) % 4;
            core_id
        }
    }
}

/// 全局调度器实例
pub static mut SCHEDULER: Option<Scheduler> = None;

/// 初始化调度器
pub fn init_scheduler() {
    unsafe {
        SCHEDULER = Some(Scheduler::new());
    }
}
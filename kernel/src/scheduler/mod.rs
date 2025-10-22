//! 进程调度器模块
//! 
//! 提供多任务调度、进程管理、上下文切换等功能

mod process;
mod scheduler;
mod context;

use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::vec::Vec;

// 全局进程ID计数器
static NEXT_PID: AtomicUsize = AtomicUsize::new(1);

/// 进程状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,      // 就绪
    Running,    // 运行中
    Blocked,    // 阻塞
    Terminated, // 终止
}

/// 进程控制块 (PCB)
pub struct ProcessControlBlock {
    pub pid: usize,           // 进程ID
    pub state: ProcessState,  // 进程状态
    pub priority: u8,         // 优先级 (0-255)
    pub context: context::Context, // 执行上下文
}

/// 调度器
pub struct Scheduler {
    processes: Vec<ProcessControlBlock>,
    current_pid: Option<usize>,
}

impl Scheduler {
    /// 创建新的调度器
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            current_pid: None,
        }
    }
    
    /// 添加进程到调度队列
    pub fn add_process(&mut self, entry_point: usize) -> usize {
        let pid = NEXT_PID.fetch_add(1, Ordering::Relaxed);
        
        let pcb = ProcessControlBlock {
            pid,
            state: ProcessState::Ready,
            priority: 1,
            context: context::Context::new(entry_point),
        };
        
        self.processes.push(pcb);
        pid
    }
    
    /// 调度下一个进程
    pub fn schedule(&mut self) -> Option<&mut ProcessControlBlock> {
        if self.processes.is_empty() {
            return None;
        }
        
        // 简单的轮转调度算法
        let next_index = match self.current_pid {
            Some(current_pid) => {
                let current_index = self.processes
                    .iter()
                    .position(|p| p.pid == current_pid)
                    .unwrap_or(0);
                (current_index + 1) % self.processes.len()
            }
            None => 0,
        };
        
        let next_pcb = &mut self.processes[next_index];
        self.current_pid = Some(next_pcb.pid);
        
        Some(next_pcb)
    }
    
    /// 获取当前运行的进程
    pub fn current_process(&self) -> Option<&ProcessControlBlock> {
        self.current_pid
            .and_then(|pid| self.processes.iter().find(|p| p.pid == pid))
    }
}

/// 启动调度器
pub fn start() {
    // 创建初始进程
    let mut scheduler = Scheduler::new();
    
    // 添加空闲进程
    scheduler.add_process(idle_task as usize);
    
    // 启动调度循环
    loop {
        if let Some(current) = scheduler.schedule() {
            // 切换到进程上下文
            unsafe {
                context::switch(&mut current.context);
            }
        }
    }
}

/// 空闲任务
fn idle_task() -> ! {
    loop {
        // 空闲时降低功耗
        cortex_a::asm::wfe(); // 等待事件
    }
}

// 导出子模块
pub use process::Process;
pub use scheduler::RoundRobinScheduler;
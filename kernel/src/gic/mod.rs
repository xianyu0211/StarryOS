//! 通用中断控制器（GIC）模块
//! 
//! 支持RK3588的GIC-500中断控制器，实现中断响应与优先级管理

#![no_std]

use core::arch::asm;
use core::sync::atomic::{AtomicU32, Ordering};

/// 中断类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptType {
    SPI = 0,    // 共享外设中断
    PPI = 1,    // 私有外设中断
    SGI = 2,    // 软件生成中断
}

/// 中断优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptPriority(u8);

impl InterruptPriority {
    /// 创建新的中断优先级
    pub const fn new(priority: u8) -> Self {
        Self(priority)
    }
    
    /// 获取优先级值
    pub fn value(&self) -> u8 {
        self.0
    }
    
    /// 最高优先级
    pub const HIGHEST: Self = Self(0);
    
    /// 默认优先级
    pub const DEFAULT: Self = Self(0x80);
    
    /// 最低优先级
    pub const LOWEST: Self = Self(0xFF);
}

/// 中断控制器管理器
pub struct GicManager {
    distributor_base: u64,
    redistributor_base: u64,
    cpu_interface_base: u64,
    enabled_interrupts: [AtomicU32; 32], // 每个bit对应一个中断
}

impl GicManager {
    /// 创建新的GIC管理器
    pub const fn new() -> Self {
        Self {
            distributor_base: 0xFD40_0000, // GICD基地址
            redistributor_base: 0xFD60_0000, // GICR基地址
            cpu_interface_base: 0xFEC0_0000, // GICC基地址
            enabled_interrupts: [AtomicU32::new(0); 32],
        }
    }
    
    /// 初始化GIC
    pub unsafe fn init(&self) {
        // 初始化分发器
        self.init_distributor();
        
        // 初始化重分发器
        self.init_redistributor();
        
        // 初始化CPU接口
        self.init_cpu_interface();
        
        // 启用系统中断
        self.enable_system_interrupts();
    }
    
    /// 初始化分发器
    unsafe fn init_distributor(&self) {
        let gicd = self.distributor_base as *mut u32;
        
        // 禁用所有中断
        for i in 0..32 {
            gicd.add(0x80 + i * 4).write_volatile(0xFFFFFFFF); // ICDICER
        }
        
        // 设置所有中断为电平触发
        for i in 0..32 {
            gicd.add(0xC00 + i * 4).write_volatile(0); // ICDICFR
        }
        
        // 设置所有中断优先级为默认值
        for i in 0..255 {
            gicd.add(0x400 + i).write_volatile(0x80); // ICDIPR
        }
        
        // 设置所有中断目标为CPU0
        for i in 0..64 {
            gicd.add(0x800 + i * 4).write_volatile(0x01010101); // ICDIPTR
        }
        
        // 启用分发器
        gicd.add(0).write_volatile(1); // ICDDCR
    }
    
    /// 初始化重分发器
    unsafe fn init_redistributor(&self) {
        let gicr = self.redistributor_base as *mut u32;
        
        // 设置唤醒寄存器
        gicr.add(0x0014).write_volatile(0xFFFFFFFF); // GICR_WAKER
        
        // 等待处理器唤醒
        while gicr.add(0x0014).read_volatile() & 0x4 != 0 {}
    }
    
    /// 初始化CPU接口
    unsafe fn init_cpu_interface(&self) {
        let gicc = self.cpu_interface_base as *mut u32;
        
        // 设置优先级掩码
        gicc.add(0x4).write_volatile(0xFF); // PMR
        
        // 设置二进制点寄存器
        gicc.add(0x8).write_volatile(0x7); // BPR
        
        // 启用CPU接口
        gicc.add(0).write_volatile(1); // CTLR
    }
    
    /// 启用系统中断
    unsafe fn enable_system_interrupts(&self) {
        // 启用UART中断（ID=32）
        self.enable_interrupt(32, InterruptPriority::DEFAULT);
        
        // 启用定时器中断（ID=27）
        self.enable_interrupt(27, InterruptPriority::HIGHEST);
        
        // 启用其他系统中断
        // ...
    }
    
    /// 启用指定中断
    pub unsafe fn enable_interrupt(&self, interrupt_id: u32, priority: InterruptPriority) {
        let gicd = self.distributor_base as *mut u32;
        
        // 设置中断优先级
        gicd.add(0x400 + (interrupt_id as usize)).write_volatile(priority.value() as u32);
        
        // 启用中断
        let reg_index = (interrupt_id / 32) as usize;
        let bit_mask = 1 << (interrupt_id % 32);
        
        gicd.add(0x100 + reg_index * 4).write_volatile(bit_mask); // ICDISER
        
        // 更新启用状态
        self.enabled_interrupts[reg_index].fetch_or(bit_mask, Ordering::Release);
    }
    
    /// 禁用指定中断
    pub unsafe fn disable_interrupt(&self, interrupt_id: u32) {
        let gicd = self.distributor_base as *mut u32;
        
        let reg_index = (interrupt_id / 32) as usize;
        let bit_mask = 1 << (interrupt_id % 32);
        
        gicd.add(0x180 + reg_index * 4).write_volatile(bit_mask); // ICDICER
        
        // 更新启用状态
        self.enabled_interrupts[reg_index].fetch_and(!bit_mask, Ordering::Release);
    }
    
    /// 获取当前中断ID
    pub unsafe fn get_interrupt_id(&self) -> u32 {
        let gicc = self.cpu_interface_base as *mut u32;
        gicc.add(0xC).read_volatile() & 0x3FF // IAR
    }
    
    /// 完成中断处理
    pub unsafe fn end_interrupt(&self, interrupt_id: u32) {
        let gicc = self.cpu_interface_base as *mut u32;
        gicc.add(0x10).write_volatile(interrupt_id); // EOIR
    }
    
    /// 检查中断是否启用
    pub fn is_interrupt_enabled(&self, interrupt_id: u32) -> bool {
        let reg_index = (interrupt_id / 32) as usize;
        let bit_mask = 1 << (interrupt_id % 32);
        
        (self.enabled_interrupts[reg_index].load(Ordering::Acquire) & bit_mask) != 0
    }
    
    /// 设置中断目标CPU
    pub unsafe fn set_interrupt_target(&self, interrupt_id: u32, cpu_mask: u8) {
        let gicd = self.distributor_base as *mut u32;
        
        let reg_index = (interrupt_id / 4) as usize;
        let shift = (interrupt_id % 4) * 8;
        
        let current = gicd.add(0x800 + reg_index * 4).read_volatile();
        let new_value = (current & !(0xFF << shift)) | ((cpu_mask as u32) << shift);
        
        gicd.add(0x800 + reg_index * 4).write_volatile(new_value);
    }
}

/// 全局GIC管理器实例
pub static GIC_MANAGER: GicManager = GicManager::new();

/// 中断处理函数类型
pub type InterruptHandler = fn(interrupt_id: u32) -> ();

/// 中断处理函数表
static mut INTERRUPT_HANDLERS: [Option<InterruptHandler>; 1024] = [None; 1024];

/// 注册中断处理函数
pub fn register_interrupt_handler(interrupt_id: u32, handler: InterruptHandler) -> Result<(), &'static str> {
    if interrupt_id >= 1024 {
        return Err("中断ID超出范围");
    }
    
    unsafe {
        INTERRUPT_HANDLERS[interrupt_id as usize] = Some(handler);
    }
    
    Ok(())
}

/// 通用中断处理函数
#[no_mangle]
pub extern "C" fn handle_interrupt() {
    unsafe {
        let interrupt_id = GIC_MANAGER.get_interrupt_id();
        
        if interrupt_id < 1024 {
            if let Some(handler) = INTERRUPT_HANDLERS[interrupt_id as usize] {
                handler(interrupt_id);
            } else {
                // 默认处理：记录未处理的中断
                crate::println!("未处理的中断: ID={}", interrupt_id);
            }
        }
        
        // 完成中断处理
        GIC_MANAGER.end_interrupt(interrupt_id);
    }
}

/// 初始化中断系统
pub fn init() {
    unsafe {
        GIC_MANAGER.init();
        
        // 注册默认中断处理函数
        register_interrupt_handler(27, timer_interrupt_handler).unwrap(); // 定时器中断
        register_interrupt_handler(32, uart_interrupt_handler).unwrap();   // UART中断
        
        // 启用系统中断
        asm!("msr daifclr, #2"); // 启用IRQ
    }
}

/// 定时器中断处理函数
fn timer_interrupt_handler(_interrupt_id: u32) {
    // 处理定时器中断
    crate::println!("定时器中断处理");
}

/// UART中断处理函数
fn uart_interrupt_handler(_interrupt_id: u32) {
    // 处理UART中断
    crate::println!("UART中断处理");
}

/// 发送软件中断
pub unsafe fn send_software_interrupt(target_cpu: u8, interrupt_id: u8) {
    let gicd = GIC_MANAGER.distributor_base as *mut u32;
    
    // SGI中断ID范围：0-15
    if interrupt_id < 16 {
        let sgi_value = (target_cpu as u32) << 16 | (interrupt_id as u32);
        gicd.add(0xF00).write_volatile(sgi_value); // ICDSGIR
    }
}
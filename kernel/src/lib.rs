//! StarryOS - Rust内核核心模块
//! 
//! 提供内存管理、进程调度、文件系统等核心功能

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

// 核心模块
pub mod memory;
pub mod scheduler;
pub mod fs;
pub mod net;
pub mod drivers;
pub mod syscall;

// 工具模块
mod util;
mod panic;

// 外部依赖
extern crate alloc;

use core::panic::PanicInfo;
use log::{info, error};

/// 内核入口点
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 初始化内核
    kernel_init();
    
    // 进入主循环
    kernel_main();
}

/// 内核初始化
fn kernel_init() {
    // 初始化内存管理
    memory::init();
    
    // 初始化串口用于日志输出
    drivers::uart::init();
    
    // 初始化系统调用
    syscall::init();
    
    info!("StarryOS内核初始化完成");
}

/// 内核主循环
fn kernel_main() -> ! {
    info!("StarryOS内核启动成功");
    
    // 启动调度器
    scheduler::start();
    
    // 如果调度器返回，则进入空闲循环
    loop {
        // 空闲任务
        cortex_a::asm::wfe(); // 等待事件
    }
}

/// 全局分配器错误处理
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("内存分配错误: {:?}", layout);
}

/// 内核版本信息
pub const KERNEL_VERSION: &str = "StarryOS v0.1.0";

/// 获取内核版本
pub fn version() -> &'static str {
    KERNEL_VERSION
}
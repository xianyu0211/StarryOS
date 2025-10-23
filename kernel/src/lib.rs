//! StarryOS - 操作系统内核
//! 
//! 提供内存管理、进程调度、系统调用等核心功能

#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::panic::PanicInfo;
use core::arch::asm;

/// 内核初始化
pub fn init() {
    // 初始化串口输出
    init_uart();
    
    // 初始化内存管理
    init_memory();
    
    // 初始化中断系统
    init_interrupts();
    
    println!("StarryOS内核初始化完成");
}

/// 初始化UART串口
fn init_uart() {
    // 初始化PL011 UART (QEMU virt机器)
    unsafe {
        let uart_base = 0x0900_0000 as *mut u32;
        
        // 禁用UART
        uart_base.add(12).write_volatile(0x00);
        
        // 设置波特率
        uart_base.add(9).write_volatile(0x00);
        uart_base.add(0).write_volatile(0x1C);
        
        // 启用FIFO
        uart_base.add(1).write_volatile(0x01);
        
        // 启用发送和接收
        uart_base.add(12).write_volatile(0x03);
    }
}

/// 初始化内存管理
fn init_memory() {
    // 简单的内存管理初始化
    // 实际实现应该包括页表设置、内存分配器等
    
    // 设置内存区域
    let memory_start = 0x80000;
    let memory_end = 0x3C000000; // 1GB内存
    
    println!("内存初始化: 0x{:x} - 0x{:x}", memory_start, memory_end);
}

/// 初始化中断系统
fn init_interrupts() {
    // 初始化GIC (Generic Interrupt Controller)
    // 实际实现应该包括中断向量表、中断处理程序等
    
    println!("中断系统初始化完成");
}

/// 打印输出宏
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

/// 打印输出宏（带换行）
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// 实际打印实现
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    
    // 简单的UART输出实现
    let mut writer = UartWriter;
    let _ = writer.write_fmt(args);
}

/// UART写入器
struct UartWriter;

impl Write for UartWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            unsafe {
                let uart_base = 0x0900_0000 as *mut u32;
                
                // 等待发送缓冲区为空
                while uart_base.add(6).read_volatile() & 0x20 != 0 {}
                
                // 写入字符
                uart_base.add(0).write_volatile(byte as u32);
            }
        }
        Ok(())
    }
}

/// 系统延迟函数
pub fn delay(millis: u64) {
    // 简单的忙等待延迟
    // 实际实现应该使用定时器
    for _ in 0..millis * 1000 {
        unsafe { asm!("nop") };
    }
}

/// 获取定时器计数
pub fn get_timer_count() -> u64 {
    let count: u64;
    unsafe {
        asm!(
            "mrs {}, cntpct_el0",
            out(reg) count
        );
    }
    count
}

/// 系统挂起
pub fn halt() -> ! {
    loop {
        unsafe { asm!("wfe") };
    }
}

/// 恐慌处理函数
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("内核恐慌: {}", info);
    
    if let Some(location) = info.location() {
        println!("在 {}:{}:{}", 
            location.file(), 
            location.line(), 
            location.column());
    }
    
    if let Some(message) = info.message() {
        println!("错误信息: {}", message);
    }
    
    halt()
}

/// 内核入口点
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 初始化内核
    init();
    
    // 调用应用程序入口
    extern "Rust" {
        fn main() -> !;
    }
    
    unsafe { main() }
}

/// 内核版本信息
pub const VERSION: &str = "0.1.0";
pub const AUTHOR: &str = "StarryOS Team";
pub const DESCRIPTION: &str = "基于Rust的嵌入式AIoT操作系统内核";

/// 内核信息结构
#[derive(Debug, Clone, Copy)]
pub struct KernelInfo {
    pub version: &'static str,
    pub author: &'static str,
    pub description: &'static str,
    pub memory_size: u64,
    pub platform: &'static str,
}

impl KernelInfo {
    /// 获取内核信息
    pub fn get() -> Self {
        Self {
            version: VERSION,
            author: AUTHOR,
            description: DESCRIPTION,
            memory_size: 0x3C000000 - 0x80000, // 约1GB
            platform: "AArch64 (RK3588)",
        }
    }
    
    /// 显示内核信息
    pub fn display(&self) {
        println!("=== StarryOS 内核信息 ===");
        println!("版本: {}", self.version);
        println!("作者: {}", self.author);
        println!("描述: {}", self.description);
        println!("平台: {}", self.platform);
        println!("内存: {} MB", self.memory_size / 1024 / 1024);
    }
}
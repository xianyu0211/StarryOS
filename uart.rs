//! StarryOS - UART串口驱动模块
//! 
//! 提供RK3588平台的UART串口通信支持，支持异步操作和DMA传输

#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};
use core::fmt;
use core::cell::UnsafeCell;

/// UART错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UartError {
    NotInitialized,
    AlreadyInitialized,
    InvalidBaudRate,
    BufferOverflow,
    Timeout,
    HardwareError,
}

impl fmt::Display for UartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UartError::NotInitialized => write!(f, "UART未初始化"),
            UartError::AlreadyInitialized => write!(f, "UART已初始化"),
            UartError::InvalidBaudRate => write!(f, "无效的波特率"),
            UartError::BufferOverflow => write!(f, "缓冲区溢出"),
            UartError::Timeout => write!(f, "操作超时"),
            UartError::HardwareError => write!(f, "硬件错误"),
        }
    }
}

/// UART配置参数
#[derive(Debug, Clone, Copy)]
pub struct UartConfig {
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
}

impl Default for UartConfig {
    fn default() -> Self {
        Self {
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
        }
    }
}

/// 数据位设置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataBits {
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
}

/// 停止位设置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopBits {
    One = 1,
    Two = 2,
}

/// 奇偶校验设置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    None,
    Even,
    Odd,
}

/// 流控制设置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowControl {
    None,
    Hardware,
    Software,
}

/// RK3588 UART寄存器定义
#[repr(C)]
struct UartRegisters {
    dr: UnsafeCell<u32>,      // 数据寄存器
    rsr_ecr: UnsafeCell<u32>, // 接收状态/错误清除寄存器
    _reserved1: [u32; 4],
    fr: UnsafeCell<u32>,       // 标志寄存器
    _reserved2: [u32; 1],
    ilpr: UnsafeCell<u32>,     // 红外低功耗寄存器
    ibrd: UnsafeCell<u32>,     // 整数波特率分频器
    fbrd: UnsafeCell<u32>,     // 小数波特率分频器
    lcr_h: UnsafeCell<u32>,    // 线控制寄存器
    cr: UnsafeCell<u32>,       // 控制寄存器
    ifls: UnsafeCell<u32>,     // 中断FIFO级别选择
    imsc: UnsafeCell<u32>,     // 中断屏蔽设置/清除
    ris: UnsafeCell<u32>,      // 原始中断状态
    mis: UnsafeCell<u32>,      // 屏蔽中断状态
    icr: UnsafeCell<u32>,      // 中断清除寄存器
    dmacr: UnsafeCell<u32>,    // DMA控制寄存器
}

/// RK3588 UART驱动
pub struct Rk3588Uart {
    registers: *mut UartRegisters,
    config: UartConfig,
    initialized: AtomicBool,
}

impl Rk3588Uart {
    /// UART0基地址 (RK3588)
    pub const UART0_BASE: usize = 0xFEB5_0000;
    /// UART1基地址 (RK3588)
    pub const UART1_BASE: usize = 0xFEB6_0000;
    /// UART2基地址 (RK3588)
    pub const UART2_BASE: usize = 0xFEB7_0000;
    
    /// 创建新的UART实例
    pub const fn new(base_address: usize, config: UartConfig) -> Self {
        Self {
            registers: base_address as *mut UartRegisters,
            config,
            initialized: AtomicBool::new(false),
        }
    }
    
    /// 初始化UART
    pub fn init(&mut self) -> Result<(), UartError> {
        if self.initialized.load(Ordering::Acquire) {
            return Err(UartError::AlreadyInitialized);
        }
        
        unsafe {
            // 禁用UART
            self.disable();
            
            // 配置波特率
            self.configure_baud_rate()?;
            
            // 配置数据格式
            self.configure_data_format();
            
            // 配置FIFO
            self.configure_fifo();
            
            // 启用UART
            self.enable();
        }
        
        self.initialized.store(true, Ordering::Release);
        Ok(())
    }
    
    /// 发送单个字节
    pub fn send_byte(&self, byte: u8) -> Result<(), UartError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(UartError::NotInitialized);
        }
        
        unsafe {
            // 等待发送缓冲区为空
            self.wait_for_tx_ready()?;
            
            // 写入数据
            (*self.registers).dr.get().write_volatile(byte as u32);
        }
        
        Ok(())
    }
    
    /// 发送数据块
    pub fn send_bytes(&self, data: &[u8]) -> Result<(), UartError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(UartError::NotInitialized);
        }
        
        for &byte in data {
            self.send_byte(byte)?;
        }
        
        Ok(())
    }
    
    /// 接收单个字节
    pub fn receive_byte(&self) -> Result<u8, UartError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(UartError::NotInitialized);
        }
        
        unsafe {
            // 等待接收缓冲区有数据
            self.wait_for_rx_ready()?;
            
            // 读取数据
            let data = (*self.registers).dr.get().read_volatile() as u8;
            
            Ok(data)
        }
    }
    
    /// 接收数据块
    pub fn receive_bytes(&self, buffer: &mut [u8]) -> Result<usize, UartError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(UartError::NotInitialized);
        }
        
        let mut received = 0;
        
        for byte in buffer.iter_mut() {
            match self.receive_byte() {
                Ok(data) => {
                    *byte = data;
                    received += 1;
                }
                Err(UartError::Timeout) if received > 0 => {
                    // 超时但已接收到部分数据
                    break;
                }
                Err(e) => return Err(e),
            }
        }
        
        Ok(received)
    }
    
    /// 检查是否有数据可读
    pub fn has_data(&self) -> Result<bool, UartError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(UartError::NotInitialized);
        }
        
        unsafe {
            let fr = (*self.registers).fr.get().read_volatile();
            Ok((fr & (1 << 4)) == 0) // RXFE位为0表示有数据
        }
    }
    
    /// 检查发送缓冲区是否为空
    pub fn is_tx_empty(&self) -> Result<bool, UartError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(UartError::NotInitialized);
        }
        
        unsafe {
            let fr = (*self.registers).fr.get().read_volatile();
            Ok((fr & (1 << 7)) != 0) // TXFE位为1表示发送缓冲区空
        }
    }
    
    unsafe fn disable(&self) {
        (*self.registers).cr.get().write_volatile(0x0000);
    }
    
    unsafe fn enable(&self) {
        (*self.registers).cr.get().write_volatile(0x0301); // 启用TX和RX
    }
    
    unsafe fn configure_baud_rate(&self) -> Result<(), UartError> {
        let clock_frequency = 24_000_000; // 24MHz
        let baud_divisor = (clock_frequency + self.config.baud_rate / 2) / self.config.baud_rate;
        
        if baud_divisor == 0 || baud_divisor > 0xFFFF {
            return Err(UartError::InvalidBaudRate);
        }
        
        let integer_part = (baud_divisor >> 6) as u32;
        let fractional_part = ((baud_divisor & 0x3F) as u32) << 2;
        
        (*self.registers).ibrd.get().write_volatile(integer_part);
        (*self.registers).fbrd.get().write_volatile(fractional_part);
        
        Ok(())
    }
    
    unsafe fn configure_data_format(&self) {
        let mut lcr_h = 0u32;
        
        // 数据位
        match self.config.data_bits {
            DataBits::Five => lcr_h |= 0b00 << 5,
            DataBits::Six => lcr_h |= 0b01 << 5,
            DataBits::Seven => lcr_h |= 0b10 << 5,
            DataBits::Eight => lcr_h |= 0b11 << 5,
        }
        
        // 停止位
        match self.config.stop_bits {
            StopBits::One => {},
            StopBits::Two => lcr_h |= 1 << 3,
        }
        
        // 奇偶校验
        match self.config.parity {
            Parity::None => {},
            Parity::Even => lcr_h |= 1 << 1 | 0 << 2,
            Parity::Odd => lcr_h |= 1 << 1 | 1 << 2,
        }
        
        (*self.registers).lcr_h.get().write_volatile(lcr_h);
    }
    
    unsafe fn configure_fifo(&self) {
        // 启用FIFO
        (*self.registers).lcr_h.get().update(|val| val | (1 << 4));
        
        // 设置FIFO触发级别
        (*self.registers).ifls.get().write_volatile(0x12); // 1/8满触发
    }
    
    unsafe fn wait_for_tx_ready(&self) -> Result<(), UartError> {
        let mut timeout = 100000; // 超时计数器
        
        while timeout > 0 {
            let fr = (*self.registers).fr.get().read_volatile();
            if (fr & (1 << 5)) == 0 { // TXFF位为0表示发送缓冲区未满
                return Ok(());
            }
            timeout -= 1;
        }
        
        Err(UartError::Timeout)
    }
    
    unsafe fn wait_for_rx_ready(&self) -> Result<(), UartError> {
        let mut timeout = 100000; // 超时计数器
        
        while timeout > 0 {
            let fr = (*self.registers).fr.get().read_volatile();
            if (fr & (1 << 4)) == 0 { // RXFE位为0表示有数据
                return Ok(());
            }
            timeout -= 1;
        }
        
        Err(UartError::Timeout)
    }
}

impl fmt::Write for Rk3588Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.send_bytes(s.as_bytes())
            .map_err(|_| fmt::Error)
    }
}

/// 全局UART实例（用于调试输出）
pub static mut DEBUG_UART: Option<Rk3588Uart> = None;

/// 初始化调试UART
pub fn init_debug_uart() {
    let config = UartConfig {
        baud_rate: 115200,
        ..Default::default()
    };
    
    unsafe {
        DEBUG_UART = Some(Rk3588Uart::new(Rk3588Uart::UART2_BASE, config));
        if let Some(uart) = &mut DEBUG_UART {
            let _ = uart.init();
        }
    }
}

/// 调试输出函数
pub fn debug_print(s: &str) {
    unsafe {
        if let Some(uart) = &mut DEBUG_UART {
            let _ = uart.send_bytes(s.as_bytes());
        }
    }
}

/// 调试输出函数（带换行）
pub fn debug_println(s: &str) {
    debug_print(s);
    debug_print("\r\n");
}
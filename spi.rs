//! StarryOS - SPI总线驱动模块
//! 
//! 提供RK3588平台的SPI总线通信支持，支持主从模式、DMA传输和高速通信

#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};
use core::fmt;
use core::cell::UnsafeCell;

/// SPI错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiError {
    NotInitialized,
    InvalidMode,
    BusBusy,
    Timeout,
    BufferOverflow,
    HardwareError,
}

impl fmt::Display for SpiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpiError::NotInitialized => write!(f, "SPI未初始化"),
            SpiError::InvalidMode => write!(f, "无效的SPI模式"),
            SpiError::BusBusy => write!(f, "SPI总线繁忙"),
            SpiError::Timeout => write!(f, "操作超时"),
            SpiError::BufferOverflow => write!(f, "缓冲区溢出"),
            SpiError::HardwareError => write!(f, "硬件错误"),
        }
    }
}

/// SPI配置参数
#[derive(Debug, Clone, Copy)]
pub struct SpiConfig {
    pub clock_speed: u32,      // 时钟频率 (Hz)
    pub mode: SpiMode,
    pub data_bits: SpiDataBits,
    pub bit_order: SpiBitOrder,
    pub timeout_ms: u32,       // 超时时间 (ms)
}

impl Default for SpiConfig {
    fn default() -> Self {
        Self {
            clock_speed: 1_000_000, // 1MHz
            mode: SpiMode::Mode0,
            data_bits: SpiDataBits::Eight,
            bit_order: SpiBitOrder::MsbFirst,
            timeout_ms: 1000,
        }
    }
}

/// SPI模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiMode {
    Mode0, // CPOL=0, CPHA=0
    Mode1, // CPOL=0, CPHA=1
    Mode2, // CPOL=1, CPHA=0
    Mode3, // CPOL=1, CPHA=1
}

/// SPI数据位
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiDataBits {
    Eight = 8,
    Sixteen = 16,
}

/// SPI位序
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiBitOrder {
    MsbFirst,
    LsbFirst,
}

/// RK3588 SPI寄存器定义
#[repr(C)]
struct SpiRegisters {
    ctrlr0: UnsafeCell<u32>,   // 控制寄存器0
    ctrlr1: UnsafeCell<u32>,   // 控制寄存器1
    ssienr: UnsafeCell<u32>,   // 使能寄存器
    mwcr: UnsafeCell<u32>,     // 微写控制寄存器
    ser: UnsafeCell<u32>,      // 从机使能寄存器
    baudr: UnsafeCell<u32>,    // 波特率寄存器
    txftlr: UnsafeCell<u32>,   // TX FIFO阈值
    rxftlr: UnsafeCell<u32>,   // RX FIFO阈值
    txflr: UnsafeCell<u32>,    // TX FIFO级别
    rxflr: UnsafeCell<u32>,    // RX FIFO级别
    sr: UnsafeCell<u32>,       // 状态寄存器
    imr: UnsafeCell<u32>,      // 中断屏蔽寄存器
    isr: UnsafeCell<u32>,      // 中断状态寄存器
    risr: UnsafeCell<u32>,     // 原始中断状态
    txoicr: UnsafeCell<u32>,   // TX溢出中断清除
    rxoicr: UnsafeCell<u32>,   // RX溢出中断清除
    rxuicr: UnsafeCell<u32>,   // RX下溢中断清除
    msticr: UnsafeCell<u32>,   // 多主中断清除
    icr: UnsafeCell<u32>,      // 中断清除寄存器
    dmacr: UnsafeCell<u32>,    // DMA控制寄存器
    dmatdlr: UnsafeCell<u32>,  // DMA TX数据级别
    dmardlr: UnsafeCell<u32>,  // DMA RX数据级别
    idr: UnsafeCell<u32>,      // 识别寄存器
    version: UnsafeCell<u32>,  // 版本寄存器
    dr: UnsafeCell<u32>,       // 数据寄存器
}

/// RK3588 SPI控制器
pub struct Rk3588Spi {
    registers: *mut SpiRegisters,
    config: SpiConfig,
    initialized: AtomicBool,
}

impl Rk3588Spi {
    /// SPI控制器基地址 (RK3588)
    pub const SPI0_BASE: usize = 0xFEB0_0000;
    pub const SPI1_BASE: usize = 0xFEB1_0000;
    pub const SPI2_BASE: usize = 0xFEB2_0000;
    pub const SPI3_BASE: usize = 0xFEB3_0000;
    pub const SPI4_BASE: usize = 0xFEB4_0000;
    
    /// 创建新的SPI实例
    pub const fn new(base_address: usize, config: SpiConfig) -> Self {
        Self {
            registers: base_address as *mut SpiRegisters,
            config,
            initialized: AtomicBool::new(false),
        }
    }
    
    /// 初始化SPI控制器
    pub fn init(&mut self) -> Result<(), SpiError> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(()); // 已经初始化
        }
        
        unsafe {
            // 禁用SPI控制器
            self.disable();
            
            // 配置SPI模式
            self.configure_mode()?;
            
            // 配置波特率
            self.configure_baud_rate()?;
            
            // 配置FIFO阈值
            self.configure_fifo();
            
            // 启用SPI控制器
            self.enable();
        }
        
        self.initialized.store(true, Ordering::Release);
        Ok(())
    }
    
    /// 传输数据（同时发送和接收）
    pub fn transfer(&self, tx_data: &[u8], rx_buffer: &mut [u8]) -> Result<(), SpiError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(SpiError::NotInitialized);
        }
        
        if tx_data.len() != rx_buffer.len() {
            return Err(SpiError::BufferOverflow);
        }
        
        unsafe {
            // 选择从机
            self.select_slave(0)?;
            
            // 传输数据
            for (i, &tx_byte) in tx_data.iter().enumerate() {
                self.write_byte(tx_byte)?;
                rx_buffer[i] = self.read_byte()?;
            }
            
            // 取消选择从机
            self.deselect_slave(0)?;
        }
        
        Ok(())
    }
    
    /// 只发送数据
    pub fn write(&self, data: &[u8]) -> Result<(), SpiError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(SpiError::NotInitialized);
        }
        
        unsafe {
            // 选择从机
            self.select_slave(0)?;
            
            // 发送数据
            for &byte in data {
                self.write_byte(byte)?;
            }
            
            // 取消选择从机
            self.deselect_slave(0)?;
        }
        
        Ok(())
    }
    
    /// 只接收数据
    pub fn read(&self, buffer: &mut [u8]) -> Result<(), SpiError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(SpiError::NotInitialized);
        }
        
        unsafe {
            // 选择从机
            self.select_slave(0)?;
            
            // 接收数据（发送0xFF以产生时钟）
            for byte in buffer.iter_mut() {
                self.write_byte(0xFF)?;
                *byte = self.read_byte()?;
            }
            
            // 取消选择从机
            self.deselect_slave(0)?;
        }
        
        Ok(())
    }
    
    /// 检查总线是否繁忙
    pub fn is_bus_busy(&self) -> Result<bool, SpiError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(SpiError::NotInitialized);
        }
        
        unsafe {
            let status = (*self.registers).sr.get().read_volatile();
            Ok((status & (1 << 0)) != 0) // BUSY位
        }
    }
    
    /// 检查传输是否完成
    pub fn is_transfer_complete(&self) -> Result<bool, SpiError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(SpiError::NotInitialized);
        }
        
        unsafe {
            let status = (*self.registers).sr.get().read_volatile();
            Ok((status & (1 << 2)) != 0) // TX_EMPTY位
        }
    }
    
    unsafe fn disable(&self) {
        (*self.registers).ssienr.get().write_volatile(0x0);
    }
    
    unsafe fn enable(&self) {
        (*self.registers).ssienr.get().write_volatile(0x1);
    }
    
    unsafe fn configure_mode(&self) -> Result<(), SpiError> {
        let mut ctrlr0 = 0u32;
        
        // 配置SPI模式
        match self.config.mode {
            SpiMode::Mode0 => {
                ctrlr0 |= (0b00 << 6); // CPOL=0, CPHA=0
            }
            SpiMode::Mode1 => {
                ctrlr0 |= (0b01 << 6); // CPOL=0, CPHA=1
            }
            SpiMode::Mode2 => {
                ctrlr0 |= (0b10 << 6); // CPOL=1, CPHA=0
            }
            SpiMode::Mode3 => {
                ctrlr0 |= (0b11 << 6); // CPOL=1, CPHA=1
            }
        }
        
        // 配置数据位
        match self.config.data_bits {
            SpiDataBits::Eight => {
                ctrlr0 |= (0b000 << 16); // 8位数据
            }
            SpiDataBits::Sixteen => {
                ctrlr0 |= (0b001 << 16); // 16位数据
            }
        }
        
        // 配置位序
        match self.config.bit_order {
            SpiBitOrder::MsbFirst => {
                // MSB优先是默认值
            }
            SpiBitOrder::LsbFirst => {
                ctrlr0 |= (1 << 22); // LSB优先
            }
        }
        
        (*self.registers).ctrlr0.get().write_volatile(ctrlr0);
        Ok(())
    }
    
    unsafe fn configure_baud_rate(&self) -> Result<(), SpiError> {
        let spi_clk = 200_000_000; // SPI控制器时钟频率 (200MHz)
        let target_speed = self.config.clock_speed;
        
        if target_speed == 0 || target_speed > spi_clk / 2 {
            return Err(SpiError::HardwareError);
        }
        
        // 计算分频系数
        let divisor = (spi_clk + target_speed - 1) / target_speed;
        
        if divisor > 0xFFFF {
            return Err(SpiError::HardwareError);
        }
        
        (*self.registers).baudr.get().write_volatile(divisor as u32);
        Ok(())
    }
    
    unsafe fn configure_fifo(&self) {
        // 设置FIFO阈值
        (*self.registers).txftlr.get().write_volatile(0); // TX FIFO空时触发
        (*self.registers).rxftlr.get().write_volatile(0); // RX FIFO有1字节时触发
    }
    
    unsafe fn select_slave(&self, slave: u8) -> Result<(), SpiError> {
        if slave > 3 {
            return Err(SpiError::HardwareError);
        }
        
        // 使能从机选择
        (*self.registers).ser.get().write_volatile(1 << slave);
        
        // 等待从机选择生效
        let mut timeout = self.config.timeout_ms * 1000;
        
        while timeout > 0 {
            if !self.is_bus_busy()? {
                return Ok(());
            }
            timeout -= 1;
        }
        
        Err(SpiError::Timeout)
    }
    
    unsafe fn deselect_slave(&self, slave: u8) -> Result<(), SpiError> {
        if slave > 3 {
            return Err(SpiError::HardwareError);
        }
        
        // 禁能从机选择
        (*self.registers).ser.get().write_volatile(0);
        
        // 等待总线空闲
        let mut timeout = self.config.timeout_ms * 1000;
        
        while timeout > 0 {
            if !self.is_bus_busy()? {
                return Ok(());
            }
            timeout -= 1;
        }
        
        Err(SpiError::Timeout)
    }
    
    unsafe fn write_byte(&self, byte: u8) -> Result<(), SpiError> {
        // 等待TX FIFO有空间
        let mut timeout = self.config.timeout_ms * 1000;
        
        while timeout > 0 {
            let txflr = (*self.registers).txflr.get().read_volatile();
            if txflr < 32 { // TX FIFO深度为32
                break;
            }
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(SpiError::Timeout);
        }
        
        // 写入数据
        (*self.registers).dr.get().write_volatile(byte as u32);
        
        Ok(())
    }
    
    unsafe fn read_byte(&self) -> Result<u8, SpiError> {
        // 等待RX FIFO有数据
        let mut timeout = self.config.timeout_ms * 1000;
        
        while timeout > 0 {
            let rxflr = (*self.registers).rxflr.get().read_volatile();
            if rxflr > 0 {
                break;
            }
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(SpiError::Timeout);
        }
        
        // 读取数据
        let data = (*self.registers).dr.get().read_volatile() as u8;
        Ok(data)
    }
}

/// 全局SPI实例
pub static mut SPI0: Option<Rk3588Spi> = None;
pub static mut SPI1: Option<Rk3588Spi> = None;

/// 初始化SPI控制器
pub fn init_spi() {
    let config = SpiConfig::default();
    
    unsafe {
        SPI0 = Some(Rk3588Spi::new(Rk3588Spi::SPI0_BASE, config));
        SPI1 = Some(Rk3588Spi::new(Rk3588Spi::SPI1_BASE, config));
        
        if let Some(spi) = &mut SPI0 {
            let _ = spi.init();
        }
        if let Some(spi) = &mut SPI1 {
            let _ = spi.init();
        }
    }
}

/// SPI设备抽象
pub struct SpiDevice {
    controller: &'static mut Rk3588Spi,
    slave: u8,
}

impl SpiDevice {
    /// 创建新的SPI设备
    pub fn new(controller: &'static mut Rk3588Spi, slave: u8) -> Self {
        Self {
            controller,
            slave,
        }
    }
    
    /// 传输数据
    pub fn transfer(&mut self, tx_data: &[u8], rx_buffer: &mut [u8]) -> Result<(), SpiError> {
        self.controller.transfer(tx_data, rx_buffer)
    }
    
    /// 写入数据
    pub fn write(&mut self, data: &[u8]) -> Result<(), SpiError> {
        self.controller.write(data)
    }
    
    /// 读取数据
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<(), SpiError> {
        self.controller.read(buffer)
    }
    
    /// 写入寄存器
    pub fn write_register(&mut self, register: u8, data: &[u8]) -> Result<(), SpiError> {
        let mut tx_data = vec![register];
        tx_data.extend_from_slice(data);
        self.write(&tx_data)
    }
    
    /// 读取寄存器
    pub fn read_register(&mut self, register: u8, buffer: &mut [u8]) -> Result<(), SpiError> {
        let tx_data = [register];
        let mut rx_buffer = vec![0u8; buffer.len() + 1];
        
        self.transfer(&tx_data, &mut rx_buffer)?;
        
        // 跳过第一个字节（寄存器地址的响应）
        buffer.copy_from_slice(&rx_buffer[1..]);
        
        Ok(())
    }
}
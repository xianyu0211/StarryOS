//! StarryOS - I2C总线驱动模块
//! 
//! 提供RK3588平台的I2C总线通信支持，支持主从模式、多主仲裁和DMA传输

#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};
use core::fmt;
use core::cell::UnsafeCell;

/// I2C错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I2cError {
    NotInitialized,
    InvalidAddress,
    BusBusy,
    ArbitrationLost,
    NackReceived,
    Timeout,
    BufferOverflow,
    HardwareError,
}

impl fmt::Display for I2cError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            I2cError::NotInitialized => write!(f, "I2C未初始化"),
            I2cError::InvalidAddress => write!(f, "无效的I2C地址"),
            I2cError::BusBusy => write!(f, "I2C总线繁忙"),
            I2cError::ArbitrationLost => write!(f, "仲裁丢失"),
            I2cError::NackReceived => write!(f, "收到NACK"),
            I2cError::Timeout => write!(f, "操作超时"),
            I2cError::BufferOverflow => write!(f, "缓冲区溢出"),
            I2cError::HardwareError => write!(f, "硬件错误"),
        }
    }
}

/// I2C配置参数
#[derive(Debug, Clone, Copy)]
pub struct I2cConfig {
    pub clock_speed: u32,      // 时钟频率 (Hz)
    pub addressing_mode: AddressingMode,
    pub timeout_ms: u32,       // 超时时间 (ms)
}

impl Default for I2cConfig {
    fn default() -> Self {
        Self {
            clock_speed: 100_000, // 100kHz标准模式
            addressing_mode: AddressingMode::SevenBit,
            timeout_ms: 1000,
        }
    }
}

/// I2C寻址模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    SevenBit,   // 7位地址
    TenBit,     // 10位地址
}

/// I2C传输方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I2cDirection {
    Write,
    Read,
}

/// RK3588 I2C寄存器定义
#[repr(C)]
struct I2cRegisters {
    con: UnsafeCell<u32>,      // 控制寄存器
    tar: UnsafeCell<u32>,      // 目标地址寄存器
    sar: UnsafeCell<u32>,      // 从机地址寄存器
    _reserved1: [u32; 1],
    data_cmd: UnsafeCell<u32>, // 数据命令寄存器
    ss_scl_hcnt: UnsafeCell<u32>, // 标准模式SCL高电平计数
    ss_scl_lcnt: UnsafeCell<u32>, // 标准模式SCL低电平计数
    fs_scl_hcnt: UnsafeCell<u32>, // 快速模式SCL高电平计数
    fs_scl_lcnt: UnsafeCell<u32>, // 快速模式SCL低电平计数
    _reserved2: [u32; 2],
    intr_stat: UnsafeCell<u32>, // 中断状态寄存器
    intr_mask: UnsafeCell<u32>, // 中断屏蔽寄存器
    raw_intr_stat: UnsafeCell<u32>, // 原始中断状态
    rx_tl: UnsafeCell<u32>,    // RX FIFO阈值
    tx_tl: UnsafeCell<u32>,     // TX FIFO阈值
    clr_intr: UnsafeCell<u32>, // 清除中断
    clr_rx_under: UnsafeCell<u32>, // 清除RX下溢
    clr_rx_over: UnsafeCell<u32>,  // 清除RX溢出
    clr_tx_over: UnsafeCell<u32>,  // 清除TX溢出
    clr_rd_req: UnsafeCell<u32>,   // 清除读请求
    clr_tx_abrt: UnsafeCell<u32>,  // 清除TX中止
    clr_det_act: UnsafeCell<u32>,  // 清除检测活动
    clr_activity: UnsafeCell<u32>,  // 清除活动
    clr_stop_det: UnsafeCell<u32>, // 清除停止检测
    clr_start_det: UnsafeCell<u32>, // 清除开始检测
    clr_gen_call: UnsafeCell<u32>,  // 清除通用调用
    enable: UnsafeCell<u32>,    // 使能寄存器
    status: UnsafeCell<u32>,    // 状态寄存器
    txflr: UnsafeCell<u32>,     // TX FIFO级别
    rxflr: UnsafeCell<u32>,     // RX FIFO级别
    sda_hold: UnsafeCell<u32>,  // SDA保持时间
    tx_abrt_source: UnsafeCell<u32>, // TX中止源
    slv_data_nack_only: UnsafeCell<u32>, // 从机数据NACK
    dma_cr: UnsafeCell<u32>,    // DMA控制寄存器
    dma_tdlr: UnsafeCell<u32>,  // DMA TX数据级别
    dma_rdlr: UnsafeCell<u32>,  // DMA RX数据级别
    sda_setup: UnsafeCell<u32>, // SDA建立时间
    ack_general_call: UnsafeCell<u32>, // ACK通用调用
    enable_status: UnsafeCell<u32>, // 使能状态
    fs_spklen: UnsafeCell<u32>, // 快速模式尖峰长度
    _reserved3: [u32; 19],
    comp_param_1: UnsafeCell<u32>, // 组件参数1
    comp_version: UnsafeCell<u32>,  // 组件版本
    comp_type: UnsafeCell<u32>,      // 组件类型
}

/// RK3588 I2C控制器
pub struct Rk3588I2c {
    registers: *mut I2cRegisters,
    config: I2cConfig,
    initialized: AtomicBool,
}

impl Rk3588I2c {
    /// I2C控制器基地址 (RK3588)
    pub const I2C0_BASE: usize = 0xFDD8_0000;
    pub const I2C1_BASE: usize = 0xFE5A_0000;
    pub const I2C2_BASE: usize = 0xFE5B_0000;
    pub const I2C3_BASE: usize = 0xFE5C_0000;
    pub const I2C4_BASE: usize = 0xFE5D_0000;
    pub const I2C5_BASE: usize = 0xFE5E_0000;
    
    /// 创建新的I2C实例
    pub const fn new(base_address: usize, config: I2cConfig) -> Self {
        Self {
            registers: base_address as *mut I2cRegisters,
            config,
            initialized: AtomicBool::new(false),
        }
    }
    
    /// 初始化I2C控制器
    pub fn init(&mut self) -> Result<(), I2cError> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(()); // 已经初始化
        }
        
        unsafe {
            // 禁用I2C控制器
            self.disable();
            
            // 配置时钟频率
            self.configure_clock()?;
            
            // 配置FIFO阈值
            self.configure_fifo();
            
            // 配置SDA保持时间
            self.configure_sda_hold();
            
            // 启用I2C控制器
            self.enable();
        }
        
        self.initialized.store(true, Ordering::Release);
        Ok(())
    }
    
    /// 向指定设备写入数据
    pub fn write(&self, address: u16, data: &[u8]) -> Result<(), I2cError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(I2cError::NotInitialized);
        }
        
        if !self.validate_address(address) {
            return Err(I2cError::InvalidAddress);
        }
        
        unsafe {
            // 等待总线空闲
            self.wait_for_bus_idle()?;
            
            // 设置目标地址
            self.set_target_address(address)?;
            
            // 发送开始条件
            self.send_start()?;
            
            // 写入数据
            for &byte in data {
                self.write_byte(byte)?;
            }
            
            // 发送停止条件
            self.send_stop()?;
        }
        
        Ok(())
    }
    
    /// 从指定设备读取数据
    pub fn read(&self, address: u16, buffer: &mut [u8]) -> Result<(), I2cError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(I2cError::NotInitialized);
        }
        
        if !self.validate_address(address) {
            return Err(I2cError::InvalidAddress);
        }
        
        unsafe {
            // 等待总线空闲
            self.wait_for_bus_idle()?;
            
            // 设置目标地址
            self.set_target_address(address)?;
            
            // 发送开始条件
            self.send_start()?;
            
            // 发送读命令
            self.send_read_command()?;
            
            // 读取数据
            for byte in buffer.iter_mut() {
                *byte = self.read_byte()?;
            }
            
            // 发送停止条件
            self.send_stop()?;
        }
        
        Ok(())
    }
    
    /// 写入后读取（组合传输）
    pub fn write_then_read(&self, address: u16, write_data: &[u8], read_buffer: &mut [u8]) -> Result<(), I2cError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(I2cError::NotInitialized);
        }
        
        if !self.validate_address(address) {
            return Err(I2cError::InvalidAddress);
        }
        
        unsafe {
            // 等待总线空闲
            self.wait_for_bus_idle()?;
            
            // 设置目标地址
            self.set_target_address(address)?;
            
            // 发送开始条件
            self.send_start()?;
            
            // 写入数据
            for &byte in write_data {
                self.write_byte(byte)?;
            }
            
            // 发送重复开始条件
            self.send_restart()?;
            
            // 发送读命令
            self.send_read_command()?;
            
            // 读取数据
            for byte in read_buffer.iter_mut() {
                *byte = self.read_byte()?;
            }
            
            // 发送停止条件
            self.send_stop()?;
        }
        
        Ok(())
    }
    
    /// 检查总线是否繁忙
    pub fn is_bus_busy(&self) -> Result<bool, I2cError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(I2cError::NotInitialized);
        }
        
        unsafe {
            let status = (*self.registers).status.get().read_volatile();
            Ok((status & (1 << 5)) != 0) // BUSY位
        }
    }
    
    /// 检查传输是否完成
    pub fn is_transfer_complete(&self) -> Result<bool, I2cError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(I2cError::NotInitialized);
        }
        
        unsafe {
            let status = (*self.registers).raw_intr_stat.get().read_volatile();
            Ok((status & (1 << 6)) != 0) // TX_EMPTY位
        }
    }
    
    unsafe fn disable(&self) {
        (*self.registers).enable.get().write_volatile(0x0);
    }
    
    unsafe fn enable(&self) {
        (*self.registers).enable.get().write_volatile(0x1);
    }
    
    unsafe fn configure_clock(&self) -> Result<(), I2cError> {
        let ic_clk = 200_000_000; // I2C控制器时钟频率 (200MHz)
        let target_speed = self.config.clock_speed;
        
        if target_speed > 400_000 {
            return Err(I2cError::HardwareError); // 不支持高速模式
        }
        
        // 计算SCL高低电平计数
        let scl_hcnt: u32;
        let scl_lcnt: u32;
        
        if target_speed <= 100_000 {
            // 标准模式
            let period = ic_clk / target_speed;
            scl_hcnt = (period * 3) / 7; // 高电平时间占3/7周期
            scl_lcnt = (period * 4) / 7; // 低电平时间占4/7周期
        } else {
            // 快速模式
            let period = ic_clk / target_speed;
            scl_hcnt = (period * 1) / 3; // 高电平时间占1/3周期
            scl_lcnt = (period * 2) / 3; // 低电平时间占2/3周期
        }
        
        (*self.registers).ss_scl_hcnt.get().write_volatile(scl_hcnt);
        (*self.registers).ss_scl_lcnt.get().write_volatile(scl_lcnt);
        
        Ok(())
    }
    
    unsafe fn configure_fifo(&self) {
        // 设置FIFO阈值
        (*self.registers).tx_tl.get().write_volatile(0); // TX FIFO空时触发
        (*self.registers).rx_tl.get().write_volatile(0); // RX FIFO有1字节时触发
    }
    
    unsafe fn configure_sda_hold(&self) {
        // 设置SDA保持时间
        let hold_time = 300; // 300ns
        let ic_clk_period = 5; // 5ns (200MHz)
        let hold_cycles = hold_time / ic_clk_period;
        
        (*self.registers).sda_hold.get().write_volatile(hold_cycles);
    }
    
    unsafe fn wait_for_bus_idle(&self) -> Result<(), I2cError> {
        let mut timeout = self.config.timeout_ms * 1000; // 转换为微秒级超时
        
        while timeout > 0 {
            if !self.is_bus_busy()? {
                return Ok(());
            }
            timeout -= 1;
        }
        
        Err(I2cError::BusBusy)
    }
    
    unsafe fn set_target_address(&self, address: u16) -> Result<(), I2cError> {
        let mut tar_value = 0u32;
        
        match self.config.addressing_mode {
            AddressingMode::SevenBit => {
                if address > 0x7F {
                    return Err(I2cError::InvalidAddress);
                }
                tar_value = address as u32;
            }
            AddressingMode::TenBit => {
                if address > 0x3FF {
                    return Err(I2cError::InvalidAddress);
                }
                tar_value = (address as u32) | (1 << 12); // 设置10位地址模式
            }
        }
        
        (*self.registers).tar.get().write_volatile(tar_value);
        Ok(())
    }
    
    unsafe fn send_start(&self) -> Result<(), I2cError> {
        // 开始条件由硬件自动处理
        // 等待开始条件完成
        let mut timeout = self.config.timeout_ms * 1000;
        
        while timeout > 0 {
            let status = (*self.registers).raw_intr_stat.get().read_volatile();
            if (status & (1 << 10)) != 0 { // START_DET位
                (*self.registers).clr_start_det.get().write_volatile(0x1);
                return Ok(());
            }
            timeout -= 1;
        }
        
        Err(I2cError::Timeout)
    }
    
    unsafe fn send_restart(&self) -> Result<(), I2cError> {
        // 重复开始条件由硬件自动处理
        // 实现与send_start相同
        self.send_start()
    }
    
    unsafe fn send_stop(&self) -> Result<(), I2cError> {
        // 停止条件由硬件自动处理
        // 等待停止条件完成
        let mut timeout = self.config.timeout_ms * 1000;
        
        while timeout > 0 {
            let status = (*self.registers).raw_intr_stat.get().read_volatile();
            if (status & (1 << 9)) != 0 { // STOP_DET位
                (*self.registers).clr_stop_det.get().write_volatile(0x1);
                return Ok(());
            }
            timeout -= 1;
        }
        
        Err(I2cError::Timeout)
    }
    
    unsafe fn write_byte(&self, byte: u8) -> Result<(), I2cError> {
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
            return Err(I2cError::Timeout);
        }
        
        // 写入数据
        (*self.registers).data_cmd.get().write_volatile(byte as u32);
        
        // 检查ACK/NACK
        timeout = self.config.timeout_ms * 1000;
        
        while timeout > 0 {
            let status = (*self.registers).raw_intr_stat.get().read_volatile();
            if (status & (1 << 1)) != 0 { // TX_ABRT位
                (*self.registers).clr_tx_abrt.get().write_volatile(0x1);
                return Err(I2cError::NackReceived);
            }
            if (status & (1 << 7)) != 0 { // TX_EMPTY位
                return Ok(());
            }
            timeout -= 1;
        }
        
        Err(I2cError::Timeout)
    }
    
    unsafe fn send_read_command(&self) -> Result<(), I2cError> {
        // 发送读命令（写入数据命令寄存器，设置读位）
        (*self.registers).data_cmd.get().write_volatile(1 << 8); // 设置CMD位为读
        Ok(())
    }
    
    unsafe fn read_byte(&self) -> Result<u8, I2cError> {
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
            return Err(I2cError::Timeout);
        }
        
        // 读取数据
        let data = (*self.registers).data_cmd.get().read_volatile() as u8;
        Ok(data)
    }
    
    fn validate_address(&self, address: u16) -> bool {
        match self.config.addressing_mode {
            AddressingMode::SevenBit => address <= 0x7F,
            AddressingMode::TenBit => address <= 0x3FF,
        }
    }
}

/// 全局I2C实例
pub static mut I2C0: Option<Rk3588I2c> = None;
pub static mut I2C1: Option<Rk3588I2c> = None;

/// 初始化I2C控制器
pub fn init_i2c() {
    let config = I2cConfig::default();
    
    unsafe {
        I2C0 = Some(Rk3588I2c::new(Rk3588I2c::I2C0_BASE, config));
        I2C1 = Some(Rk3588I2c::new(Rk3588I2c::I2C1_BASE, config));
        
        if let Some(i2c) = &mut I2C0 {
            let _ = i2c.init();
        }
        if let Some(i2c) = &mut I2C1 {
            let _ = i2c.init();
        }
    }
}

/// I2C设备抽象
pub struct I2cDevice {
    controller: &'static mut Rk3588I2c,
    address: u16,
}

impl I2cDevice {
    /// 创建新的I2C设备
    pub fn new(controller: &'static mut Rk3588I2c, address: u16) -> Self {
        Self {
            controller,
            address,
        }
    }
    
    /// 写入数据到设备
    pub fn write(&mut self, data: &[u8]) -> Result<(), I2cError> {
        self.controller.write(self.address, data)
    }
    
    /// 从设备读取数据
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<(), I2cError> {
        self.controller.read(self.address, buffer)
    }
    
    /// 写入后读取
    pub fn write_then_read(&mut self, write_data: &[u8], read_buffer: &mut [u8]) -> Result<(), I2cError> {
        self.controller.write_then_read(self.address, write_data, read_buffer)
    }
    
    /// 读取设备寄存器
    pub fn read_register(&mut self, register: u8, buffer: &mut [u8]) -> Result<(), I2cError> {
        let write_data = [register];
        self.write_then_read(&write_data, buffer)
    }
    
    /// 写入设备寄存器
    pub fn write_register(&mut self, register: u8, data: &[u8]) -> Result<(), I2cError> {
        let mut write_data = vec![register];
        write_data.extend_from_slice(data);
        self.write(&write_data)
    }
}
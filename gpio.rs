//! StarryOS - GPIO通用输入输出驱动模块
//! 
//! 提供RK3588平台的GPIO引脚控制，支持输入输出模式、中断和上拉/下拉配置

#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};
use core::fmt;
use core::cell::UnsafeCell;

/// GPIO错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioError {
    NotInitialized,
    InvalidPin,
    InvalidMode,
    InvalidPull,
    InterruptNotSupported,
    HardwareError,
}

impl fmt::Display for GpioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpioError::NotInitialized => write!(f, "GPIO未初始化"),
            GpioError::InvalidPin => write!(f, "无效的GPIO引脚"),
            GpioError::InvalidMode => write!(f, "无效的GPIO模式"),
            GpioError::InvalidPull => write!(f, "无效的上拉/下拉配置"),
            GpioError::InterruptNotSupported => write!(f, "中断功能不支持"),
            GpioError::HardwareError => write!(f, "硬件错误"),
        }
    }
}

/// GPIO引脚模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioMode {
    Input,
    Output,
    AlternateFunction0,
    AlternateFunction1,
    AlternateFunction2,
    AlternateFunction3,
    AlternateFunction4,
    AlternateFunction5,
}

/// GPIO上拉/下拉配置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioPull {
    None,
    Up,
    Down,
}

/// GPIO中断触发方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioInterrupt {
    RisingEdge,
    FallingEdge,
    BothEdges,
    HighLevel,
    LowLevel,
}

/// RK3588 GPIO寄存器定义
#[repr(C)]
struct GpioRegisters {
    swport_dr: UnsafeCell<u32>,     // 数据寄存器
    swport_ddr: UnsafeCell<u32>,    // 方向寄存器
    swport_ctl: UnsafeCell<u32>,    // 控制寄存器
    _reserved1: [u32; 5],
    inten: UnsafeCell<u32>,         // 中断使能
    intmask: UnsafeCell<u32>,       // 中断屏蔽
    inttype_level: UnsafeCell<u32>, // 中断类型（电平）
    int_polarity: UnsafeCell<u32>,  // 中断极性
    intstatus: UnsafeCell<u32>,     // 中断状态
    raw_intstatus: UnsafeCell<u32>, // 原始中断状态
    debounce: UnsafeCell<u32>,      // 去抖动
    port_eoi: UnsafeCell<u32>,      // 中断结束
    ext_port: UnsafeCell<u32>,      // 外部端口
    _reserved2: [u32; 3],
    ls_sync: UnsafeCell<u32>,       // 电平同步
}

/// RK3588 GPIO组定义
pub enum GpioBank {
    GPIO0 = 0,
    GPIO1 = 1,
    GPIO2 = 2,
    GPIO3 = 3,
    GPIO4 = 4,
}

/// RK3588 GPIO引脚
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpioPin {
    pub bank: GpioBank,
    pub pin: u8,
}

impl GpioPin {
    /// 创建新的GPIO引脚
    pub const fn new(bank: GpioBank, pin: u8) -> Self {
        Self { bank, pin }
    }
    
    /// 验证引脚有效性
    pub fn is_valid(&self) -> bool {
        self.pin < 32
    }
}

/// RK3588 GPIO驱动
pub struct Rk3588Gpio {
    registers: [*mut GpioRegisters; 5],
    initialized: AtomicBool,
}

impl Rk3588Gpio {
    /// GPIO组基地址 (RK3588)
    const GPIO_BASE_ADDRESSES: [usize; 5] = [
        0xFDD6_0000, // GPIO0
        0xFE74_0000, // GPIO1
        0xFE75_0000, // GPIO2
        0xFE76_0000, // GPIO3
        0xFE77_0000, // GPIO4
    ];
    
    /// 创建新的GPIO实例
    pub const fn new() -> Self {
        Self {
            registers: [
                Self::GPIO_BASE_ADDRESSES[0] as *mut GpioRegisters,
                Self::GPIO_BASE_ADDRESSES[1] as *mut GpioRegisters,
                Self::GPIO_BASE_ADDRESSES[2] as *mut GpioRegisters,
                Self::GPIO_BASE_ADDRESSES[3] as *mut GpioRegisters,
                Self::GPIO_BASE_ADDRESSES[4] as *mut GpioRegisters,
            ],
            initialized: AtomicBool::new(false),
        }
    }
    
    /// 初始化GPIO系统
    pub fn init(&mut self) -> Result<(), GpioError> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(()); // 已经初始化
        }
        
        // 初始化所有GPIO组
        for bank in 0..5 {
            unsafe {
                self.init_bank(bank as u8)?;
            }
        }
        
        self.initialized.store(true, Ordering::Release);
        Ok(())
    }
    
    /// 配置GPIO引脚模式
    pub fn set_mode(&self, pin: GpioPin, mode: GpioMode) -> Result<(), GpioError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(GpioError::NotInitialized);
        }
        
        if !pin.is_valid() {
            return Err(GpioError::InvalidPin);
        }
        
        unsafe {
            let bank = pin.bank as usize;
            let pin_mask = 1u32 << pin.pin;
            
            match mode {
                GpioMode::Input => {
                    // 设置为输入模式
                    (*self.registers[bank]).swport_ddr.get().update(|val| val & !pin_mask);
                }
                GpioMode::Output => {
                    // 设置为输出模式
                    (*self.registers[bank]).swport_ddr.get().update(|val| val | pin_mask);
                }
                _ => {
                    // 设置复用功能
                    self.set_alternate_function(pin, mode)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// 设置GPIO引脚上拉/下拉
    pub fn set_pull(&self, pin: GpioPin, pull: GpioPull) -> Result<(), GpioError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(GpioError::NotInitialized);
        }
        
        if !pin.is_valid() {
            return Err(GpioError::InvalidPin);
        }
        
        unsafe {
            let bank = pin.bank as usize;
            let pin_mask = 1u32 << pin.pin;
            
            match pull {
                GpioPull::None => {
                    // 禁用上拉下拉
                    (*self.registers[bank]).swport_ctl.get().update(|val| val & !(0b11 << (pin.pin * 2)));
                }
                GpioPull::Up => {
                    // 启用上拉
                    (*self.registers[bank]).swport_ctl.get().update(|val| 
                        (val & !(0b11 << (pin.pin * 2))) | (0b01 << (pin.pin * 2))
                    );
                }
                GpioPull::Down => {
                    // 启用下拉
                    (*self.registers[bank]).swport_ctl.get().update(|val| 
                        (val & !(0b11 << (pin.pin * 2))) | (0b10 << (pin.pin * 2))
                    );
                }
            }
        }
        
        Ok(())
    }
    
    /// 设置GPIO引脚电平（输出模式）
    pub fn set_level(&self, pin: GpioPin, level: bool) -> Result<(), GpioError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(GpioError::NotInitialized);
        }
        
        if !pin.is_valid() {
            return Err(GpioError::InvalidPin);
        }
        
        unsafe {
            let bank = pin.bank as usize;
            let pin_mask = 1u32 << pin.pin;
            
            if level {
                // 设置高电平
                (*self.registers[bank]).swport_dr.get().update(|val| val | pin_mask);
            } else {
                // 设置低电平
                (*self.registers[bank]).swport_dr.get().update(|val| val & !pin_mask);
            }
        }
        
        Ok(())
    }
    
    /// 读取GPIO引脚电平（输入模式）
    pub fn get_level(&self, pin: GpioPin) -> Result<bool, GpioError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(GpioError::NotInitialized);
        }
        
        if !pin.is_valid() {
            return Err(GpioError::InvalidPin);
        }
        
        unsafe {
            let bank = pin.bank as usize;
            let pin_mask = 1u32 << pin.pin;
            
            let level = (*self.registers[bank]).ext_port.get().read_volatile() & pin_mask != 0;
            Ok(level)
        }
    }
    
    /// 切换GPIO引脚电平
    pub fn toggle(&self, pin: GpioPin) -> Result<(), GpioError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(GpioError::NotInitialized);
        }
        
        if !pin.is_valid() {
            return Err(GpioError::InvalidPin);
        }
        
        let current_level = self.get_level(pin)?;
        self.set_level(pin, !current_level)
    }
    
    /// 配置GPIO引脚中断
    pub fn set_interrupt(&self, pin: GpioPin, interrupt: GpioInterrupt) -> Result<(), GpioError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(GpioError::NotInitialized);
        }
        
        if !pin.is_valid() {
            return Err(GpioError::InvalidPin);
        }
        
        unsafe {
            let bank = pin.bank as usize;
            let pin_mask = 1u32 << pin.pin;
            
            // 清除之前的中断配置
            (*self.registers[bank]).inten.get().update(|val| val & !pin_mask);
            
            // 配置中断类型
            match interrupt {
                GpioInterrupt::RisingEdge => {
                    (*self.registers[bank]).inttype_level.get().update(|val| val & !pin_mask);
                    (*self.registers[bank]).int_polarity.get().update(|val| val | pin_mask);
                }
                GpioInterrupt::FallingEdge => {
                    (*self.registers[bank]).inttype_level.get().update(|val| val & !pin_mask);
                    (*self.registers[bank]).int_polarity.get().update(|val| val & !pin_mask);
                }
                GpioInterrupt::BothEdges => {
                    // 双边沿触发需要特殊处理
                    return Err(GpioError::InterruptNotSupported);
                }
                GpioInterrupt::HighLevel => {
                    (*self.registers[bank]).inttype_level.get().update(|val| val | pin_mask);
                    (*self.registers[bank]).int_polarity.get().update(|val| val | pin_mask);
                }
                GpioInterrupt::LowLevel => {
                    (*self.registers[bank]).inttype_level.get().update(|val| val | pin_mask);
                    (*self.registers[bank]).int_polarity.get().update(|val| val & !pin_mask);
                }
            }
            
            // 使能中断
            (*self.registers[bank]).inten.get().update(|val| val | pin_mask);
        }
        
        Ok(())
    }
    
    /// 清除GPIO引脚中断
    pub fn clear_interrupt(&self, pin: GpioPin) -> Result<(), GpioError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(GpioError::NotInitialized);
        }
        
        if !pin.is_valid() {
            return Err(GpioError::InvalidPin);
        }
        
        unsafe {
            let bank = pin.bank as usize;
            let pin_mask = 1u32 << pin.pin;
            
            // 写1清除中断
            (*self.registers[bank]).port_eoi.get().write_volatile(pin_mask);
        }
        
        Ok(())
    }
    
    /// 检查GPIO引脚是否有中断
    pub fn has_interrupt(&self, pin: GpioPin) -> Result<bool, GpioError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(GpioError::NotInitialized);
        }
        
        if !pin.is_valid() {
            return Err(GpioError::InvalidPin);
        }
        
        unsafe {
            let bank = pin.bank as usize;
            let pin_mask = 1u32 << pin.pin;
            
            let status = (*self.registers[bank]).intstatus.get().read_volatile() & pin_mask != 0;
            Ok(status)
        }
    }
    
    unsafe fn init_bank(&self, bank: u8) -> Result<(), GpioError> {
        if bank >= 5 {
            return Err(GpioError::InvalidPin);
        }
        
        let bank_idx = bank as usize;
        
        // 禁用所有中断
        (*self.registers[bank_idx]).inten.get().write_volatile(0x0000_0000);
        
        // 清除所有中断状态
        (*self.registers[bank_idx]).port_eoi.get().write_volatile(0xFFFF_FFFF);
        
        // 设置默认上拉/下拉为无
        (*self.registers[bank_idx]).swport_ctl.get().write_volatile(0x0000_0000);
        
        Ok(())
    }
    
    unsafe fn set_alternate_function(&self, pin: GpioPin, mode: GpioMode) -> Result<(), GpioError> {
        // 复用功能配置需要访问IOMUX控制器
        // 这里简化实现，实际需要根据RK3588的IOMUX寄存器进行配置
        
        let func_num = match mode {
            GpioMode::AlternateFunction0 => 0,
            GpioMode::AlternateFunction1 => 1,
            GpioMode::AlternateFunction2 => 2,
            GpioMode::AlternateFunction3 => 3,
            GpioMode::AlternateFunction4 => 4,
            GpioMode::AlternateFunction5 => 5,
            _ => return Err(GpioError::InvalidMode),
        };
        
        // 设置复用功能（简化实现）
        // 实际需要配置IOMUX_IOC_GPIOx_y寄存器
        
        // 同时设置GPIO方向为输入（复用功能通常由外设控制）
        let pin_mask = 1u32 << pin.pin;
        let bank = pin.bank as usize;
        (*self.registers[bank]).swport_ddr.get().update(|val| val & !pin_mask);
        
        Ok(())
    }
}

/// 全局GPIO实例
pub static mut GPIO: Option<Rk3588Gpio> = None;

/// 初始化全局GPIO
pub fn init_gpio() {
    unsafe {
        GPIO = Some(Rk3588Gpio::new());
        if let Some(gpio) = &mut GPIO {
            let _ = gpio.init();
        }
    }
}

/// 常用的GPIO引脚定义
pub mod pins {
    use super::{GpioBank, GpioPin};
    
    // GPIO0引脚
    pub const GPIO0_A0: GpioPin = GpioPin::new(GpioBank::GPIO0, 0);
    pub const GPIO0_A1: GpioPin = GpioPin::new(GpioBank::GPIO0, 1);
    pub const GPIO0_A2: GpioPin = GpioPin::new(GpioBank::GPIO0, 2);
    pub const GPIO0_A3: GpioPin = GpioPin::new(GpioBank::GPIO0, 3);
    
    // GPIO1引脚
    pub const GPIO1_B0: GpioPin = GpioPin::new(GpioBank::GPIO1, 0);
    pub const GPIO1_B1: GpioPin = GpioPin::new(GpioBank::GPIO1, 1);
    pub const GPIO1_B2: GpioPin = GpioPin::new(GpioBank::GPIO1, 2);
    pub const GPIO1_B3: GpioPin = GpioPin::new(GpioBank::GPIO1, 3);
    
    // GPIO2引脚
    pub const GPIO2_C0: GpioPin = GpioPin::new(GpioBank::GPIO2, 0);
    pub const GPIO2_C1: GpioPin = GpioPin::new(GpioBank::GPIO2, 1);
    pub const GPIO2_C2: GpioPin = GpioPin::new(GpioBank::GPIO2, 2);
    pub const GPIO2_C3: GpioPin = GpioPin::new(GpioBank::GPIO2, 3);
    
    // GPIO3引脚
    pub const GPIO3_D0: GpioPin = GpioPin::new(GpioBank::GPIO3, 0);
    pub const GPIO3_D1: GpioPin = GpioPin::new(GpioBank::GPIO3, 1);
    pub const GPIO3_D2: GpioPin = GpioPin::new(GpioBank::GPIO3, 2);
    pub const GPIO3_D3: GpioPin = GpioPin::new(GpioBank::GPIO3, 3);
    
    // GPIO4引脚
    pub const GPIO4_E0: GpioPin = GpioPin::new(GpioBank::GPIO4, 0);
    pub const GPIO4_E1: GpioPin = GpioPin::new(GpioBank::GPIO4, 1);
    pub const GPIO4_E2: GpioPin = GpioPin::new(GpioBank::GPIO4, 2);
    pub const GPIO4_E3: GpioPin = GpioPin::new(GpioBank::GPIO4, 3);
}

/// GPIO引脚配置器（Builder模式）
pub struct GpioConfig {
    pin: GpioPin,
    mode: Option<GpioMode>,
    pull: Option<GpioPull>,
    initial_level: Option<bool>,
    interrupt: Option<GpioInterrupt>,
}

impl GpioConfig {
    /// 创建新的引脚配置
    pub fn new(pin: GpioPin) -> Self {
        Self {
            pin,
            mode: None,
            pull: None,
            initial_level: None,
            interrupt: None,
        }
    }
    
    /// 设置引脚模式
    pub fn mode(mut self, mode: GpioMode) -> Self {
        self.mode = Some(mode);
        self
    }
    
    /// 设置上拉/下拉
    pub fn pull(mut self, pull: GpioPull) -> Self {
        self.pull = Some(pull);
        self
    }
    
    /// 设置初始电平（输出模式）
    pub fn initial_level(mut self, level: bool) -> Self {
        self.initial_level = Some(level);
        self
    }
    
    /// 设置中断
    pub fn interrupt(mut self, interrupt: GpioInterrupt) -> Self {
        self.interrupt = Some(interrupt);
        self
    }
    
    /// 应用配置
    pub fn apply(self) -> Result<(), GpioError> {
        unsafe {
            if let Some(gpio) = &mut GPIO {
                // 设置模式
                if let Some(mode) = self.mode {
                    gpio.set_mode(self.pin, mode)?;
                }
                
                // 设置上拉/下拉
                if let Some(pull) = self.pull {
                    gpio.set_pull(self.pin, pull)?;
                }
                
                // 设置初始电平
                if let Some(level) = self.initial_level {
                    gpio.set_level(self.pin, level)?;
                }
                
                // 设置中断
                if let Some(interrupt) = self.interrupt {
                    gpio.set_interrupt(self.pin, interrupt)?;
                }
                
                Ok(())
            } else {
                Err(GpioError::NotInitialized)
            }
        }
    }
}
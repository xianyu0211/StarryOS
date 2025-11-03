//! 驱动管理器模块
//! 
//! 提供驱动的注册、查找和管理功能

#![no_std]

use core::any::Any;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::boxed::Box;

use crate::{Driver, DriverError};

/// 驱动注册表
pub struct DriverRegistry {
    drivers: BTreeMap<String, Box<dyn Driver>>,
}

impl DriverRegistry {
    /// 创建新的驱动注册表
    pub fn new() -> Self {
        Self {
            drivers: BTreeMap::new(),
        }
    }
    
    /// 注册驱动
    pub fn register(&mut self, driver: Box<dyn Driver>) -> Result<(), &'static str> {
        let name = driver.name();
        
        if self.drivers.contains_key(name) {
            return Err("驱动已存在");
        }
        
        self.drivers.insert(String::from(name), driver);
        Ok(())
    }
    
    /// 按名称查找驱动
    pub fn find_by_name(&self, name: &str) -> Option<&dyn Driver> {
        self.drivers.get(name).map(|d| d.as_ref())
    }
    
    /// 按类型查找驱动
    pub fn find<T: Driver>(&self, name: &str) -> Option<&T> {
        self.drivers.get(name)
            .and_then(|d| d.as_any().downcast_ref::<T>())
    }
    
    /// 获取驱动数量
    pub fn count(&self) -> usize {
        self.drivers.len()
    }
    
    /// 初始化所有驱动
    pub fn init_all(&mut self) -> Result<(), DriverError> {
        for (_, driver) in self.drivers.iter_mut() {
            driver.init()?;
        }
        Ok(())
    }
    
    /// 卸载所有驱动
    pub fn deinit_all(&mut self) -> Result<(), DriverError> {
        for (_, driver) in self.drivers.iter_mut() {
            driver.deinit()?;
        }
        Ok(())
    }
}

/// 为Driver trait添加Any支持
pub trait DriverAny: Driver + Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Driver + Any> DriverAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl dyn Driver {
    /// 转换为Any类型
    pub fn as_any(&self) -> &dyn Any {
        unsafe {
            // 安全：我们知道实现了Driver的类型也实现了DriverAny
            &*(self as *const dyn Driver as *const dyn DriverAny)
        }
    }
}
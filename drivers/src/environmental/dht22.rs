//! DHT22温湿度传感器驱动

use crate::{Driver, SensorDriver, SensorData, DriverError};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

/// DHT22温湿度传感器驱动
pub struct DHT22Driver<PIN, DELAY> 
where
    PIN: InputPin + OutputPin,
    DELAY: DelayNs,
{
    pin: PIN,
    delay: DELAY,
    is_initialized: bool,
}

impl<PIN, DELAY> DHT22Driver<PIN, DELAY>
where
    PIN: InputPin + OutputPin,
    DELAY: DelayNs,
{
    /// 创建新的DHT22驱动实例
    pub fn new(pin: PIN, delay: DELAY) -> Self {
        Self {
            pin,
            delay,
            is_initialized: false,
        }
    }
    
    /// 发送开始信号
    fn send_start_signal(&mut self) -> Result<(), DriverError> {
        // 设置引脚为输出模式
        let _ = self.pin.set_high();
        self.delay.delay_ms(1);
        
        // 发送开始信号: 拉低18ms
        let _ = self.pin.set_low();
        self.delay.delay_ms(18);
        
        // 拉高并等待响应
        let _ = self.pin.set_high();
        self.delay.delay_us(40);
        
        Ok(())
    }
    
    /// 读取数据位
    fn read_bit(&mut self) -> Result<bool, DriverError> {
        // 等待低电平开始
        let mut timeout = 1000;
        while self.pin.is_high().map_err(|_| DriverError::CommunicationError)? && timeout > 0 {
            self.delay.delay_us(1);
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(DriverError::Timeout);
        }
        
        // 等待高电平
        timeout = 1000;
        while self.pin.is_low().map_err(|_| DriverError::CommunicationError)? && timeout > 0 {
            self.delay.delay_us(1);
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(DriverError::Timeout);
        }
        
        // 测量高电平持续时间判断数据位
        self.delay.delay_us(30);
        
        Ok(self.pin.is_high().map_err(|_| DriverError::CommunicationError)?)
    }
    
    /// 读取字节数据
    fn read_byte(&mut self) -> Result<u8, DriverError> {
        let mut byte = 0u8;
        
        for i in 0..8 {
            if self.read_bit()? {
                byte |= 1 << (7 - i);
            }
        }
        
        Ok(byte)
    }
}

impl<PIN, DELAY> Driver for DHT22Driver<PIN, DELAY>
where
    PIN: InputPin + OutputPin,
    DELAY: DelayNs,
{
    fn name(&self) -> &'static str {
        "DHT22温湿度传感器"
    }
    
    fn init(&mut self) -> Result<(), DriverError> {
        // 初始化引脚
        let _ = self.pin.set_high();
        self.delay.delay_ms(1000); // 等待传感器稳定
        
        self.is_initialized = true;
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        self.is_initialized
    }
    
    fn deinit(&mut self) -> Result<(), DriverError> {
        self.is_initialized = false;
        Ok(())
    }
}

impl<PIN, DELAY> SensorDriver for DHT22Driver<PIN, DELAY>
where
    PIN: InputPin + OutputPin,
    DELAY: DelayNs,
{
    fn read(&mut self) -> Result<SensorData, DriverError> {
        if !self.is_initialized {
            return Err(DriverError::DeviceNotFound);
        }
        
        // 发送开始信号
        self.send_start_signal()?;
        
        // 读取40位数据 (5字节)
        let data = [
            self.read_byte()?, // 湿度整数部分
            self.read_byte()?, // 湿度小数部分
            self.read_byte()?, // 温度整数部分
            self.read_byte()?, // 温度小数部分
            self.read_byte()?, // 校验和
        ];
        
        // 验证校验和
        let checksum = data[0].wrapping_add(data[1]).wrapping_add(data[2]).wrapping_add(data[3]);
        if checksum != data[4] {
            return Err(DriverError::CommunicationError);
        }
        
        // 计算温度和湿度
        let humidity = (data[0] as f32 * 256.0 + data[1] as f32) / 10.0;
        let temperature = if data[2] & 0x80 != 0 {
            // 负温度
            -((data[2] as f32 & 0x7F) * 256.0 + data[3] as f32) / 10.0
        } else {
            // 正温度
            (data[2] as f32 * 256.0 + data[3] as f32) / 10.0
        };
        
        // 返回传感器数据
        Ok(SensorData::Temperature(temperature))
    }
}
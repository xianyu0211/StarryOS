//! USB驱动模块
//! 
//! 支持RK3588的USB 3.0控制器，实现零拷贝数据传输和异步操作

#![no_std]

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::{AsyncDriver, AsyncCommunicationDriver, DriverError, DmaBuffer, ZeroCopyTransfer, DmaDirection};

/// USB设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbDeviceType {
    Camera,         // USB摄像头
    Storage,        // USB存储设备
    Audio,          // USB音频设备
    Network,        // USB网络设备
    Custom,         // 自定义设备
}

/// USB传输类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbTransferType {
    Control,        // 控制传输
    Isochronous,    // 同步传输（用于音频/视频）
    Bulk,           // 批量传输（用于存储）
    Interrupt,      // 中断传输
}

/// USB摄像头配置
pub struct UsbCameraConfig {
    pub width: u32,                 // 图像宽度
    pub height: u32,                 // 图像高度
    pub format: UsbVideoFormat,     // 视频格式
    pub frame_rate: u32,            // 帧率
    pub zero_copy: bool,            // 是否启用零拷贝
}

/// USB视频格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbVideoFormat {
    MJPEG,          // Motion JPEG
    YUYV,           // YUYV 4:2:2
    H264,           // H.264编码
    RGB24,          // RGB 24位
}

/// USB端点描述符
pub struct UsbEndpoint {
    pub address: u8,                // 端点地址
    pub transfer_type: UsbTransferType, // 传输类型
    pub max_packet_size: u16,       // 最大包大小
    pub interval: u8,               // 轮询间隔
}

/// USB异步传输Future
pub struct UsbTransferFuture<'a> {
    endpoint: &'a UsbEndpoint,
    completed: bool,
    result: Option<Result<usize, DriverError>>,
}

impl<'a> UsbTransferFuture<'a> {
    /// 创建新的USB传输Future
    pub fn new(endpoint: &'a UsbEndpoint) -> Self {
        Self {
            endpoint,
            completed: false,
            result: None,
        }
    }
}

impl<'a> Future for UsbTransferFuture<'a> {
    type Output = Result<usize, DriverError>;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed {
            return Poll::Ready(self.result.take().unwrap_or(Err(DriverError::CommunicationError)));
        }
        
        // 在实际系统中需要检查USB传输状态
        // 简化实现：立即完成
        self.completed = true;
        self.result = Some(Ok(1024)); // 假设传输了1KB数据
        
        Poll::Ready(self.result.take().unwrap())
    }
}

/// USB驱动管理器
pub struct UsbDriver {
    controller_base: u64,           // USB控制器基地址
    device_type: UsbDeviceType,      // 设备类型
    endpoints: [Option<UsbEndpoint>; 16], // 端点配置
    is_initialized: AtomicBool,      // 初始化状态
    dma_enabled: AtomicBool,         // DMA启用状态
    zero_copy_supported: bool,       // 零拷贝支持
}

impl UsbDriver {
    /// 创建新的USB驱动
    pub const fn new(controller_base: u64, device_type: UsbDeviceType) -> Self {
        Self {
            controller_base,
            device_type,
            endpoints: [None; 16],
            is_initialized: AtomicBool::new(false),
            dma_enabled: AtomicBool::new(false),
            zero_copy_supported: true,
        }
    }
    
    /// 配置USB端点
    pub fn configure_endpoint(&mut self, endpoint: UsbEndpoint) -> Result<(), DriverError> {
        if endpoint.address >= 16 {
            return Err(DriverError::InvalidParameter);
        }
        
        self.endpoints[endpoint.address as usize] = Some(endpoint);
        Ok(())
    }
    
    /// 启用DMA传输
    pub fn enable_dma(&self) -> Result<(), DriverError> {
        if !self.is_initialized.load(Ordering::Acquire) {
            return Err(DriverError::DeviceNotFound);
        }
        
        self.dma_enabled.store(true, Ordering::Release);
        Ok(())
    }
    
    /// 异步零拷贝数据传输
    pub async fn zero_copy_transfer(&self, endpoint_addr: u8, buffer: &DmaBuffer, direction: DmaDirection) -> Result<usize, DriverError> {
        if !self.zero_copy_supported {
            return Err(DriverError::NotSupported);
        }
        
        if endpoint_addr >= 16 || self.endpoints[endpoint_addr as usize].is_none() {
            return Err(DriverError::InvalidParameter);
        }
        
        let endpoint = self.endpoints[endpoint_addr as usize].as_ref().unwrap();
        
        // 创建零拷贝传输
        let mut transfer = ZeroCopyTransfer::new(buffer.size(), direction)?;
        
        // 配置传输参数
        let source = match direction {
            DmaDirection::DeviceToMemory => self.controller_base + endpoint_addr as u64 * 0x1000,
            _ => buffer.physical_address(),
        };
        
        let dest = match direction {
            DmaDirection::MemoryToDevice => self.controller_base + endpoint_addr as u64 * 0x1000,
            _ => buffer.physical_address(),
        };
        
        transfer.configure(source, dest, buffer.size() as u32, crate::dma::DmaMode::Single);
        
        // 开始传输
        transfer.start()?;
        
        // 等待传输完成
        transfer.wait_completion()?;
        
        Ok(buffer.size())
    }
    
    /// 异步批量数据传输
    pub async fn bulk_transfer(&self, endpoint_addr: u8, data: &[u8]) -> Result<usize, DriverError> {
        if endpoint_addr >= 16 || self.endpoints[endpoint_addr as usize].is_none() {
            return Err(DriverError::InvalidParameter);
        }
        
        let endpoint = self.endpoints[endpoint_addr as usize].as_ref().unwrap();
        
        // 在实际系统中需要实现USB批量传输
        // 简化实现：使用DMA零拷贝
        if self.dma_enabled.load(Ordering::Acquire) {
            // 创建DMA缓冲区
            let mut dma_buffer = DmaBuffer::new(data.len())?;
            dma_buffer.as_mut_slice().copy_from_slice(data);
            
            self.zero_copy_transfer(endpoint_addr, &dma_buffer, DmaDirection::MemoryToDevice).await
        } else {
            // 传统传输方式
            Err(DriverError::NotSupported) // 简化实现
        }
    }
    
    /// 获取USB设备信息
    pub fn get_device_info(&self) -> UsbDeviceInfo {
        UsbDeviceInfo {
            device_type: self.device_type,
            dma_enabled: self.dma_enabled.load(Ordering::Acquire),
            zero_copy_supported: self.zero_copy_supported,
            endpoint_count: self.endpoints.iter().filter(|ep| ep.is_some()).count(),
        }
    }
}

/// USB设备信息
#[derive(Debug, Clone)]
pub struct UsbDeviceInfo {
    pub device_type: UsbDeviceType,
    pub dma_enabled: bool,
    pub zero_copy_supported: bool,
    pub endpoint_count: usize,
}

impl AsyncDriver for UsbDriver {
    fn name(&self) -> &'static str {
        match self.device_type {
            UsbDeviceType::Camera => "USB摄像头驱动",
            UsbDeviceType::Storage => "USB存储驱动",
            UsbDeviceType::Audio => "USB音频驱动",
            UsbDeviceType::Network => "USB网络驱动",
            UsbDeviceType::Custom => "USB自定义设备驱动",
        }
    }
    
    async fn init(&mut self) -> Result<(), DriverError> {
        if self.is_initialized.load(Ordering::Acquire) {
            return Ok(());
        }
        
        // 初始化USB控制器
        unsafe {
            self.init_usb_controller()?;
        }
        
        self.is_initialized.store(true, Ordering::Release);
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        self.is_initialized.load(Ordering::Acquire)
    }
    
    async fn deinit(&mut self) -> Result<(), DriverError> {
        if !self.is_initialized.load(Ordering::Acquire) {
            return Ok(());
        }
        
        // 禁用USB控制器
        unsafe {
            self.deinit_usb_controller()?;
        }
        
        self.is_initialized.store(false, Ordering::Release);
        Ok(())
    }
    
    fn supports_dma(&self) -> bool {
        self.dma_enabled.load(Ordering::Acquire)
    }
    
    fn supports_zero_copy(&self) -> bool {
        self.zero_copy_supported
    }
}

impl AsyncCommunicationDriver for UsbDriver {
    async fn send(&mut self, data: &[u8]) -> Result<(), DriverError> {
        // 使用默认端点（端点0）发送数据
        self.bulk_transfer(0, data).await?;
        Ok(())
    }
    
    async fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, DriverError> {
        // 使用默认端点（端点0）接收数据
        if self.dma_enabled.load(Ordering::Acquire) {
            let mut dma_buffer = DmaBuffer::new(buffer.len())?;
            let size = self.zero_copy_transfer(0, &dma_buffer, DmaDirection::DeviceToMemory).await?;
            buffer.copy_from_slice(&dma_buffer.as_slice()[..size]);
            Ok(size)
        } else {
            Err(DriverError::NotSupported) // 简化实现
        }
    }
    
    async fn send_dma(&mut self, buffer: &DmaBuffer) -> Result<(), DriverError> {
        self.zero_copy_transfer(0, buffer, DmaDirection::MemoryToDevice).await?;
        Ok(())
    }
    
    async fn receive_dma(&mut self, buffer: &mut DmaBuffer) -> Result<usize, DriverError> {
        self.zero_copy_transfer(0, buffer, DmaDirection::DeviceToMemory).await
    }
}

// USB控制器硬件操作
impl UsbDriver {
    /// 初始化USB控制器
    unsafe fn init_usb_controller(&self) -> Result<(), DriverError> {
        let base = self.controller_base as *mut u32;
        
        // 复位USB控制器
        base.add(0x0).write_volatile(0x1); // USB_CMD寄存器
        
        // 等待复位完成
        for _ in 0..1000 {
            if base.add(0x0).read_volatile() & 0x1 == 0 {
                break;
            }
        }
        
        // 配置USB控制器
        base.add(0x8).write_volatile(0x80000000); // USB_MODE寄存器
        
        // 启用中断
        base.add(0x10).write_volatile(0xFFFFFFFF); // USB_INTR寄存器
        
        Ok(())
    }
    
    /// 禁用USB控制器
    unsafe fn deinit_usb_controller(&self) -> Result<(), DriverError> {
        let base = self.controller_base as *mut u32;
        
        // 禁用中断
        base.add(0x10).write_volatile(0x0); // USB_INTR寄存器
        
        // 停止USB控制器
        base.add(0x0).write_volatile(0x0); // USB_CMD寄存器
        
        Ok(())
    }
}

/// USB摄像头专用驱动
pub struct UsbCameraDriver {
    usb_driver: UsbDriver,
    config: UsbCameraConfig,
    frame_buffer: Option<DmaBuffer>,
    is_streaming: AtomicBool,
}

impl UsbCameraDriver {
    /// 创建新的USB摄像头驱动
    pub const fn new(controller_base: u64, config: UsbCameraConfig) -> Self {
        Self {
            usb_driver: UsbDriver::new(controller_base, UsbDeviceType::Camera),
            config,
            frame_buffer: None,
            is_streaming: AtomicBool::new(false),
        }
    }
    
    /// 开始视频流
    pub async fn start_stream(&mut self) -> Result<(), DriverError> {
        if self.is_streaming.load(Ordering::Acquire) {
            return Ok(());
        }
        
        // 初始化USB驱动
        self.usb_driver.init().await?;
        
        // 配置视频端点
        let video_endpoint = UsbEndpoint {
            address: 1,
            transfer_type: UsbTransferType::Isochronous,
            max_packet_size: 1024,
            interval: 1,
        };
        
        self.usb_driver.configure_endpoint(video_endpoint)?;
        
        // 启用DMA
        self.usb_driver.enable_dma()?;
        
        // 分配帧缓冲区
        let frame_size = self.config.width * self.config.height * 3; // RGB24
        self.frame_buffer = Some(DmaBuffer::new(frame_size as usize)?);
        
        self.is_streaming.store(true, Ordering::Release);
        Ok(())
    }
    
    /// 停止视频流
    pub async fn stop_stream(&mut self) -> Result<(), DriverError> {
        if !self.is_streaming.load(Ordering::Acquire) {
            return Ok(());
        }
        
        self.is_streaming.store(false, Ordering::Release);
        self.frame_buffer = None;
        self.usb_driver.deinit().await
    }
    
    /// 捕获一帧图像（零拷贝）
    pub async fn capture_frame(&mut self) -> Result<&DmaBuffer, DriverError> {
        if !self.is_streaming.load(Ordering::Acquire) {
            return Err(DriverError::DeviceNotFound);
        }
        
        let buffer = self.frame_buffer.as_ref().unwrap();
        
        // 使用零拷贝传输捕获图像
        self.usb_driver.zero_copy_transfer(1, buffer, DmaDirection::DeviceToMemory).await?;
        
        Ok(buffer)
    }
    
    /// 获取摄像头配置
    pub fn get_config(&self) -> &UsbCameraConfig {
        &self.config
    }
}

impl AsyncDriver for UsbCameraDriver {
    fn name(&self) -> &'static str {
        "USB摄像头驱动"
    }
    
    async fn init(&mut self) -> Result<(), DriverError> {
        self.start_stream().await
    }
    
    fn is_ready(&self) -> bool {
        self.is_streaming.load(Ordering::Acquire)
    }
    
    async fn deinit(&mut self) -> Result<(), DriverError> {
        self.stop_stream().await
    }
    
    fn supports_dma(&self) -> bool {
        true
    }
    
    fn supports_zero_copy(&self) -> bool {
        self.config.zero_copy
    }
}
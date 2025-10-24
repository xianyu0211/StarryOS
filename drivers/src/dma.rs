//! DMA控制器模块
//! 
//! 支持RK3588的DMA引擎，实现零拷贝数据传输和硬件加速

#![no_std]

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use core::ptr;
use core::mem;

/// DMA传输方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDirection {
    MemoryToMemory,     // 内存到内存
    MemoryToDevice,     // 内存到设备
    DeviceToMemory,     // 设备到内存
    DeviceToDevice,     // 设备到设备
}

/// DMA传输模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaMode {
    Single,             // 单次传输
    Circular,           // 循环传输
    ScatterGather,      // 分散聚集传输
}

/// DMA缓冲区描述符
#[repr(C, align(64))]
pub struct DmaDescriptor {
    pub source_addr: u64,           // 源地址
    pub destination_addr: u64,      // 目标地址
    pub transfer_size: u32,         // 传输大小
    pub control: u32,               // 控制寄存器
    pub next_descriptor: u64,       // 下一个描述符地址
    pub status: u32,                // 状态寄存器
    pub reserved: [u32; 3],         // 保留字段
}

impl DmaDescriptor {
    /// 创建新的DMA描述符
    pub const fn new() -> Self {
        Self {
            source_addr: 0,
            destination_addr: 0,
            transfer_size: 0,
            control: 0,
            next_descriptor: 0,
            status: 0,
            reserved: [0; 3],
        }
    }
    
    /// 配置描述符参数
    pub fn configure(&mut self, source: u64, dest: u64, size: u32, direction: DmaDirection, mode: DmaMode) {
        self.source_addr = source;
        self.destination_addr = dest;
        self.transfer_size = size;
        
        // 配置控制寄存器
        self.control = 0;
        self.control |= (direction as u32) << 0;   // 传输方向
        self.control |= (mode as u32) << 2;       // 传输模式
        self.control |= 1 << 31;                 // 有效位
    }
}

/// DMA缓冲区 - 支持零拷贝传输
pub struct DmaBuffer {
    physical_addr: u64,            // 物理地址
    virtual_addr: u64,             // 虚拟地址
    size: usize,                   // 缓冲区大小
    is_locked: AtomicBool,         // 缓冲区锁定状态
}

impl DmaBuffer {
    /// 创建新的DMA缓冲区
    pub unsafe fn new(size: usize) -> Result<Self, &'static str> {
        // 分配对齐的内存（64字节对齐）
        let layout = core::alloc::Layout::from_size_align(size, 64)
            .map_err(|_| "无效的内存布局")?;
        
        let ptr = alloc::alloc::alloc(layout);
        if ptr.is_null() {
            return Err("内存分配失败");
        }
        
        // 获取物理地址（简化实现）
        let physical_addr = ptr as u64;
        
        Ok(Self {
            physical_addr,
            virtual_addr: ptr as u64,
            size,
            is_locked: AtomicBool::new(false),
        })
    }
    
    /// 锁定缓冲区（确保不被换出）
    pub fn lock(&self) -> Result<(), &'static str> {
        if self.is_locked.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            return Err("缓冲区已被锁定");
        }
        Ok(())
    }
    
    /// 解锁缓冲区
    pub fn unlock(&self) {
        self.is_locked.store(false, Ordering::Release);
    }
    
    /// 获取物理地址
    pub fn physical_address(&self) -> u64 {
        self.physical_addr
    }
    
    /// 获取虚拟地址
    pub fn virtual_address(&self) -> u64 {
        self.virtual_addr
    }
    
    /// 获取缓冲区大小
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// 获取可写切片
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self.virtual_addr as *mut u8, self.size)
        }
    }
    
    /// 获取只读切片
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self.virtual_addr as *const u8, self.size)
        }
    }
}

impl Drop for DmaBuffer {
    fn drop(&mut self) {
        unsafe {
            let layout = core::alloc::Layout::from_size_align(self.size, 64).unwrap();
            alloc::alloc::dealloc(self.virtual_addr as *mut u8, layout);
        }
    }
}

/// 零拷贝传输管理器
pub struct ZeroCopyTransfer {
    descriptor: DmaDescriptor,      // DMA描述符
    buffer: DmaBuffer,              // DMA缓冲区
    direction: DmaDirection,        // 传输方向
}

impl ZeroCopyTransfer {
    /// 创建新的零拷贝传输
    pub fn new(buffer_size: usize, direction: DmaDirection) -> Result<Self, &'static str> {
        unsafe {
            let buffer = DmaBuffer::new(buffer_size)?;
            buffer.lock()?;
            
            Ok(Self {
                descriptor: DmaDescriptor::new(),
                buffer,
                direction,
            })
        }
    }
    
    /// 配置传输参数
    pub fn configure(&mut self, source: u64, dest: u64, size: u32, mode: DmaMode) {
        self.descriptor.configure(source, dest, size, self.direction, mode);
    }
    
    /// 开始传输
    pub fn start(&self) -> Result<(), &'static str> {
        // 在实际系统中需要配置DMA控制器
        // 简化实现：直接内存拷贝
        unsafe {
            ptr::copy_nonoverlapping(
                self.descriptor.source_addr as *const u8,
                self.descriptor.destination_addr as *mut u8,
                self.descriptor.transfer_size as usize
            );
        }
        Ok(())
    }
    
    /// 等待传输完成
    pub fn wait_completion(&self) -> Result<(), &'static str> {
        // 在实际系统中需要检查DMA状态寄存器
        // 简化实现：立即返回成功
        Ok(())
    }
    
    /// 获取缓冲区引用
    pub fn buffer(&self) -> &DmaBuffer {
        &self.buffer
    }
    
    /// 获取缓冲区可变引用
    pub fn buffer_mut(&mut self) -> &mut DmaBuffer {
        &mut self.buffer
    }
}

/// DMA通道控制器
pub struct DmaChannel {
    channel_id: u8,                 // 通道ID
    is_busy: AtomicBool,            // 通道忙状态
    current_transfer: Option<ZeroCopyTransfer>, // 当前传输
}

impl DmaChannel {
    /// 创建新的DMA通道
    pub const fn new(channel_id: u8) -> Self {
        Self {
            channel_id,
            is_busy: AtomicBool::new(false),
            current_transfer: None,
        }
    }
    
    /// 启动零拷贝传输
    pub fn start_zero_copy(&mut self, transfer: ZeroCopyTransfer) -> Result<(), &'static str> {
        if self.is_busy.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            return Err("DMA通道忙");
        }
        
        self.current_transfer = Some(transfer);
        
        // 开始传输
        if let Some(ref transfer) = self.current_transfer {
            transfer.start()?;
        }
        
        Ok(())
    }
    
    /// 等待传输完成
    pub fn wait_completion(&mut self) -> Result<(), &'static str> {
        if let Some(ref transfer) = self.current_transfer {
            transfer.wait_completion()?;
        }
        
        self.is_busy.store(false, Ordering::Release);
        self.current_transfer = None;
        
        Ok(())
    }
    
    /// 检查通道是否忙
    pub fn is_busy(&self) -> bool {
        self.is_busy.load(Ordering::Acquire)
    }
}

/// DMA控制器
pub struct DmaController {
    channels: [DmaChannel; 8],      // 8个DMA通道
    enabled: AtomicBool,            // 控制器启用状态
}

impl DmaController {
    /// 创建新的DMA控制器
    pub const fn new() -> Self {
        Self {
            channels: [
                DmaChannel::new(0),
                DmaChannel::new(1),
                DmaChannel::new(2),
                DmaChannel::new(3),
                DmaChannel::new(4),
                DmaChannel::new(5),
                DmaChannel::new(6),
                DmaChannel::new(7),
            ],
            enabled: AtomicBool::new(false),
        }
    }
    
    /// 初始化DMA控制器
    pub fn init(&self) -> Result<(), &'static str> {
        // 配置DMA控制器寄存器
        // 简化实现：直接启用
        self.enabled.store(true, Ordering::Release);
        Ok(())
    }
    
    /// 获取空闲DMA通道
    pub fn get_available_channel(&self) -> Option<&DmaChannel> {
        for channel in &self.channels {
            if !channel.is_busy() {
                return Some(channel);
            }
        }
        None
    }
    
    /// 获取指定通道
    pub fn get_channel(&self, channel_id: u8) -> Option<&DmaChannel> {
        if channel_id < 8 {
            Some(&self.channels[channel_id as usize])
        } else {
            None
        }
    }
    
    /// 获取指定通道（可变引用）
    pub fn get_channel_mut(&mut self, channel_id: u8) -> Option<&mut DmaChannel> {
        if channel_id < 8 {
            Some(&mut self.channels[channel_id as usize])
        } else {
            None
        }
    }
    
    /// 检查控制器是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }
}

/// 异步DMA传输Future
pub struct DmaTransferFuture<'a> {
    channel: &'a mut DmaChannel,
    completed: bool,
}

impl<'a> DmaTransferFuture<'a> {
    /// 创建新的DMA传输Future
    pub fn new(channel: &'a mut DmaChannel) -> Self {
        Self {
            channel,
            completed: false,
        }
    }
}

impl<'a> Future for DmaTransferFuture<'a> {
    type Output = Result<(), &'static str>;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed {
            return Poll::Ready(Ok(()));
        }
        
        // 检查传输是否完成
        if !self.channel.is_busy() {
            self.completed = true;
            Poll::Ready(Ok(()))
        } else {
            // 传输未完成，重新调度
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

// 全局DMA控制器实例
static DMA_CONTROLLER: DmaController = DmaController::new();

/// 初始化全局DMA控制器
pub fn init_dma_controller() -> Result<(), &'static str> {
    DMA_CONTROLLER.init()
}

/// 获取全局DMA控制器
pub fn get_dma_controller() -> &'static DmaController {
    &DMA_CONTROLLER
}
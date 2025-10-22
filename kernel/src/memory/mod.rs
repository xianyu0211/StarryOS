//! 内存管理模块
//! 
//! 提供物理内存管理、虚拟内存映射、内存分配器等功能

mod allocator;
mod frame;
mod heap;
mod page_table;

use core::alloc::Layout;
use buddy_system_allocator::LockedHeap;

// 全局堆分配器
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

/// 内存初始化
pub fn init() {
    // 初始化堆分配器
    heap::init();
    
    // 初始化页表
    page_table::init();
    
    // 初始化物理内存帧分配器
    frame::init();
}

/// 内存分配
pub fn alloc(layout: Layout) -> Result<*mut u8, &'static str> {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .alloc(layout)
            .map_err(|_| "内存分配失败")
    }
}

/// 内存释放
pub fn dealloc(ptr: *mut u8, layout: Layout) {
    unsafe {
        HEAP_ALLOCATOR.lock().dealloc(ptr, layout);
    }
}

/// 获取内存使用情况
pub fn memory_usage() -> MemoryStats {
    MemoryStats {
        total: 0,
        used: 0,
        free: 0,
    }
}

/// 内存统计信息
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub total: usize,
    pub used: usize,
    pub free: usize,
}

// 导出子模块
pub use allocator::BuddyAllocator;
pub use frame::FrameAllocator;
pub use page_table::PageTable;
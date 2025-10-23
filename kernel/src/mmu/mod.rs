//! 内存管理单元（MMU）模块
//! 
//! 支持RK3588的地址映射与内存保护机制

#![no_std]

use core::arch::asm;
use core::mem::size_of;

/// 页大小（4KB）
pub const PAGE_SIZE: usize = 4096;

/// 内存属性
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryAttribute {
    Normal = 0,
    Device = 1,
    NonCacheable = 2,
}

/// 内存权限
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPermission {
    ReadOnly = 0b01,
    ReadWrite = 0b11,
    ExecuteOnly = 0b10,
    ExecuteRead = 0b11,
}

/// 页表项
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// 创建新的页表项
    pub fn new(
        physical_addr: u64,
        attribute: MemoryAttribute,
        permission: MemoryPermission,
        valid: bool,
    ) -> Self {
        let mut entry = 0u64;
        
        // 设置物理地址（48位地址，低12位为属性）
        entry |= (physical_addr >> 12) << 12;
        
        // 设置属性
        match attribute {
            MemoryAttribute::Normal => {
                entry |= 1 << 2; // Normal memory
                entry |= 1 << 4; // Inner Shareable
            }
            MemoryAttribute::Device => {
                entry |= 0 << 2; // Device memory
                entry |= 1 << 4; // Gather, Reorder, Early Write Ack
            }
            MemoryAttribute::NonCacheable => {
                entry |= 1 << 2; // Normal memory
                entry |= 0 << 4; // Non-cacheable
            }
        }
        
        // 设置权限
        match permission {
            MemoryPermission::ReadOnly => {
                entry |= 0b01 << 6; // AP[2:1] = 01 (Read-only)
            }
            MemoryPermission::ReadWrite => {
                entry |= 0b11 << 6; // AP[2:1] = 11 (Read-write)
            }
            MemoryPermission::ExecuteOnly => {
                entry |= 0b10 << 6; // AP[2:1] = 10 (Execute-only)
            }
            MemoryPermission::ExecuteRead => {
                entry |= 0b11 << 6; // AP[2:1] = 11 (Execute-read)
            }
        }
        
        // 设置有效位
        if valid {
            entry |= 1 << 0; // Valid bit
        }
        
        // 设置访问位
        entry |= 1 << 10; // Access flag
        
        Self(entry)
    }
    
    /// 检查页表项是否有效
    pub fn is_valid(&self) -> bool {
        (self.0 & 1) != 0
    }
    
    /// 获取物理地址
    pub fn physical_address(&self) -> u64 {
        (self.0 & 0x0000_FFFF_FFFF_F000) << 12
    }
    
    /// 获取内存属性
    pub fn memory_attribute(&self) -> MemoryAttribute {
        let attr_bits = (self.0 >> 2) & 0b11;
        match attr_bits {
            0b00 => MemoryAttribute::Device,
            0b01 => MemoryAttribute::Normal,
            _ => MemoryAttribute::Normal,
        }
    }
}

/// 页表级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageTableLevel {
    Level0 = 0, // 512GB blocks
    Level1 = 1, // 1GB blocks
    Level2 = 2, // 2MB blocks
    Level3 = 3, // 4KB pages
}

/// 页表管理器
pub struct PageTableManager {
    root_table: *mut PageTableEntry,
    current_asid: u16,
}

impl PageTableManager {
    /// 创建新的页表管理器
    pub unsafe fn new() -> Self {
        // 分配页表内存（4KB对齐）
        let root_table = Self::allocate_page_table() as *mut PageTableEntry;
        
        Self {
            root_table,
            current_asid: 1,
        }
    }
    
    /// 分配页表内存
    unsafe fn allocate_page_table() -> *mut u8 {
        // 简单的页分配器，实际应该使用内核内存分配器
        static mut PAGE_TABLE_MEMORY: [u8; PAGE_SIZE * 16] = [0; PAGE_SIZE * 16];
        static mut NEXT_PAGE: usize = 0;
        
        let page_addr = &mut PAGE_TABLE_MEMORY[NEXT_PAGE] as *mut u8;
        NEXT_PAGE += PAGE_SIZE;
        
        // 确保4KB对齐
        let aligned_addr = (page_addr as usize & !(PAGE_SIZE - 1)) as *mut u8;
        aligned_addr
    }
    
    /// 映射内存区域
    pub unsafe fn map_region(
        &mut self,
        virtual_addr: u64,
        physical_addr: u64,
        size: usize,
        attribute: MemoryAttribute,
        permission: MemoryPermission,
    ) -> Result<(), &'static str> {
        if virtual_addr % PAGE_SIZE as u64 != 0 || physical_addr % PAGE_SIZE as u64 != 0 {
            return Err("地址未对齐");
        }
        
        let page_count = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        
        for i in 0..page_count {
            let vaddr = virtual_addr + (i * PAGE_SIZE) as u64;
            let paddr = physical_addr + (i * PAGE_SIZE) as u64;
            
            self.map_page(vaddr, paddr, attribute, permission)?;
        }
        
        Ok(())
    }
    
    /// 映射单个页面
    pub unsafe fn map_page(
        &mut self,
        virtual_addr: u64,
        physical_addr: u64,
        attribute: MemoryAttribute,
        permission: MemoryPermission,
    ) -> Result<(), &'static str> {
        let level0_index = (virtual_addr >> 39) & 0x1FF;
        let level1_index = (virtual_addr >> 30) & 0x1FF;
        let level2_index = (virtual_addr >> 21) & 0x1FF;
        let level3_index = (virtual_addr >> 12) & 0x1FF;
        
        // 遍历页表层级
        let mut current_table = self.root_table;
        
        // Level 0
        let l0_entry = &mut *current_table.add(level0_index as usize);
        if !l0_entry.is_valid() {
            // 分配新的L1页表
            let new_table = Self::allocate_page_table() as *mut PageTableEntry;
            *l0_entry = PageTableEntry::new(
                new_table as u64,
                MemoryAttribute::Normal,
                MemoryPermission::ReadWrite,
                true,
            );
        }
        
        // Level 1
        current_table = l0_entry.physical_address() as *mut PageTableEntry;
        let l1_entry = &mut *current_table.add(level1_index as usize);
        if !l1_entry.is_valid() {
            // 分配新的L2页表
            let new_table = Self::allocate_page_table() as *mut PageTableEntry;
            *l1_entry = PageTableEntry::new(
                new_table as u64,
                MemoryAttribute::Normal,
                MemoryPermission::ReadWrite,
                true,
            );
        }
        
        // Level 2
        current_table = l1_entry.physical_address() as *mut PageTableEntry;
        let l2_entry = &mut *current_table.add(level2_index as usize);
        if !l2_entry.is_valid() {
            // 分配新的L3页表
            let new_table = Self::allocate_page_table() as *mut PageTableEntry;
            *l2_entry = PageTableEntry::new(
                new_table as u64,
                MemoryAttribute::Normal,
                MemoryPermission::ReadWrite,
                true,
            );
        }
        
        // Level 3 - 最终页表项
        current_table = l2_entry.physical_address() as *mut PageTableEntry;
        let l3_entry = &mut *current_table.add(level3_index as usize);
        
        *l3_entry = PageTableEntry::new(physical_addr, attribute, permission, true);
        
        Ok(())
    }
    
    /// 取消映射内存区域
    pub unsafe fn unmap_region(&mut self, virtual_addr: u64, size: usize) -> Result<(), &'static str> {
        let page_count = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        
        for i in 0..page_count {
            let vaddr = virtual_addr + (i * PAGE_SIZE) as u64;
            self.unmap_page(vaddr)?;
        }
        
        Ok(())
    }
    
    /// 取消映射单个页面
    pub unsafe fn unmap_page(&mut self, virtual_addr: u64) -> Result<(), &'static str> {
        let level0_index = (virtual_addr >> 39) & 0x1FF;
        let level1_index = (virtual_addr >> 30) & 0x1FF;
        let level2_index = (virtual_addr >> 21) & 0x1FF;
        let level3_index = (virtual_addr >> 12) & 0x1FF;
        
        // 遍历页表层级
        let mut current_table = self.root_table;
        
        // Level 0
        let l0_entry = &mut *current_table.add(level0_index as usize);
        if !l0_entry.is_valid() {
            return Err("页面未映射");
        }
        
        // Level 1
        current_table = l0_entry.physical_address() as *mut PageTableEntry;
        let l1_entry = &mut *current_table.add(level1_index as usize);
        if !l1_entry.is_valid() {
            return Err("页面未映射");
        }
        
        // Level 2
        current_table = l1_entry.physical_address() as *mut PageTableEntry;
        let l2_entry = &mut *current_table.add(level2_index as usize);
        if !l2_entry.is_valid() {
            return Err("页面未映射");
        }
        
        // Level 3 - 最终页表项
        current_table = l2_entry.physical_address() as *mut PageTableEntry;
        let l3_entry = &mut *current_table.add(level3_index as usize);
        
        // 清除页表项
        *l3_entry = PageTableEntry(0);
        
        Ok(())
    }
    
    /// 激活页表
    pub unsafe fn activate(&self) {
        // 设置TTBR0_EL1（用户空间页表）
        asm!(
            "msr ttbr0_el1, {0}",
            "isb",
            in(reg) self.root_table as u64
        );
        
        // 设置TTBR1_EL1（内核空间页表）
        asm!(
            "msr ttbr1_el1, {0}",
            "isb", 
            in(reg) self.root_table as u64
        );
        
        // 设置TCR_EL1（转换控制寄存器）
        let tcr_value: u64 = 0x2B << 16 |  // TBI1=0, TBI0=0
                          0x2B << 0 |   // T1SZ=0x2B, T0SZ=0x2B
                          1 << 8 |      // IRGN1=Normal WB
                          1 << 10 |     // ORGN1=Normal WB
                          1 << 12 |     // SH1=Inner Shareable
                          1 << 14 |     // TG1=4KB
                          1 << 23 |     // HA=1
                          1 << 24 |     // HD=1
                          1 << 25;      // HPD=1
        
        asm!(
            "msr tcr_el1, {0}",
            "isb",
            in(reg) tcr_value
        );
        
        // 设置MAIR_EL1（内存属性索引寄存器）
        let mair_value: u64 = 0xFF << 0 |   // Attr0=Normal memory
                          0x04 << 8 |   // Attr1=Device memory
                          0x44 << 16;   // Attr2=Non-cacheable
        
        asm!(
            "msr mair_el1, {0}",
            "isb",
            in(reg) mair_value
        );
        
        // 刷新TLB
        self.flush_tlb();
    }
    
    /// 刷新TLB
    pub unsafe fn flush_tlb(&self) {
        // 刷新整个TLB
        asm!(
            "tlbi vmalle1",
            "dsb ish",
            "isb"
        );
    }
    
    /// 获取根页表地址
    pub fn root_table_address(&self) -> u64 {
        self.root_table as u64
    }
}

/// 全局页表管理器实例
pub static mut PAGE_TABLE_MANAGER: Option<PageTableManager> = None;

/// 初始化MMU系统
pub unsafe fn init() {
    // 创建页表管理器
    PAGE_TABLE_MANAGER = Some(PageTableManager::new());
    
    if let Some(mmu) = &mut PAGE_TABLE_MANAGER {
        // 映射内核代码段（只读执行）
        mmu.map_region(
            0x80000,        // 内核入口地址
            0x80000,        // 物理地址
            0x100000,       // 1MB内核代码
            MemoryAttribute::Normal,
            MemoryPermission::ExecuteRead,
        ).unwrap();
        
        // 映射内核数据段（读写）
        mmu.map_region(
            0x90000,        // 内核数据地址
            0x90000,        // 物理地址
            0x100000,       // 1MB内核数据
            MemoryAttribute::Normal,
            MemoryPermission::ReadWrite,
        ).unwrap();
        
        // 映射设备内存（UART等）
        mmu.map_region(
            0x0900_0000,    // UART地址
            0x0900_0000,    // 物理地址
            0x1000,         // 4KB
            MemoryAttribute::Device,
            MemoryPermission::ReadWrite,
        ).unwrap();
        
        // 激活页表
        mmu.activate();
        
        // 启用MMU
        enable_mmu();
    }
}

/// 启用MMU
unsafe fn enable_mmu() {
    let sctlr: u64;
    
    // 读取当前SCTLR_EL1
    asm!("mrs {}, sctlr_el1", out(reg) sctlr);
    
    // 设置MMU使能位
    let new_sctlr = sctlr | (1 << 0); // M=1 (MMU enable)
    
    asm!(
        "msr sctlr_el1, {0}",
        "isb",
        in(reg) new_sctlr
    );
}
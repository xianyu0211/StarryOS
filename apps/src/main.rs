//! StarryOS 应用程序入口
//! 
//! 演示RK3588 AIoT系统的完整功能

#![no_std]
#![no_main]

use starry_kernel::{init, println, delay, KernelInfo};
use starry_drivers::{init as init_drivers, AsyncRuntime, DmaBuffer};
use starry_ai::{init as init_ai, AIManager, YoloV8Engine};

/// 应用程序主函数
#[no_mangle]
pub extern "C" fn main() -> ! {
    // 初始化内核
    init();
    
    // 显示内核信息
    let kernel_info = KernelInfo::get();
    kernel_info.display();
    
    println!("=== StarryOS AIoT系统启动 ===");
    
    // 初始化驱动系统
    init_drivers();
    println!("驱动系统初始化完成");
    
    // 初始化AI系统
    init_ai();
    println!("AI系统初始化完成");
    
    // 演示CPU调度功能
    demo_cpu_scheduling();
    
    // 演示内存管理功能
    demo_memory_management();
    
    // 演示AI推理功能
    demo_ai_inference();
    
    // 演示驱动功能
    demo_drivers();
    
    println!("=== StarryOS 演示完成 ===");
    
    // 系统挂起
    loop {
        delay(1000);
        println!("系统运行中...");
    }
}

/// 演示CPU调度功能
fn demo_cpu_scheduling() {
    println!("\n--- CPU调度演示 ---");
    
    use starry_kernel::cpu::{TaskInfo, schedule_task_intelligent, CoreId};
    
    // 创建不同类型的任务
    let compute_task = TaskInfo::new(
        true,   // 计算密集型
        false,  // 非延迟敏感
        100,    // 预估运行时间100ms
        1024,   // 内存使用1MB
        80      // 优先级80
    );
    
    let latency_task = TaskInfo::new(
        false,  // 非计算密集型
        true,   // 延迟敏感
        10,     // 预估运行时间10ms
        512,    // 内存使用512KB
        90      // 优先级90
    );
    
    // 智能任务调度
    if let Some(core_id) = schedule_task_intelligent(&compute_task) {
        println!("计算密集型任务分配到核心: {:?}", core_id);
    }
    
    if let Some(core_id) = schedule_task_intelligent(&latency_task) {
        println!("延迟敏感型任务分配到核心: {:?}", core_id);
    }
}

/// 演示内存管理功能
fn demo_memory_management() {
    println!("\n--- 内存管理演示 ---");
    
    use starry_kernel::memory::dynamic_memory::{get_dynamic_memory_stats, MemoryRegionType};
    use core::alloc::Layout;
    
    // 获取内存统计信息
    let stats = get_dynamic_memory_stats();
    println!("内存使用情况: {}/{} MB", 
        stats.used_memory / 1024 / 1024, 
        stats.total_memory / 1024 / 1024
    );
    println!("内存压力: {}%, 碎片化程度: {}%", 
        stats.memory_pressure, stats.fragmentation_level
    );
    
    // 演示智能内存分配
    let layout = Layout::new::<[u8; 1024]>(); // 分配1KB内存
    
    unsafe {
        if let Ok(ptr) = starry_kernel::memory::dynamic_memory::smart_allocate(
            layout, 
            MemoryRegionType::KernelData
        ) {
            println!("成功分配1KB内存");
            
            // 使用内存
            let slice = core::slice::from_raw_parts_mut(ptr, 1024);
            slice.fill(0xAA);
            
            // 释放内存
            starry_kernel::memory::dynamic_memory::smart_deallocate(ptr, layout);
            println!("内存已释放");
        }
    }
}

/// 演示AI推理功能
fn demo_ai_inference() {
    println!("\n--- AI推理演示 ---");
    
    use starry_ai::{AIManager, YoloV8Engine, InferenceParams, OptimizationLevel, Precision};
    
    // 创建AI管理器
    let mut ai_manager = AIManager::new();
    
    // 注册YOLO-v8引擎
    let yolo_engine = YoloV8Engine::new();
    ai_manager.register_engine(Box::new(yolo_engine));
    
    // 设置推理参数
    let params = InferenceParams {
        batch_size: 1,
        use_hardware_acceleration: true,
        optimization_level: OptimizationLevel::Aggressive,
    };
    
    // 创建测试输入数据
    let input_size = 3 * 640 * 640; // RGB 640x640图像
    let test_input = vec![0.5f32; input_size];
    
    // 执行推理
    if let Ok(result) = ai_manager.infer(&test_input) {
        println!("AI推理完成，输出大小: {}", result.len());
        
        // 解析检测结果
        if let Some(yolo_engine) = ai_manager.engines.get_mut(0) {
            if let Ok(yolo) = yolo_engine.as_any().downcast_ref::<YoloV8Engine>() {
                let detections = yolo.parse_detections(&result);
                println!("检测到 {} 个目标", detections.len());
                
                for detection in detections {
                    println!("目标: 类别={}, 置信度={:.2}%, 位置=({:.1},{:.1},{:.1},{:.1})", 
                        detection.class_id, 
                        detection.confidence * 100.0,
                        detection.bbox[0], detection.bbox[1], 
                        detection.bbox[2], detection.bbox[3]
                    );
                }
            }
        }
    }
}

/// 演示驱动功能
fn demo_drivers() {
    println!("\n--- 驱动系统演示 ---");
    
    use starry_drivers::{AsyncRuntime, DmaBuffer, DmaDirection};
    
    // 初始化异步运行时
    if let Ok(_) = AsyncRuntime::init_async_runtime() {
        println!("异步运行时初始化完成");
        
        // 演示DMA零拷贝传输
        if let Ok(mut dma_buffer) = DmaBuffer::new(4096) { // 4KB缓冲区
            println!("DMA缓冲区创建成功");
            
            // 使用DMA缓冲区
            let slice = dma_buffer.as_mut_slice();
            slice.fill(0x55);
            
            println!("DMA数据传输演示完成");
        }
    }
    
    // 演示中断优先级管理
    use starry_kernel::gic::DYNAMIC_PRIORITY_MANAGER;
    
    // 记录中断统计
    DYNAMIC_PRIORITY_MANAGER.record_interrupt(27, 50); // 定时器中断，延迟50us
    DYNAMIC_PRIORITY_MANAGER.record_interrupt(32, 100); // UART中断，延迟100us
    
    println!("中断优先级管理演示完成");
}

/// 恐慌处理函数
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("应用程序恐慌: {}", info);
    
    if let Some(location) = info.location() {
        println!("在 {}:{}:{}", 
            location.file(), 
            location.line(), 
            location.column()
        );
    }
    
    loop {
        starry_kernel::halt();
    }
}
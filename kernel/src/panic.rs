//! 内核panic处理

use core::panic::PanicInfo;
use cortex_a::asm;
use log::error;

/// 内核panic处理函数
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("内核panic: {}", info);
    
    // 打印panic位置信息
    if let Some(location) = info.location() {
        error!("位于 {}:{}:{}", 
            location.file(), 
            location.line(), 
            location.column()
        );
    }
    
    // 打印panic消息
    if let Some(message) = info.message() {
        error!("消息: {}", message);
    }
    
    // 进入无限循环，等待调试
    loop {
        asm::wfe(); // 等待事件，降低功耗
    }
}

/// 断言失败处理
#[cfg(debug_assertions)]
#[inline(never)]
#[no_mangle]
pub extern "C" fn __assert_fail(
    expr: &'static str,
    file: &'static str,
    line: u32,
    func: &'static str,
) -> ! {
    error!("断言失败: {} at {}:{} in {}", expr, file, line, func);
    panic!("断言失败");
}
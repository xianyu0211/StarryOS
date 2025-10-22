//! 构建脚本 - 用于内核构建配置

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // 设置构建目标
    let target = env::var("TARGET").unwrap();
    
    // 检查是否为aarch64目标
    if target.contains("aarch64") {
        println!("cargo:rustc-cfg=target_arch=\"aarch64\"");
        
        // 设置链接器脚本
        let linker_script = "kernel/src/arch/aarch64/link.ld";
        if Path::new(linker_script).exists() {
            println!("cargo:rustc-link-arg=-T{}", linker_script);
        }
        
        // 设置特定于架构的标志
        println!("cargo:rustc-link-arg=--nmagic");
        println!("cargo:rustc-link-arg=--gc-sections");
    }
    
    // 设置优化级别
    if env::var("PROFILE").unwrap() == "release" {
        println!("cargo:rustc-cfg=release");
    }
    
    // 生成版本信息
    generate_version_info();
}

/// 生成版本信息文件
fn generate_version_info() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let version_file = Path::new(&out_dir).join("version.rs");
    
    let version_info = format!(
        r#"
// 自动生成的版本信息
pub const KERNEL_VERSION: &str = "StarryOS v0.1.0";
pub const BUILD_TIMESTAMP: &str = "{}";
pub const BUILD_TARGET: &str = "{}";
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        env::var("TARGET").unwrap()
    );
    
    fs::write(version_file, version_info).unwrap();
}
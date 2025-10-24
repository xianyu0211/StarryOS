//! StarryOS - 综合测试套件
//! 
//! 提供完整的系统测试、性能基准测试和集成验证

#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, feature(test))]

// 测试模块
pub mod unit;
pub mod integration;
pub mod benchmarks;
pub mod hardware;
pub mod security;

// 测试工具模块
mod utils;

use core::fmt;

/// 测试错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestError {
    TestFailed,
    Timeout,
    HardwareError,
    ResourceUnavailable,
    InvalidConfiguration,
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestError::TestFailed => write!(f, "测试失败"),
            TestError::Timeout => write!(f, "测试超时"),
            TestError::HardwareError => write!(f, "硬件错误"),
            TestError::ResourceUnavailable => write!(f, "资源不可用"),
            TestError::InvalidConfiguration => write!(f, "配置无效"),
        }
    }
}

/// 测试结果
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: &'static str,
    pub passed: bool,
    pub duration_ms: u64,
    pub error: Option<TestError>,
    pub details: Option<&'static str>,
}

impl TestResult {
    /// 创建成功的测试结果
    pub fn success(name: &'static str, duration_ms: u64) -> Self {
        Self {
            name,
            passed: true,
            duration_ms,
            error: None,
            details: None,
        }
    }
    
    /// 创建失败的测试结果
    pub fn failure(name: &'static str, error: TestError, details: Option<&'static str>) -> Self {
        Self {
            name,
            passed: false,
            duration_ms: 0,
            error: Some(error),
            details,
        }
    }
}

/// 测试套件特征
pub trait TestSuite {
    /// 测试套件名称
    fn name(&self) -> &'static str;
    
    /// 运行测试套件
    fn run(&self) -> Vec<TestResult>;
    
    /// 测试套件描述
    fn description(&self) -> &'static str {
        ""
    }
}

/// 测试运行器
pub struct TestRunner {
    suites: Vec<Box<dyn TestSuite>>,
}

impl TestRunner {
    /// 创建新的测试运行器
    pub fn new() -> Self {
        Self {
            suites: Vec::new(),
        }
    }
    
    /// 注册测试套件
    pub fn register_suite<T: TestSuite + 'static>(&mut self, suite: T) {
        self.suites.push(Box::new(suite));
    }
    
    /// 运行所有测试套件
    pub fn run_all(&self) -> Vec<TestResult> {
        let mut all_results = Vec::new();
        
        for suite in &self.suites {
            let results = suite.run();
            all_results.extend(results);
        }
        
        all_results
    }
    
    /// 生成测试报告
    pub fn generate_report(&self, results: &[TestResult]) -> TestReport {
        let total_tests = results.len();
        let passed_tests = results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;
        
        let total_duration = results.iter().map(|r| r.duration_ms).sum();
        
        TestReport {
            total_tests,
            passed_tests,
            failed_tests,
            total_duration_ms: total_duration,
            results: results.to_vec(),
        }
    }
}

/// 测试报告
#[derive(Debug, Clone)]
pub struct TestReport {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub total_duration_ms: u64,
    pub results: Vec<TestResult>,
}

impl TestReport {
    /// 获取测试成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            return 0.0;
        }
        self.passed_tests as f64 / self.total_tests as f64 * 100.0
    }
    
    /// 显示测试报告
    pub fn display(&self) {
        println!("=== StarryOS 测试报告 ===");
        println!("总测试数: {}", self.total_tests);
        println!("通过数: {}", self.passed_tests);
        println!("失败数: {}", self.failed_tests);
        println!("成功率: {:.2}%", self.success_rate());
        println!("总耗时: {} ms", self.total_duration_ms);
        
        if self.failed_tests > 0 {
            println!("\n失败的测试:");
            for result in &self.results {
                if !result.passed {
                    println!("  - {}: {:?}", result.name, result.error);
                    if let Some(details) = result.details {
                        println!("    详情: {}", details);
                    }
                }
            }
        }
    }
}

/// 全局测试运行器实例
pub static mut TEST_RUNNER: Option<TestRunner> = None;

/// 初始化测试系统
pub fn init() {
    unsafe {
        TEST_RUNNER = Some(TestRunner::new());
    }
}

/// 运行所有测试
pub fn run_all_tests() -> TestReport {
    unsafe {
        if let Some(runner) = &TEST_RUNNER {
            let results = runner.run_all();
            runner.generate_report(&results)
        } else {
            panic!("测试运行器未初始化");
        }
    }
}

/// 测试宏
#[macro_export]
macro_rules! test_case {
    ($name:expr, $body:block) => {
        {
            use core::time::Duration;
            use $crate::{TestResult, TestError};
            
            let start = std::time::Instant::now();
            let result = std::panic::catch_unwind(|| $body);
            let duration = start.elapsed().as_millis() as u64;
            
            match result {
                Ok(()) => TestResult::success($name, duration),
                Err(_) => TestResult::failure($name, TestError::TestFailed, None),
            }
        }
    };
}

/// 断言宏
#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {
        if $left != $right {
            panic!("断言失败: {} != {}", $left, $right);
        }
    };
}

#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr) => {
        if $left == $right {
            panic!("断言失败: {} == {}", $left, $right);
        }
    };
}

#[macro_export]
macro_rules! assert_true {
    ($expr:expr) => {
        if !$expr {
            panic!("断言失败: 表达式为假");
        }
    };
}

#[macro_export]
macro_rules! assert_false {
    ($expr:expr) => {
        if $expr {
            panic!("断言失败: 表达式为真");
        }
    };
}
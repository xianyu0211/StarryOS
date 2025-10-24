//! 通用数据结构模块
//! 
//! 提供所有模块共享的基础数据结构

use core::fmt;

/// 边界框
/// 
/// 用于表示二维空间中的矩形区域，广泛应用于目标检测等场景
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct BoundingBox {
    pub x: f32,      // 中心点x坐标
    pub y: f32,      // 中心点y坐标
    pub width: f32,  // 宽度
    pub height: f32, // 高度
    pub area: f32,   // 预计算面积，避免重复计算
}

impl BoundingBox {
    /// 创建新的边界框（性能优化版本）
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            area: width * height, // 预计算面积
        }
    }
    
    /// 创建边界框并验证有效性
    pub fn new_checked(x: f32, y: f32, width: f32, height: f32) -> Option<Self> {
        if width > 0.0 && height > 0.0 {
            Some(Self::new(x, y, width, height))
        } else {
            None
        }
    }
    
    /// 获取边界框的面积（使用预计算值）
    pub fn area(&self) -> f32 {
        self.area
    }
    
    /// 计算两个边界框的交并比(IoU) - 优化版本
    pub fn calculate_iou(&self, other: &BoundingBox) -> f32 {
        // 快速检查：如果边界框不相交，直接返回0
        if self.x.abs_diff(other.x) > (self.width + other.width) / 2.0 ||
           self.y.abs_diff(other.y) > (self.height + other.height) / 2.0 {
            return 0.0;
        }
        
        let left = self.x - self.width / 2.0;
        let right = self.x + self.width / 2.0;
        let top = self.y - self.height / 2.0;
        let bottom = self.y + self.height / 2.0;
        
        let other_left = other.x - other.width / 2.0;
        let other_right = other.x + other.width / 2.0;
        let other_top = other.y - other.height / 2.0;
        let other_bottom = other.y + other.height / 2.0;
        
        // 计算交集
        let intersection_left = left.max(other_left);
        let intersection_right = right.min(other_right);
        let intersection_top = top.max(other_top);
        let intersection_bottom = bottom.min(other_bottom);
        
        if intersection_left >= intersection_right || intersection_top >= intersection_bottom {
            return 0.0;
        }
        
        let intersection_area = (intersection_right - intersection_left) * 
                               (intersection_bottom - intersection_top);
        let union_area = self.area() + other.area() - intersection_area;
        
        intersection_area / union_area
    }
    
    /// 检查边界框是否有效（宽度和高度为正数）
    pub fn is_valid(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

impl fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BoundingBox(x={}, y={}, width={}, height={})", 
               self.x, self.y, self.width, self.height)
    }
}

/// 检测结果
/// 
/// 用于表示目标检测的结果
#[derive(Debug, Clone)]
pub struct Detection {
    pub class_id: u32,
    pub class_name: &'static str,
    pub confidence: f32,
    pub bbox: BoundingBox,
}

impl Detection {
    /// 创建新的检测结果
    pub fn new(class_id: u32, class_name: &'static str, confidence: f32, bbox: BoundingBox) -> Self {
        Self {
            class_id,
            class_name,
            confidence,
            bbox,
        }
    }
    
    /// 检查检测结果是否有效
    pub fn is_valid(&self) -> bool {
        self.confidence >= 0.0 && self.confidence <= 1.0 && self.bbox.is_valid()
    }
}

/// 传感器数据
/// 
/// 用于表示环境传感器采集的数据
#[derive(Debug, Clone, Copy)]
pub struct SensorData {
    pub temperature: Option<f32>,  // 温度 (°C)
    pub humidity: Option<f32>,     // 湿度 (%)
    pub light_level: Option<f32>,  // 光照强度 (lux)
    pub acceleration: Option<(f32, f32, f32)>, // 加速度 (x, y, z)
    pub gyroscope: Option<(f32, f32, f32)>,    // 陀螺仪 (x, y, z)
}

impl SensorData {
    /// 创建新的传感器数据
    pub const fn new() -> Self {
        Self {
            temperature: None,
            humidity: None,
            light_level: None,
            acceleration: None,
            gyroscope: None,
        }
    }
}

/// 系统配置
/// 
/// 用于表示系统级别的配置参数
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceMode {
    PowerSaving,  // 省电模式
    Balanced,     // 平衡模式
    Performance,  // 性能模式
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,  // 错误级别
    Warn,   // 警告级别
    Info,   // 信息级别
    Debug,  // 调试级别
    Trace,  // 跟踪级别
}

/// 任务信息
/// 
/// 用于任务调度和资源管理
#[derive(Debug, Clone, Copy)]
pub struct TaskInfo {
    pub is_compute_intensive: bool,    // 是否为计算密集型任务
    pub is_latency_sensitive: bool,    // 是否为延迟敏感型任务
    pub estimated_runtime: u64,        // 预估运行时间(ms)
    pub memory_usage: u32,            // 内存使用量(KB)
    pub priority: u8,                  // 任务优先级(0-100)
}

impl TaskInfo {
    /// 创建新的任务信息
    pub const fn new(
        is_compute_intensive: bool,
        is_latency_sensitive: bool,
        estimated_runtime: u64,
        memory_usage: u32,
        priority: u8
    ) -> Self {
        Self {
            is_compute_intensive,
            is_latency_sensitive,
            estimated_runtime,
            memory_usage,
            priority,
        }
    }
}
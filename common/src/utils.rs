//! 通用工具函数模块
//! 
//! 提供内存管理、数据处理和优化算法等辅助功能

#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use core::mem::size_of;

/// 内存对齐函数
/// 
/// 将给定的值对齐到指定的边界
pub fn align_up(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}

/// 计算数组或切片的元素数量
pub fn count_elements<T>(data: &[u8]) -> usize {
    data.len() / size_of::<T>()
}

/// 安全地将字节切片转换为类型引用
pub unsafe fn bytes_to_ref<T>(data: &[u8]) -> Option<&T> {
    if data.len() >= size_of::<T>() {
        Some(&*(data.as_ptr() as *const T))
    } else {
        None
    }
}

/// 安全地将字节切片转换为可变类型引用
pub unsafe fn bytes_to_mut_ref<T>(data: &mut [u8]) -> Option<&mut T> {
    if data.len() >= size_of::<T>() {
        Some(&mut *(data.as_mut_ptr() as *mut T))
    } else {
        None
    }
}

/// 计算向量的平均值（优化版本）
pub fn calculate_mean(data: &[f32]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    
    // 使用迭代器避免边界检查
    let sum: f32 = data.iter().copied().sum();
    sum / data.len() as f32
}

/// 计算向量的标准差（优化版本）
pub fn calculate_stddev(data: &[f32]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    
    let mean = calculate_mean(data);
    let variance: f32 = data.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f32>() / data.len() as f32;
    
    variance.sqrt()
}

/// 快速排序算法（优化版本）
pub fn quick_sort<T: PartialOrd + Clone>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }
    
    let pivot_index = partition(arr);
    quick_sort(&mut arr[0..pivot_index]);
    quick_sort(&mut arr[pivot_index + 1..]);
}

fn partition<T: PartialOrd + Clone>(arr: &mut [T]) -> usize {
    let pivot_index = arr.len() - 1;
    let mut i = 0;
    
    for j in 0..pivot_index {
        if arr[j] <= arr[pivot_index] {
            arr.swap(i, j);
            i += 1;
        }
    }
    
    arr.swap(i, pivot_index);
    i
}

/// 非极大值抑制算法（优化版本）
pub fn non_max_suppression(boxes: &[BoundingBox], scores: &[f32], iou_threshold: f32) -> Vec<usize> {
    if boxes.is_empty() {
        return Vec::new();
    }
    
    // 预分配结果向量
    let mut result = Vec::with_capacity(boxes.len());
    
    // 创建索引并排序
    let mut indices: Vec<usize> = (0..boxes.len()).collect();
    indices.sort_by(|&a, &b| scores[b].partial_cmp(&scores[a]).unwrap());
    
    while !indices.is_empty() {
        let current = indices.remove(0);
        result.push(current);
        
        // 使用迭代器过滤
        indices.retain(|&i| boxes[current].calculate_iou(&boxes[i]) <= iou_threshold);
    }
    
    result
}

/// 向量归一化（优化版本）
pub fn normalize_vector(vec: &mut [f32]) {
    let magnitude = vec.iter().map(|&x| x * x).sum::<f32>().sqrt();
    
    if magnitude > 0.0 {
        for x in vec.iter_mut() {
            *x /= magnitude;
        }
    }
}

/// 向量点积（优化版本）
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum()
}

/// 计算向量的标准差
pub fn calculate_stddev(data: &[f32]) -> f32 {
    if data.len() <= 1 {
        return 0.0;
    }
    
    let mean = calculate_mean(data);
    let variance: f32 = data.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f32>() / (data.len() as f32 - 1.0);
    
    variance.sqrt()
}

/// 快速排序算法实现
pub fn quicksort<T: Ord>(data: &mut [T]) {
    if data.len() <= 1 {
        return;
    }
    
    let pivot_index = partition(data);
    quicksort(&mut data[0..pivot_index]);
    quicksort(&mut data[pivot_index + 1..]);
}

/// 分区函数（用于快速排序）
fn partition<T: Ord>(data: &mut [T]) -> usize {
    let len = data.len();
    let pivot_index = len / 2;
    
    // 将枢轴元素移到末尾
    data.swap(pivot_index, len - 1);
    
    let mut store_index = 0;
    for i in 0..len - 1 {
        if data[i] <= data[len - 1] {
            data.swap(i, store_index);
            store_index += 1;
        }
    }
    
    // 将枢轴元素移回正确位置
    data.swap(store_index, len - 1);
    store_index
}

/// 非最大值抑制（NMS）
/// 
/// 用于目标检测结果的后处理，去除重叠的边界框
pub fn non_maximum_suppression<T, F>(
    detections: &mut [T], 
    get_score: F, 
    iou_threshold: f32
) where 
    F: Fn(&T) -> (&BoundingBox, f32),
{
    // 按置信度降序排序
    detections.sort_by(|a, b| {
        let (_, score_a) = get_score(a);
        let (_, score_b) = get_score(b);
        score_b.partial_cmp(&score_a).unwrap_or(core::cmp::Ordering::Equal)
    });
    
    let mut keep_indices = Vec::new();
    let mut suppressed = vec![false; detections.len()];
    
    for i in 0..detections.len() {
        if suppressed[i] {
            continue;
        }
        
        keep_indices.push(i);
        let (current_bbox, _) = get_score(&detections[i]);
        
        for j in i + 1..detections.len() {
            if suppressed[j] {
                continue;
            }
            
            let (other_bbox, _) = get_score(&detections[j]);
            let iou = current_bbox.calculate_iou(other_bbox);
            
            if iou > iou_threshold {
                suppressed[j] = true;
            }
        }
    }
    
    // 重新排列保留的检测结果
    let mut result = Vec::with_capacity(keep_indices.len());
    for &idx in &keep_indices {
        result.push(core::mem::replace(&mut detections[idx], core::mem::MaybeUninit::uninit().assume_init()));
    }
    
    detections.clear();
    detections.extend(result);
}

// 导入BoundingBox用于NMS函数
use super::data_structures::BoundingBox;

/// 向量归一化
pub fn normalize_vector(data: &mut [f32]) {
    let norm: f32 = data.iter().map(|&x| x.powi(2)).sum::<f32>().sqrt();
    
    if norm > 0.0 {
        for value in data {
            *value /= norm;
        }
    }
}

/// 计算两个向量的点积
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// 安全的整数除法，避免除零错误
pub fn safe_divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

/// 限制值在指定范围内
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// 线性插值
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * clamp(t, 0.0, 1.0)
}

/// 计算运行平均值
pub struct RunningAverage {
    sum: f32,
    count: usize,
}

impl RunningAverage {
    /// 创建新的运行平均值计算器
    pub fn new() -> Self {
        Self {
            sum: 0.0,
            count: 0,
        }
    }
    
    /// 添加一个新的值
    pub fn add(&mut self, value: f32) {
        self.sum += value;
        self.count += 1;
    }
    
    /// 获取当前平均值
    pub fn get(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f32
        }
    }
    
    /// 重置计算器
    pub fn reset(&mut self) {
        self.sum = 0.0;
        self.count = 0;
    }
}
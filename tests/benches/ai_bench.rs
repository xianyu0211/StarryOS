//! AI模块性能基准测试
//!
//! 提供全面的AI推理性能评估，包括延迟、吞吐量、内存使用等指标

#![no_std]
#![no_main]

use criterion::{black_box, Criterion, Throughput};
use starry_ai::{
    AIManager, InferenceParams, Precision, OptimizationLevel, 
    ModelInfo, AIError, InferenceEngine
};
use starry_ai::yolo_v8::YoloV8Engine;
use starry_ai::speech_recognition::SpeechRecognitionModel;
use starry_ai::text_to_speech::TextToSpeechModel;
use starry_ai::npu::{NPUDevice, create_npu_driver, detect_available_npus};
use alloc::vec::Vec;
use alloc::string::String;
use core::time::Duration;

/// 基准测试配置
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub warmup_iterations: usize,
    pub measurement_iterations: usize,
    pub sample_size: usize,
    pub enable_memory_tracking: bool,
    pub enable_power_measurement: bool,
    pub output_format: OutputFormat,
}

/// 输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    HumanReadable,
    JSON,
    CSV,
}

/// 基准测试结果
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub latency_stats: LatencyStats,
    pub throughput: f32,
    pub memory_usage: MemoryUsage,
    pub power_consumption: Option<f32>,
    pub hardware_utilization: Option<f32>,
}

/// 延迟统计
#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub min: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

/// 内存使用情况
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub peak_rss: usize,
    pub model_memory: usize,
    pub inference_memory: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 10,
            measurement_iterations: 100,
            sample_size: 100,
            enable_memory_tracking: true,
            enable_power_measurement: false,
            output_format: OutputFormat::HumanReadable,
        }
    }
}

/// Yolo-v8推理性能测试
fn yolo_v8_inference_benchmark(c: &mut Criterion) {
    let mut engine = YoloV8Engine::new();
    
    // 模拟模型加载
    let model_data = vec![0u8; 5 * 1024 * 1024]; // 5MB模型
    engine.load_model(&model_data).unwrap();
    
    // 设置推理参数
    let params = InferenceParams {
        batch_size: 1,
        use_hardware_acceleration: true,
        optimization_level: OptimizationLevel::High,
        precision: Precision::FP16,
    };
    engine.set_params(params).unwrap();
    
    // 创建测试输入数据
    let input_size = 3 * 640 * 640; // RGB 640x640图像
    let test_input = vec![0.5f32; input_size];
    
    let mut group = c.benchmark_group("yolo_v8");
    group.throughput(Throughput::Elements(1));
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("inference", |b| {
        b.iter(|| {
            // 执行推理
            let result = engine.infer(black_box(&test_input));
            black_box(result)
        })
    });
    
    // 批处理性能测试
    group.bench_function("batch_4", |b| {
        let batch_input = vec![test_input.clone(); 4];
        b.iter(|| {
            let result = engine.infer_batch(black_box(&batch_input));
            black_box(result)
        })
    });
    
    group.finish();
}

/// 不同精度下的性能比较
fn precision_comparison_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("precision_comparison");
    
    for precision in &[Precision::FP32, Precision::FP16, Precision::INT8] {
        let mut engine = YoloV8Engine::new();
        let model_data = vec![0u8; 5 * 1024 * 1024];
        engine.load_model(&model_data).unwrap();
        
        let params = InferenceParams {
            batch_size: 1,
            use_hardware_acceleration: true,
            optimization_level: OptimizationLevel::High,
            precision: *precision,
        };
        engine.set_params(params).unwrap();
        
        let input_size = 3 * 640 * 640;
        let test_input = vec![0.5f32; input_size];
        
        group.bench_function(&format!("{:?}", precision), |b| {
            b.iter(|| {
                let result = engine.infer(black_box(&test_input));
                black_box(result)
            })
        });
    }
    
    group.finish();
}

/// AI管理器性能测试
fn ai_manager_benchmark(c: &mut Criterion) {
    let mut ai_manager = AIManager::new();
    
    // 注册多个引擎
    let yolo_engine = YoloV8Engine::new();
    let speech_engine = SpeechRecognitionModel::new(starry_ai::speech_recognition::Language::Chinese);
    let tts_engine = TextToSpeechModel::new(starry_ai::text_to_speech::VoiceType::Female);
    
    ai_manager.register_engine("yolo", Box::new(yolo_engine));
    ai_manager.register_engine("speech", Box::new(speech_engine));
    ai_manager.register_engine("tts", Box::new(tts_engine));
    
    // 创建测试输入数据
    let input_size = 3 * 640 * 640;
    let test_input = vec![0.5f32; input_size];
    
    let mut group = c.benchmark_group("ai_manager");
    
    group.bench_function("single_inference", |b| {
        ai_manager.set_current_engine("yolo").unwrap();
        b.iter(|| {
            let result = ai_manager.infer(black_box(&test_input));
            black_box(result)
        })
    });
    
    group.bench_function("engine_switching", |b| {
        b.iter(|| {
            // 测试引擎切换性能
            ai_manager.set_current_engine("yolo").unwrap();
            let _ = ai_manager.infer(black_box(&test_input));
            
            ai_manager.set_current_engine("speech").unwrap();
            let audio_input = vec![0i16; 16000]; // 1秒音频
            let _ = ai_manager.infer(&audio_input);
            
            black_box(())
        })
    });
    
    group.finish();
}

/// 模型加载性能测试
fn model_loading_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("model_loading");
    
    // 测试不同模型大小的加载时间
    let model_sizes = [1 * 1024 * 1024, 5 * 1024 * 1024, 10 * 1024 * 1024]; // 1MB, 5MB, 10MB
    
    for &size in &model_sizes {
        group.bench_function(&format!("{}MB", size / 1024 / 1024), |b| {
            b.iter(|| {
                let mut engine = YoloV8Engine::new();
                let model_data = vec![0u8; size];
                
                let result = engine.load_model(black_box(&model_data));
                black_box(result)
            })
        });
    }
    
    group.finish();
}

/// NPU加速性能测试
fn npu_acceleration_benchmark(c: &mut Criterion) {
    let available_npus = detect_available_npus();
    
    if available_npus.is_empty() {
        log::warn!("未检测到可用的NPU设备，跳过NPU基准测试");
        return;
    }
    
    let mut group = c.benchmark_group("npu_acceleration");
    
    for &npu_device in &available_npus {
        if let Ok(mut npu_driver) = create_npu_driver(npu_device) {
            let model_data = vec![0u8; 5 * 1024 * 1024];
            npu_driver.load_model(&model_data).unwrap();
            
            let input_size = 3 * 640 * 640;
            let test_input = vec![0.5f32; input_size];
            
            group.bench_function(&format!("{:?}", npu_device), |b| {
                b.iter(|| {
                    let result = npu_driver.infer(black_box(&test_input));
                    black_box(result)
                })
            });
        }
    }
    
    // CPU基准线
    let mut cpu_engine = YoloV8Engine::new();
    let model_data = vec![0u8; 5 * 1024 * 1024];
    cpu_engine.load_model(&model_data).unwrap();
    
    let cpu_params = InferenceParams {
        batch_size: 1,
        use_hardware_acceleration: false,
        optimization_level: OptimizationLevel::Basic,
        precision: Precision::FP32,
    };
    cpu_engine.set_params(cpu_params).unwrap();
    
    let input_size = 3 * 640 * 640;
    let test_input = vec![0.5f32; input_size];
    
    group.bench_function("CPU", |b| {
        b.iter(|| {
            let result = cpu_engine.infer(black_box(&test_input));
            black_box(result)
        })
    });
    
    group.finish();
}

/// 内存使用基准测试
fn memory_usage_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    group.bench_function("yolo_v8_memory", |b| {
        b.iter(|| {
            // 测量模型加载和推理的内存使用
            let mut engine = YoloV8Engine::new();
            let model_data = vec![0u8; 5 * 1024 * 1024];
            engine.load_model(&model_data).unwrap();
            
            let input_size = 3 * 640 * 640;
            let test_input = vec![0.5f32; input_size];
            
            let _result = engine.infer(&test_input);
            
            // 获取内存使用统计
            if let Some(info) = engine.get_model_info() {
                black_box(info);
            }
        })
    });
    
    group.finish();
}

/// 语音识别性能测试
fn speech_recognition_benchmark(c: &mut Criterion) {
    let mut engine = SpeechRecognitionModel::new(starry_ai::speech_recognition::Language::Chinese);
    
    // 模拟模型加载
    let model_data = vec![0u8; 2 * 1024 * 1024]; // 2MB语音模型
    engine.load_model(&model_data).unwrap();
    
    // 创建测试音频数据 (16kHz, 16bit, 3秒)
    let audio_duration_seconds = 3;
    let sample_rate = 16000;
    let audio_data = vec![0i16; sample_rate * audio_duration_seconds];
    
    let mut group = c.benchmark_group("speech_recognition");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("recognition", |b| {
        b.iter(|| {
            let result = engine.recognize(black_box(&audio_data));
            black_box(result)
        })
    });
    
    // 流式识别测试
    group.bench_function("streaming", |b| {
        let chunk_size = sample_rate / 10; // 100ms chunks
        b.iter(|| {
            for chunk in audio_data.chunks(chunk_size) {
                let result = engine.recognize_stream(black_box(chunk));
                black_box(result);
            }
        })
    });
    
    group.finish();
}

/// 文本转语音性能测试
fn text_to_speech_benchmark(c: &mut Criterion) {
    let mut engine = TextToSpeechModel::new(starry_ai::text_to_speech::VoiceType::Female);
    
    let model_data = vec![0u8; 3 * 1024 * 1024]; // 3MB TTS模型
    engine.load_model(&model_data).unwrap();
    
    let test_text = "欢迎使用星火AI系统，这是一个性能基准测试。";
    let config = starry_ai::text_to_speech::TTSConfig::default();
    
    let mut group = c.benchmark_group("text_to_speech");
    
    group.bench_function("synthesis", |b| {
        b.iter(|| {
            let result = engine.synthesize(black_box(test_text), black_box(config.clone()));
            black_box(result)
        })
    });
    
    // 不同文本长度测试
    let text_lengths = [10, 50, 100]; // 字符数
    for &length in &text_lengths {
        let test_text_long = "测".repeat(length);
        
        group.bench_function(&format!("text_length_{}", length), |b| {
            b.iter(|| {
                let result = engine.synthesize(black_box(&test_text_long), black_box(config.clone()));
                black_box(result)
            })
        });
    }
    
    group.finish();
}

/// 端到端AI流水线性能测试
fn end_to_end_pipeline_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end_pipeline");
    
    group.bench_function("vision_to_speech", |b| {
        // 模拟视觉识别 + 语音播报的完整流程
        b.iter(|| {
            // 1. 视觉识别
            let mut yolo_engine = YoloV8Engine::new();
            let yolo_model = vec![0u8; 5 * 1024 * 1024];
            yolo_engine.load_model(&yolo_model).unwrap();
            
            let image_input = vec![0.5f32; 3 * 640 * 640];
            let detection_result = yolo_engine.infer(&image_input).unwrap();
            
            // 2. 生成语音描述
            let mut tts_engine = TextToSpeechModel::new(starry_ai::text_to_speech::VoiceType::Female);
            let tts_model = vec![0u8; 3 * 1024 * 1024];
            tts_engine.load_model(&tts_model).unwrap();
            
            let description = "检测到多个物体";
            let audio_result = tts_engine.synthesize(description, Default::default()).unwrap();
            
            black_box((detection_result, audio_result))
        })
    });
    
    group.finish();
}

/// 压力测试 - 长时间运行稳定性
fn stress_test_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_test");
    group.sample_size(500); // 更多样本以测试稳定性
    group.measurement_time(Duration::from_secs(30));
    
    group.bench_function("long_running", |b| {
        let mut engine = YoloV8Engine::new();
        let model_data = vec![0u8; 5 * 1024 * 1024];
        engine.load_model(&model_data).unwrap();
        
        let input_size = 3 * 640 * 640;
        let test_input = vec![0.5f32; input_size];
        
        b.iter(|| {
            for _ in 0..10 {
                let result = engine.infer(black_box(&test_input));
                black_box(result);
            }
        })
    });
    
    group.finish();
}

/// 自定义基准测试运行器
pub fn run_comprehensive_benchmarks(config: BenchmarkConfig) -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    
    // 这里可以实现自定义的基准测试逻辑
    // 包括内存跟踪、功耗测量等高级功能
    
    results
}

/// 性能回归测试
pub fn performance_regression_test(baseline: &BenchmarkResult, current: &BenchmarkResult) -> bool {
    // 检查性能是否退化
    let latency_increase = current.latency_stats.mean.as_secs_f64() / baseline.latency_stats.mean.as_secs_f64();
    let memory_increase = current.memory_usage.peak_rss as f64 / baseline.memory_usage.peak_rss as f64;
    
    // 如果延迟增加超过10%或内存使用增加超过20%，则认为性能退化
    latency_increase > 1.10 || memory_increase > 1.20
}

criterion_group!(
    benches,
    yolo_v8_inference_benchmark,
    precision_comparison_benchmark,
    ai_manager_benchmark,
    model_loading_benchmark,
    npu_acceleration_benchmark,
    memory_usage_benchmark,
    speech_recognition_benchmark,
    text_to_speech_benchmark,
    end_to_end_pipeline_benchmark,
    stress_test_benchmark
);

criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_config() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.warmup_iterations, 10);
        assert_eq!(config.measurement_iterations, 100);
    }
    
    #[test]
    fn test_performance_regression() {
        let baseline = BenchmarkResult {
            test_name: "test".to_string(),
            latency_stats: LatencyStats {
                min: Duration::from_millis(10),
                max: Duration::from_millis(20),
                mean: Duration::from_millis(15),
                p50: Duration::from_millis(15),
                p95: Duration::from_millis(18),
                p99: Duration::from_millis(19),
            },
            throughput: 66.67,
            memory_usage: MemoryUsage {
                peak_rss: 100 * 1024 * 1024,
                model_memory: 50 * 1024 * 1024,
                inference_memory: 10 * 1024 * 1024,
            },
            power_consumption: Some(2.5),
            hardware_utilization: Some(80.0),
        };
        
        let good_result = baseline.clone();
        let bad_result = BenchmarkResult {
            latency_stats: LatencyStats {
                mean: Duration::from_millis(20), // 延迟增加
                ..baseline.latency_stats
            },
            ..baseline
        };
        
        assert!(!performance_regression_test(&baseline, &good_result));
        assert!(performance_regression_test(&baseline, &bad_result));
    }
}
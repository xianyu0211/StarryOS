//! AI模块性能基准测试

#![no_std]
#![no_main]

use criterion::{black_box, Criterion};
use starry_ai::{AIManager, InferenceParams, Precision, OptimizationLevel};
use starry_ai::yolo_v8::YoloV8Engine;

/// Yolo-v8推理性能测试
fn yolo_v8_inference_benchmark(c: &mut Criterion) {
    let mut engine = YoloV8Engine::new();
    
    // 模拟模型加载
    let model_data = vec![0u8; 1024];
    engine.load_model(&model_data).unwrap();
    
    // 设置推理参数
    let params = InferenceParams {
        batch_size: 1,
        use_hardware_acceleration: false,
        optimization_level: OptimizationLevel::Basic,
    };
    engine.set_params(params).unwrap();
    
    // 创建测试输入数据
    let input_size = 3 * 640 * 640; // RGB 640x640图像
    let test_input = vec![0.5f32; input_size];
    
    c.bench_function("yolo_v8_inference", |b| {
        b.iter(|| {
            // 执行推理
            let result = engine.infer(black_box(&test_input));
            black_box(result);
        })
    });
}

/// AI管理器性能测试
fn ai_manager_benchmark(c: &mut Criterion) {
    let mut ai_manager = AIManager::new();
    
    // 注册Yolo-v8引擎
    let yolo_engine = YoloV8Engine::new();
    ai_manager.register_engine(Box::new(yolo_engine));
    ai_manager.set_current_engine(0).unwrap();
    
    // 创建测试输入数据
    let input_size = 3 * 640 * 640;
    let test_input = vec![0.5f32; input_size];
    
    c.bench_function("ai_manager_inference", |b| {
        b.iter(|| {
            // 通过AI管理器执行推理
            let result = ai_manager.infer(black_box(&test_input));
            black_box(result);
        })
    });
}

/// 模型加载性能测试
fn model_loading_benchmark(c: &mut Criterion) {
    c.bench_function("yolo_v8_model_loading", |b| {
        b.iter(|| {
            let mut engine = YoloV8Engine::new();
            let model_data = vec![0u8; 1024];
            
            let result = engine.load_model(black_box(&model_data));
            black_box(result);
        })
    });
}

criterion_group!(
    ai_benches,
    yolo_v8_inference_benchmark,
    ai_manager_benchmark,
    model_loading_benchmark
);

criterion_main!(ai_benches);
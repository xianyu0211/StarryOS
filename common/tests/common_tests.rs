// 通用共享库单元测试

use common::{Error, SystemError, DriverError, AIError, AppError, CommonResult};
use common::{BoundingBox, Detection, SensorData, PerformanceMode, LogLevel, TaskInfo};
use common::{calculate_mean, calculate_stddev, normalize_vector, dot_product};

#[test]
fn test_error_conversion() {
    // 测试系统错误转换
    let sys_err = SystemError::MemoryAllocationFailed;
    let err: Error = sys_err.into();
    assert!(matches!(err, Error::SystemError(SystemError::MemoryAllocationFailed)));
    
    // 测试驱动错误转换
    let drv_err = DriverError::InitializationFailed;
    let err: Error = drv_err.into();
    assert!(matches!(err, Error::DriverError(DriverError::InitializationFailed)));
    
    // 测试AI错误转换
    let ai_err = AIError::ModelLoadingFailed;
    let err: Error = ai_err.into();
    assert!(matches!(err, Error::AIError(AIError::ModelLoadingFailed)));
    
    // 测试应用错误转换
    let app_err = AppError::InitializationFailed;
    let err: Error = app_err.into();
    assert!(matches!(err, Error::AppError(AppError::InitializationFailed)));
}

#[test]
fn test_bounding_box() {
    let bbox = BoundingBox {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 150.0,
    };
    
    assert_eq!(bbox.x, 10.0);
    assert_eq!(bbox.y, 20.0);
    assert_eq!(bbox.width, 100.0);
    assert_eq!(bbox.height, 150.0);
    
    // 测试克隆
    let cloned_bbox = bbox;
    assert_eq!(cloned_bbox, bbox);
}

#[test]
fn test_sensor_data() {
    let sensor_data = SensorData {
        temperature: 25.5,
        humidity: 60.0,
        light_level: 300.0,
        motion_detected: true,
    };
    
    assert_eq!(sensor_data.temperature, 25.5);
    assert_eq!(sensor_data.humidity, 60.0);
    assert_eq!(sensor_data.light_level, 300.0);
    assert!(sensor_data.motion_detected);
}

#[test]
fn test_math_utils() {
    // 测试平均值计算
    let values = [1.0, 2.0, 3.0, 4.0, 5.0];
    let mean = calculate_mean(&values);
    assert_eq!(mean, 3.0);
    
    // 测试标准差计算
    let stddev = calculate_stddev(&values);
    assert!((stddev - 1.4142).abs() < 0.001);
    
    // 测试向量点积
    let v1 = [1.0, 2.0, 3.0];
    let v2 = [4.0, 5.0, 6.0];
    let dot = dot_product(&v1, &v2);
    assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
}

#[test]
fn test_task_info() {
    let task_info = TaskInfo {
        id: 1,
        name: "test_task",
        priority: 5,
        stack_size: 1024,
    };
    
    assert_eq!(task_info.id, 1);
    assert_eq!(task_info.name, "test_task");
    assert_eq!(task_info.priority, 5);
    assert_eq!(task_info.stack_size, 1024);
}
//! 音频处理驱动模块
//! 
//! 提供麦克风输入、扬声器输出、语音编解码等音频功能支持

mod microphone;
mod speaker;
mod codec;

use crate::{Driver, DriverError};
use alloc::vec::Vec;

/// 音频设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioDevice {
    Microphone,
    Speaker,
    Codec,
}

/// 音频配置参数
#[derive(Debug, Clone, Copy)]
pub struct AudioConfig {
    pub sample_rate: u32,      // 采样率 (Hz)
    pub channels: u8,         // 声道数
    pub bit_depth: u8,        // 位深度
    pub buffer_size: usize,   // 缓冲区大小
}

/// 音频数据格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    PCM16,      // 16位PCM
    PCM24,      // 24位PCM  
    PCM32,      // 32位PCM
    G711A,      // G.711 A律
    G711U,      // G.711 μ律
}

/// 音频驱动特征
pub trait AudioDriver: Driver {
    /// 开始录音
    fn start_recording(&mut self) -> Result<(), DriverError>;
    
    /// 停止录音
    fn stop_recording(&mut self) -> Result<(), DriverError>;
    
    /// 获取录音数据
    fn get_audio_data(&mut self, buffer: &mut [i16]) -> Result<usize, DriverError>;
    
    /// 播放音频数据
    fn play_audio(&mut self, data: &[i16]) -> Result<(), DriverError>;
    
    /// 设置音频参数
    fn set_config(&mut self, config: AudioConfig) -> Result<(), DriverError>;
}

/// 语音活动检测 (VAD)
pub struct VoiceActivityDetector {
    energy_threshold: f32,
    silence_duration: u32,
    speech_duration: u32,
}

impl VoiceActivityDetector {
    /// 创建新的VAD检测器
    pub fn new(energy_threshold: f32) -> Self {
        Self {
            energy_threshold,
            silence_duration: 0,
            speech_duration: 0,
        }
    }
    
    /// 检测语音活动
    pub fn detect_voice_activity(&mut self, audio_data: &[i16]) -> bool {
        // 计算音频能量
        let energy = self.calculate_energy(audio_data);
        
        if energy > self.energy_threshold {
            self.speech_duration += 1;
            self.silence_duration = 0;
            self.speech_duration > 3 // 连续3帧语音才认为是有效语音
        } else {
            self.silence_duration += 1;
            self.speech_duration = 0;
            false
        }
    }
    
    /// 计算音频能量
    fn calculate_energy(&self, audio_data: &[i16]) -> f32 {
        let sum_squares: f32 = audio_data.iter()
            .map(|&sample| (sample as f32).powi(2))
            .sum();
        
        (sum_squares / audio_data.len() as f32).sqrt()
    }
}

/// 音频管理器
pub struct AudioManager {
    devices: Vec<Box<dyn AudioDriver>>,
    vad: VoiceActivityDetector,
    recording: bool,
}

impl AudioManager {
    /// 创建新的音频管理器
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            vad: VoiceActivityDetector::new(1000.0), // 能量阈值
            recording: false,
        }
    }
    
    /// 注册音频设备
    pub fn register_device(&mut self, device: Box<dyn AudioDriver>) {
        self.devices.push(device);
    }
    
    /// 开始语音识别
    pub fn start_voice_recognition(&mut self) -> Result<(), DriverError> {
        if let Some(device) = self.devices.first_mut() {
            device.start_recording()?;
            self.recording = true;
        }
        Ok(())
    }
    
    /// 处理音频数据
    pub fn process_audio_data(&mut self) -> Result<Option<Vec<i16>>, DriverError> {
        if !self.recording {
            return Ok(None);
        }
        
        if let Some(device) = self.devices.first_mut() {
            let mut buffer = vec![0i16; 1600]; // 100ms @ 16kHz
            let len = device.get_audio_data(&mut buffer)?;
            
            if len > 0 {
                let audio_data = buffer[..len].to_vec();
                
                // 语音活动检测
                if self.vad.detect_voice_activity(&audio_data) {
                    return Ok(Some(audio_data));
                }
            }
        }
        
        Ok(None)
    }
    
    /// 播放语音响应
    pub fn play_voice_response(&mut self, audio_data: &[i16]) -> Result<(), DriverError> {
        if let Some(device) = self.devices.first_mut() {
            device.play_audio(audio_data)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
}
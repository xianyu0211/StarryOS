//! 音频编解码模块
//! 
//! 提供音频数据的编解码功能，支持多种音频格式

use crate::DriverError;
use alloc::vec::Vec;

/// 音频编解码器
pub struct AudioCodec {
    format: AudioFormat,
    sample_rate: u32,
    channels: u8,
}

/// 支持的音频格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    PCM16,      // 16位PCM
    PCM24,      // 24位PCM  
    PCM32,      // 32位PCM
    G711A,      // G.711 A律
    G711U,      // G.711 μ律
    OPUS,       // Opus编码
}

impl AudioCodec {
    /// 创建新的音频编解码器
    pub fn new(format: AudioFormat, sample_rate: u32, channels: u8) -> Self {
        Self {
            format,
            sample_rate,
            channels,
        }
    }
    
    /// PCM编码（无压缩）
    fn encode_pcm(&self, audio_data: &[f32]) -> Vec<u8> {
        match self.format {
            AudioFormat::PCM16 => {
                audio_data.iter()
                    .flat_map(|&sample| {
                        let sample_i16 = (sample * 32767.0).max(-32768.0).min(32767.0) as i16;
                        sample_i16.to_le_bytes().to_vec()
                    })
                    .collect()
            }
            AudioFormat::PCM24 => {
                // 24位PCM编码
                audio_data.iter()
                    .flat_map(|&sample| {
                        let sample_i32 = (sample * 8388607.0) as i32; // 2^23 - 1
                        let bytes = sample_i32.to_le_bytes();
                        vec![bytes[0], bytes[1], bytes[2]] // 取低24位
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }
    
    /// PCM解码
    fn decode_pcm(&self, encoded_data: &[u8]) -> Result<Vec<f32>, DriverError> {
        match self.format {
            AudioFormat::PCM16 => {
                if encoded_data.len() % 2 != 0 {
                    return Err(DriverError::InvalidParameter);
                }
                
                let mut audio_data = Vec::with_capacity(encoded_data.len() / 2);
                
                for chunk in encoded_data.chunks(2) {
                    let sample_i16 = i16::from_le_bytes([chunk[0], chunk[1]]);
                    audio_data.push(sample_i16 as f32 / 32768.0);
                }
                
                Ok(audio_data)
            }
            AudioFormat::PCM24 => {
                if encoded_data.len() % 3 != 0 {
                    return Err(DriverError::InvalidParameter);
                }
                
                let mut audio_data = Vec::with_capacity(encoded_data.len() / 3);
                
                for chunk in encoded_data.chunks(3) {
                    // 将24位数据扩展到32位
                    let mut bytes = [0u8; 4];
                    bytes[0..3].copy_from_slice(chunk);
                    // 处理符号位扩展
                    let sample_i32 = if chunk[2] & 0x80 != 0 {
                        i32::from_le_bytes(bytes) | 0xFF000000
                    } else {
                        i32::from_le_bytes(bytes)
                    };
                    
                    audio_data.push(sample_i32 as f32 / 8388608.0);
                }
                
                Ok(audio_data)
            }
            _ => Err(DriverError::NotSupported),
        }
    }
    
    /// G.711 A律编码
    fn encode_g711a(&self, audio_data: &[f32]) -> Vec<u8> {
        audio_data.iter()
            .map(|&sample| {
                let sample_i16 = (sample * 32767.0) as i16;
                self.linear_to_alaw(sample_i16)
            })
            .collect()
    }
    
    /// G.711 A律解码
    fn decode_g711a(&self, encoded_data: &[u8]) -> Vec<f32> {
        encoded_data.iter()
            .map(|&byte| {
                let sample_i16 = self.alaw_to_linear(byte);
                sample_i16 as f32 / 32768.0
            })
            .collect()
    }
    
    /// 线性PCM到A律转换
    fn linear_to_alaw(&self, sample: i16) -> u8 {
        // 简化的A律编码实现
        // 实际实现应该遵循ITU-T G.711标准
        
        let sign = if sample < 0 { 0x00 } else { 0x80 };
        let abs_sample = sample.abs() as u16;
        
        if abs_sample <= 32 {
            (abs_sample >> 1) as u8 | sign
        } else {
            // 简化的分段量化
            let segment = (abs_sample.ilog2() as u8).saturating_sub(5);
            let quantization = (abs_sample >> (segment + 1)) as u8 & 0x0F;
            
            ((segment << 4) | quantization) | sign
        }
    }
    
    /// A律到线性PCM转换
    fn alaw_to_linear(&self, byte: u8) -> i16 {
        // 简化的A律解码实现
        
        let sign = if byte & 0x80 != 0 { 1 } else { -1 };
        let abs_byte = (byte & 0x7F) as u16;
        
        if abs_byte <= 15 {
            sign * (abs_byte << 1) as i16
        } else {
            let segment = (abs_byte >> 4) as u8;
            let quantization = (abs_byte & 0x0F) as u16;
            
            let base = 1 << (segment + 4);
            let offset = quantization << (segment + 1);
            
            sign * (base + offset) as i16
        }
    }
    
    /// 编码音频数据
    pub fn encode(&self, audio_data: &[f32]) -> Result<Vec<u8>, DriverError> {
        match self.format {
            AudioFormat::PCM16 | AudioFormat::PCM24 => Ok(self.encode_pcm(audio_data)),
            AudioFormat::G711A => Ok(self.encode_g711a(audio_data)),
            AudioFormat::G711U => {
                // G.711 μ律编码（类似A律）
                Ok(self.encode_g711a(audio_data)) // 简化实现
            }
            _ => Err(DriverError::NotSupported),
        }
    }
    
    /// 解码音频数据
    pub fn decode(&self, encoded_data: &[u8]) -> Result<Vec<f32>, DriverError> {
        match self.format {
            AudioFormat::PCM16 | AudioFormat::PCM24 => self.decode_pcm(encoded_data),
            AudioFormat::G711A => Ok(self.decode_g711a(encoded_data)),
            AudioFormat::G711U => {
                // G.711 μ律解码（类似A律）
                Ok(self.decode_g711a(encoded_data)) // 简化实现
            }
            _ => Err(DriverError::NotSupported),
        }
    }
    
    /// 获取压缩比
    pub fn get_compression_ratio(&self) -> f32 {
        match self.format {
            AudioFormat::PCM16 => 1.0,
            AudioFormat::PCM24 => 1.5,
            AudioFormat::G711A | AudioFormat::G711U => 2.0,
            AudioFormat::OPUS => 4.0,
            _ => 1.0,
        }
    }
}
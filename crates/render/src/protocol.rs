//! Protocol module - FlatBuffers encoding/decoding for streaming
//!
//! Provides zero-copy parsing and efficient frame encoding/decoding
//! for the MathCore streaming protocol.

use flatbuffers::FlatBufferBuilder;
use std::collections::HashMap;
use std::sync::Arc;

pub use crate::stream::{
    BackpressureConfig, BackpressureState, ControlMessage, GraphicsFrameData,
    ProgressData, ProgressStatus, ProofStepData, StreamBuffer, StreamError,
    StreamMessage, StreamPublisher, StreamSubscriber, StepType,
};

// FlatBuffers generated types would go here
// For now, we use manual implementation that matches the schema

/// Frame encoder - encodes messages to FlatBuffers
pub struct FrameEncoder {
    // Encoding state
}

impl FrameEncoder {
    pub fn new() -> Self {
        Self {}
    }

    /// Encode a proof step
    pub fn encode_proof_step(&self, step: &ProofStepData) -> Result<Vec<u8>, StreamError> {
        let mut builder = FlatBufferBuilder::new();
        
        // Create content string
        let content = builder.create_string(&step.content);
        
        // Create dependencies vector
        let deps = builder.create_vector(&step.dependencies);
        
        // Create metadata (empty for now)
        // let metadata = ...
        
        // Build the ProofStep
        // For now, just serialize the data as a simple binary format
        // In production, this would use the generated FlatBuffers code
        
        let mut data = Vec::new();
        
        // Simple binary encoding (for demonstration)
        // Magic number
        data.extend_from_slice(b"PRF1");
        
        // step_id (u64)
        data.extend_from_slice(&step.step_id.to_le_bytes());
        
        // proof_id (u64)
        data.extend_from_slice(&step.proof_id.to_le_bytes());
        
        // content length + content
        let content_bytes = step.content.as_bytes();
        data.extend_from_slice(&(content_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(content_bytes);
        
        // step_type (u8)
        data.push(step.step_type as u8);
        
        // dependencies length + data
        data.extend_from_slice(&(step.dependencies.len() as u32).to_le_bytes());
        for dep in &step.dependencies {
            data.extend_from_slice(&dep.to_le_bytes());
        }
        
        // timestamp (i64)
        data.extend_from_slice(&step.timestamp.to_le_bytes());
        
        // confidence (f32)
        data.extend_from_slice(&step.confidence.to_le_bytes());
        
        Ok(data)
    }

    /// Encode a graphics frame
    pub fn encode_graphics_frame(&self, frame: &GraphicsFrameData) -> Result<Vec<u8>, StreamError> {
        let mut data = Vec::new();
        
        // Magic number
        data.extend_from_slice(b"GFX1");
        
        // frame_id (u64)
        data.extend_from_slice(&frame.frame_id.to_le_bytes());
        
        // timestamp (i64)
        data.extend_from_slice(&frame.timestamp.to_le_bytes());
        
        // width, height (u16)
        data.extend_from_slice(&frame.width.to_le_bytes());
        data.extend_from_slice(&frame.height.to_le_bytes());
        
        // pixels length + pixels
        data.extend_from_slice(&(frame.pixels.len() as u32).to_le_bytes());
        data.extend_from_slice(&frame.pixels);
        
        // shm_id length + content
        let shm_bytes = frame.shm_id.as_bytes();
        data.extend_from_slice(&(shm_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(shm_bytes);
        
        Ok(data)
    }

    /// Encode a progress update
    pub fn encode_progress(&self, progress: &ProgressData) -> Result<Vec<u8>, StreamError> {
        let mut data = Vec::new();
        
        // Magic number
        data.extend_from_slice(b"PRG1");
        
        // proof_id (u64)
        data.extend_from_slice(&progress.proof_id.to_le_bytes());
        
        // current_step, total_steps (u64)
        data.extend_from_slice(&progress.current_step.to_le_bytes());
        data.extend_from_slice(&progress.total_steps.to_le_bytes());
        
        // percentage (f32)
        data.extend_from_slice(&progress.percentage.to_le_bytes());
        
        // status (u8)
        data.push(progress.status as u8);
        
        // message length + content
        let msg_bytes = progress.message.as_bytes();
        data.extend_from_slice(&(msg_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(msg_bytes);
        
        // timestamp (i64)
        data.extend_from_slice(&progress.timestamp.to_le_bytes());
        
        Ok(data)
    }

    /// Encode any stream message
    pub fn encode(&self, msg: &StreamMessage) -> Result<Vec<u8>, StreamError> {
        match msg {
            StreamMessage::ProofStep(step) => self.encode_proof_step(step),
            StreamMessage::GraphicsFrame(frame) => self.encode_graphics_frame(frame),
            StreamMessage::Progress(progress) => self.encode_progress(progress),
            StreamMessage::Heartbeat => Ok(vec![b'H', b'E', b'A', b'R']),
            StreamMessage::Control(_) => Ok(vec![b'C', b'T', b'R', b'L']),
        }
    }
}

/// Frame decoder - decodes FlatBuffers to messages
pub struct FrameDecoder {
    // Decoding state
}

impl FrameDecoder {
    pub fn new() -> Self {
        Self {}
    }

    /// Decode a proof step
    pub fn decode_proof_step(&self, data: &[u8]) -> Result<ProofStepData, StreamError> {
        if data.len() < 4 + 8 + 8 + 4 {
            return Err(StreamError::Serialization("Data too short".to_string()));
        }
        
        // Check magic
        if &data[0..4] != b"PRF1" {
            return Err(StreamError::Serialization("Invalid magic".to_string()));
        }
        
        let mut offset = 4;
        
        let step_id = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        offset += 8;
        
        let proof_id = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        offset += 8;
        
        let content_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        
        let content = String::from_utf8(data[offset..offset+content_len].to_vec())
            .map_err(|e| StreamError::Serialization(e.to_string()))?;
        offset += content_len;
        
        let step_type = StepType::from_u8(data[offset]);
        offset += 1;
        
        let dep_count = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        
        let mut dependencies = Vec::with_capacity(dep_count);
        for _ in 0..dep_count {
            let dep = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
            dependencies.push(dep);
            offset += 8;
        }
        
        let timestamp = i64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        offset += 8;
        
        let confidence = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
        
        Ok(ProofStepData {
            step_id,
            proof_id,
            content,
            step_type,
            dependencies,
            timestamp,
            confidence,
        })
    }

    /// Decode a graphics frame
    pub fn decode_graphics_frame(&self, data: &[u8]) -> Result<GraphicsFrameData, StreamError> {
        if data.len() < 4 + 8 + 8 + 2 + 2 + 4 {
            return Err(StreamError::Serialization("Data too short".to_string()));
        }
        
        // Check magic
        if &data[0..4] != b"GFX1" {
            return Err(StreamError::Serialization("Invalid magic".to_string()));
        }
        
        let mut offset = 4;
        
        let frame_id = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        offset += 8;
        
        let timestamp = i64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        offset += 8;
        
        let width = u16::from_le_bytes(data[offset..offset+2].try_into().unwrap());
        offset += 2;
        
        let height = u16::from_le_bytes(data[offset..offset+2].try_into().unwrap());
        offset += 2;
        
        let pixel_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        
        let pixels = data[offset..offset+pixel_len].to_vec();
        offset += pixel_len;
        
        let shm_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        
        let shm_id = String::from_utf8(data[offset..offset+shm_len].to_vec())
            .map_err(|e| StreamError::Serialization(e.to_string()))?;
        
        Ok(GraphicsFrameData {
            frame_id,
            timestamp,
            width,
            height,
            pixels,
            shm_id,
        })
    }

    /// Decode a progress update
    pub fn decode_progress(&self, data: &[u8]) -> Result<ProgressData, StreamError> {
        if data.len() < 4 + 8 + 8 + 8 + 4 + 1 + 4 {
            return Err(StreamError::Serialization("Data too short".to_string()));
        }
        
        // Check magic
        if &data[0..4] != b"PRG1" {
            return Err(StreamError::Serialization("Invalid magic".to_string()));
        }
        
        let mut offset = 4;
        
        let proof_id = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        offset += 8;
        
        let current_step = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        offset += 8;
        
        let total_steps = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        offset += 8;
        
        let percentage = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
        offset += 4;
        
        let status = ProgressStatus::from_u8(data[offset]);
        offset += 1;
        
        let msg_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        
        let message = String::from_utf8(data[offset..offset+msg_len].to_vec())
            .map_err(|e| StreamError::Serialization(e.to_string()))?;
        offset += msg_len;
        
        let timestamp = i64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        
        Ok(ProgressData {
            proof_id,
            current_step,
            total_steps,
            percentage,
            status,
            message,
            timestamp,
        })
    }

    /// Decode any message (auto-detect type)
    pub fn decode(&self, data: &[u8]) -> Result<StreamMessage, StreamError> {
        if data.len() < 4 {
            return Err(StreamError::Serialization("Data too short".to_string()));
        }
        
        match &data[0..4] {
            b"PRF1" => Ok(StreamMessage::ProofStep(self.decode_proof_step(data)?)),
            b"GFX1" => Ok(StreamMessage::GraphicsFrame(self.decode_graphics_frame(data)?)),
            b"PRG1" => Ok(StreamMessage::Progress(self.decode_progress(data)?)),
            b"HEAR" => Ok(StreamMessage::Heartbeat),
            b"CTRL" => Ok(StreamMessage::Control(ControlMessage::Start)),
            _ => Err(StreamError::Serialization("Unknown message type".to_string())),
        }
    }
}

/// Extension trait for StepType
impl StepType {
    pub fn from_u8(val: u8) -> Self {
        match val % 8 {
            0 => StepType::Axiom,
            1 => StepType::Definition,
            2 => StepType::Lemma,
            3 => StepType::Theorem,
            4 => StepType::Proof,
            5 => StepType::Corollary,
            6 => StepType::Computation,
            _ => StepType::Verification,
        }
    }
}

/// Extension trait for ProgressStatus
impl ProgressStatus {
    pub fn from_u8(val: u8) -> Self {
        match val % 5 {
            0 => ProgressStatus::Pending,
            1 => ProgressStatus::Running,
            2 => ProgressStatus::Completed,
            3 => ProgressStatus::Failed,
            _ => ProgressStatus::Cancelled,
        }
    }
}

/// Benchmark result
#[derive(Debug)]
pub struct BenchmarkResult {
    pub encode_ns: u64,
    pub decode_ns: u64,
    pub throughput_mbps: f64,
}

/// Protocol benchmark
pub fn benchmark_protocol(iterations: usize) -> BenchmarkResult {
    use std::time::Instant;
    
    let encoder = FrameEncoder::new();
    let decoder = FrameDecoder::new();
    
    // Create test data
    let proof_step = ProofStepData {
        step_id: 1,
        proof_id: 1,
        content: "x^2 + y^2 = z^2".to_string(),
        step_type: StepType::Theorem,
        dependencies: vec![1, 2, 3],
        timestamp: 1234567890,
        confidence: 0.95,
    };
    
    // Benchmark encode
    let start = Instant::now();
    for _ in 0..iterations {
        let encoded = encoder.encode_proof_step(&proof_step).unwrap();
        std::hint::black_box(encoded);
    }
    let encode_ns = start.elapsed().as_nanos() as u64 / iterations as u64;
    
    // Benchmark decode
    let encoded = encoder.encode_proof_step(&proof_step).unwrap();
    let start = Instant::now();
    for _ in 0..iterations {
        let decoded = decoder.decode_proof_step(&encoded).unwrap();
        std::hint::black_box(decoded);
    }
    let decode_ns = start.elapsed().as_nanos() as u64 / iterations as u64;
    
    // Calculate throughput
    let data_size = encoded.len();
    let throughput_mbps = (data_size as f64 * 1000.0) / (encode_ns as f64 / 1000.0) / 1_000_000.0;
    
    BenchmarkResult {
        encode_ns,
        decode_ns,
        throughput_mbps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_proof_step() {
        let encoder = FrameEncoder::new();
        let decoder = FrameDecoder::new();
        
        let step = ProofStepData {
            step_id: 42,
            proof_id: 1,
            content: "test proof".to_string(),
            step_type: StepType::Proof,
            dependencies: vec![1, 2],
            timestamp: 1234567890,
            confidence: 0.99,
        };
        
        let encoded = encoder.encode_proof_step(&step).unwrap();
        let decoded = decoder.decode_proof_step(&encoded).unwrap();
        
        assert_eq!(step.step_id, decoded.step_id);
        assert_eq!(step.proof_id, decoded.proof_id);
        assert_eq!(step.content, decoded.content);
        assert_eq!(step.step_type, decoded.step_type);
    }

    #[test]
    fn test_encode_decode_graphics_frame() {
        let encoder = FrameEncoder::new();
        let decoder = FrameDecoder::new();
        
        let frame = GraphicsFrameData {
            frame_id: 1,
            timestamp: 1234567890,
            width: 800,
            height: 600,
            pixels: vec![0u8; 800 * 600 * 4],
            shm_id: "shm://frame1".to_string(),
        };
        
        let encoded = encoder.encode_graphics_frame(&frame).unwrap();
        let decoded = decoder.decode_graphics_frame(&encoded).unwrap();
        
        assert_eq!(frame.frame_id, decoded.frame_id);
        assert_eq!(frame.width, decoded.width);
        assert_eq!(frame.height, decoded.height);
    }

    #[test]
    fn test_benchmark() {
        let result = benchmark_protocol(1000);
        println!("Encode: {} ns", result.encode_ns);
        println!("Decode: {} ns", result.decode_ns);
        println!("Throughput: {:.2} MB/s", result.throughput_mbps);
        
        // Verify targets - relaxed for CI/test environments
        assert!(result.encode_ns < 10000); // < 10µs for encoding
        assert!(result.decode_ns < 5000);   // < 5µs for decoding

    }
}

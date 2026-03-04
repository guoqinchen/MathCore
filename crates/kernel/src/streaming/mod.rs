//! MathCore Streaming Module - FlatBuffers based real-time streaming protocol
//!
//! This module provides:
//! - FlatBuffers message serialization
//! - Streaming transport layer
//! - Backpressure handling

use flatbuffers::FlatBufferBuilder;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Stream message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamMessageType {
    Request,
    Response,
    Error,
    Heartbeat,
    ChunkStart,
    ChunkData,
    ChunkEnd,
}

/// Stream message header
#[derive(Debug, Clone)]
pub struct StreamHeader {
    pub msg_type: StreamMessageType,
    pub stream_id: u64,
    pub sequence: u64,
    pub timestamp_ns: u64,
    pub payload_size: u32,
}

/// Stream message with header and body
#[derive(Debug, Clone)]
pub struct StreamMessage {
    pub header: StreamHeader,
    pub payload: Vec<u8>,
}

/// Backpressure configuration
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    pub max_queue_size: usize,
    pub high_water_mark: usize,
    pub low_water_mark: usize,
    pub pause_duration_ms: u64,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            high_water_mark: 800,
            low_water_mark: 200,
            pause_duration_ms: 10,
        }
    }
}

/// Streaming transport layer
pub struct StreamingTransport {
    config: BackpressureConfig,
    sender: Option<mpsc::Sender<StreamMessage>>,
    receiver: Option<mpsc::Receiver<StreamMessage>>,
    is_paused: Arc<RwLock<bool>>,
}

impl StreamingTransport {
    /// Create a new streaming transport
    pub fn new(config: BackpressureConfig) -> Self {
        let (sender, receiver) = mpsc::channel(config.max_queue_size);
        Self {
            config,
            sender: Some(sender),
            receiver: Some(receiver),
            is_paused: Arc::new(RwLock::new(false)),
        }
    }

    /// Get sender half
    pub fn sender(&self) -> Option<mpsc::Sender<StreamMessage>> {
        self.sender.clone()
    }

    /// Get receiver half
    pub fn receiver(&mut self) -> Option<mpsc::Receiver<StreamMessage>> {
        self.receiver.take()
    }

    /// Check and apply backpressure
    pub async fn check_backpressure(&self, queue_len: usize) -> bool {
        if queue_len >= self.config.high_water_mark {
            let mut paused = self.is_paused.write().await;
            if !*paused {
                *paused = true;
                return true;
            }
        } else if queue_len <= self.config.low_water_mark {
            let mut paused = self.is_paused.write().await;
            if *paused {
                *paused = false;
                return false;
            }
        }
        false
    }

    /// Send a message
    pub async fn send(&self, msg: StreamMessage) -> Result<(), StreamError> {
        if let Some(ref sender) = self.sender {
            sender.send(msg).await.map_err(|_| StreamError::SendError)
        } else {
            Err(StreamError::TransportClosed)
        }
    }
}

/// Streaming errors
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("Failed to serialize message")]
    SerializationError,

    #[error("Failed to deserialize message")]
    DeserializationError,

    #[error("Send error")]
    SendError,

    #[error("Receive error")]
    ReceiveError,

    #[error("Transport closed")]
    TransportClosed,

    #[error("Backpressure applied")]
    Backpressure,

    #[error("Invalid message format")]
    InvalidFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backpressure_config_default() {
        let config = BackpressureConfig::default();
        assert_eq!(config.max_queue_size, 1000);
        assert_eq!(config.high_water_mark, 800);
        assert_eq!(config.low_water_mark, 200);
    }

    #[tokio::test]
    async fn test_transport_creation() {
        let transport = StreamingTransport::new(BackpressureConfig::default());
        assert!(transport.sender().is_some());
    }
}

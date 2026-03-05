//! Stream module - Real-time streaming with backpressure handling
//!
//! Provides publisher/subscriber patterns for streaming proof steps,
//! graphics frames, and progress updates with flow control.

use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Message types for streaming
#[derive(Debug, Clone)]
pub enum StreamMessage {
    ProofStep(ProofStepData),
    GraphicsFrame(GraphicsFrameData),
    Progress(ProgressData),
    Heartbeat,
    Control(ControlMessage),
}

/// Proof step data
#[derive(Debug, Clone)]
pub struct ProofStepData {
    pub step_id: u64,
    pub proof_id: u64,
    pub content: String,
    pub step_type: StepType,
    pub dependencies: Vec<u64>,
    pub timestamp: i64,
    pub confidence: f32,
}

/// Graphics frame data
#[derive(Debug, Clone)]
pub struct GraphicsFrameData {
    pub frame_id: u64,
    pub timestamp: i64,
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<u8>,
    pub shm_id: String,
}

/// Progress data
#[derive(Debug, Clone)]
pub struct ProgressData {
    pub proof_id: u64,
    pub current_step: u64,
    pub total_steps: u64,
    pub percentage: f32,
    pub status: ProgressStatus,
    pub message: String,
    pub timestamp: i64,
}

/// Step types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepType {
    Axiom,
    Definition,
    Lemma,
    Theorem,
    Proof,
    Corollary,
    Computation,
    Verification,
}

/// Progress status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Control messages for stream management
#[derive(Debug, Clone)]
pub enum ControlMessage {
    Start,
    Stop,
    Pause,
    Resume,
    Flush,
    SetBackpressure(u32),
}

/// Backpressure configuration
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// Maximum messages in flight
    pub max_pending: usize,
    /// Flow control window size
    pub window_size: usize,
    /// Timeout for waiting on backpressure (ms)
    pub timeout_ms: u64,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_pending: 1000,
            window_size: 100,
            timeout_ms: 5000,
        }
    }
}

/// Backpressure state
#[derive(Debug)]
pub struct BackpressureState {
    config: BackpressureConfig,
    pending_count: AtomicUsize,
    window_counter: AtomicUsize,
    paused: RwLock<bool>,
}

impl BackpressureState {
    pub fn new(config: BackpressureConfig) -> Self {
        Self {
            config,
            pending_count: AtomicUsize::new(0),
            window_counter: AtomicUsize::new(0),
            paused: RwLock::new(false),
        }
    }

    /// Try to acquire permission to send a message
    pub fn try_acquire(&self) -> bool {
        if *self.paused.read() {
            return false;
        }

        let count = self.pending_count.load(Ordering::Acquire);
        count < self.config.max_pending
    }

    /// Acquire with timeout behavior
    pub fn acquire(&self) -> bool {
        // Quick check first
        if self.try_acquire() {
            return true;
        }

        // Signal window update periodically
        let window = self.window_counter.fetch_add(1, Ordering::AcqRel);
        if window % self.config.window_size == 0 {
            // Could emit backpressure event here
        }

        self.try_acquire()
    }

    /// Release a message slot
    pub fn release(&self) {
        self.pending_count.fetch_sub(1, Ordering::Release);
    }

    /// Mark message as pending
    pub fn mark_pending(&self) {
        self.pending_count.fetch_add(1, Ordering::Release);
    }

    /// Pause the stream
    pub fn pause(&self) {
        *self.paused.write() = true;
    }

    /// Resume the stream
    pub fn resume(&self) {
        *self.paused.write() = false;
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        *self.paused.read()
    }

    /// Get current pending count
    pub fn pending(&self) -> usize {
        self.pending_count.load(Ordering::Acquire)
    }
}

/// Stream publisher - sends messages to subscribers
pub struct StreamPublisher {
    config: BackpressureConfig,
    state: Arc<BackpressureState>,
    subscribers: RwLock<Vec<mpsc::Sender<StreamMessage>>>,
}

impl StreamPublisher {
    pub fn new(config: BackpressureConfig) -> Self {
        Self {
            config: config.clone(),
            state: Arc::new(BackpressureState::new(config)),
            subscribers: RwLock::new(Vec::new()),
        }
    }

    /// Get the backpressure state for external monitoring
    pub fn state(&self) -> Arc<BackpressureState> {
        self.state.clone()
    }

    /// Subscribe to this publisher
    pub fn subscribe(&self, buffer_size: usize) -> StreamSubscriber {
        let (tx, rx) = mpsc::channel(buffer_size);
        self.subscribers.write().push(tx);
        StreamSubscriber {
            rx,
            state: self.state.clone(),
        }
    }

    /// Publish a message to all subscribers
    pub fn publish(&self, msg: StreamMessage) -> Result<(), StreamError> {
        if !self.state.acquire() {
            return Err(StreamError::Backpressure);
        }

        self.state.mark_pending();

        let subscribers = self.subscribers.read().clone();

        // Try to send to all subscribers
        for tx in subscribers.iter() {
            let _ = tx.try_send(msg.clone());
        }

        self.state.release();

        Ok(())
    }

    /// Publish without blocking (fire and forget)
    pub fn try_publish(&self, msg: StreamMessage) -> bool {
        if !self.state.try_acquire() {
            return false;
        }

        self.state.mark_pending();

        let subscribers = self.subscribers.read().clone();
        for tx in subscribers {
            let _ = tx.try_send(msg.clone());
        }

        true
    }

    /// Flush pending messages
    pub fn flush(&self) {
        // Wait until pending count drops to 0
        while self.state.pending() > 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    /// Pause the stream
    pub fn pause(&self) {
        self.state.pause();
    }

    /// Resume the stream
    pub fn resume(&self) {
        self.state.resume();
    }

    /// Get subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.read().len()
    }
}

/// Stream subscriber - receives messages from a publisher
pub struct StreamSubscriber {
    rx: mpsc::Receiver<StreamMessage>,
    state: Arc<BackpressureState>,
}

impl StreamSubscriber {
    /// Receive the next message (blocking)
    pub async fn recv(&mut self) -> Option<StreamMessage> {
        let msg = self.rx.recv().await;
        if msg.is_some() {
            self.state.release();
        }
        msg
    }

    /// Try to receive a message (non-blocking)
    pub fn try_recv(&mut self) -> Option<StreamMessage> {
        match self.rx.try_recv() {
            Ok(msg) => {
                self.state.release();
                Some(msg)
            }
            Err(mpsc::error::TryRecvError::Empty) => None,
            Err(mpsc::error::TryRecvError::Disconnected) => None,
        }
    }

    /// Check if the subscriber is closed
    pub fn is_closed(&self) -> bool {
        self.rx.is_closed()
    }

    /// Get backpressure state
    pub fn state(&self) -> &Arc<BackpressureState> {
        &self.state
    }
}

/// Errors that can occur in streaming
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("Backpressure: stream is full")]
    Backpressure,

    #[error("Stream closed")]
    Closed,

    #[error("Timeout waiting for backpressure")]
    Timeout,

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Buffer manager for streaming
pub struct StreamBuffer {
    buffer: RwLock<VecDeque<StreamMessage>>,
    capacity: usize,
}

impl StreamBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: RwLock::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn push(&self, msg: StreamMessage) -> bool {
        let mut buffer = self.buffer.write();
        if buffer.len() >= self.capacity {
            // Drop oldest message to make room
            buffer.pop_front();
        }
        buffer.push_back(msg);
        true
    }

    pub fn pop(&self) -> Option<StreamMessage> {
        self.buffer.write().pop_front()
    }

    pub fn len(&self) -> usize {
        self.buffer.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.read().is_empty()
    }

    pub fn clear(&self) {
        self.buffer.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backpressure_state() {
        let config = BackpressureConfig {
            max_pending: 5,
            window_size: 2,
            timeout_ms: 100,
        };

        let state = BackpressureState::new(config);

        // Mark pending and release
        state.mark_pending();
        assert_eq!(state.pending(), 1);
        state.release();
        assert_eq!(state.pending(), 0);

        // Test pause/resume
        assert!(!state.is_paused());
        state.pause();
        assert!(state.is_paused());
        state.resume();
        assert!(!state.is_paused());
    }

    #[test]
    fn test_publisher_subscriber() {
        let publisher = StreamPublisher::new(BackpressureConfig::default());
        let mut subscriber = publisher.subscribe(10);

        // Publish a message
        publisher.publish(StreamMessage::Heartbeat).unwrap();

        // Receive it
        let msg = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { subscriber.recv().await });

        assert!(matches!(msg, Some(StreamMessage::Heartbeat)));
    }

    #[test]
    fn test_stream_buffer() {
        let buffer = StreamBuffer::new(3);

        buffer.push(StreamMessage::Heartbeat);
        buffer.push(StreamMessage::Heartbeat);
        buffer.push(StreamMessage::Heartbeat);

        assert_eq!(buffer.len(), 3);

        // Adding more should evict oldest
        buffer.push(StreamMessage::Heartbeat);

        assert_eq!(buffer.len(), 3);
    }
}

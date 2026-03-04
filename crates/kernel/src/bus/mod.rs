//! Message bus for inter-component communication

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Message metadata
#[derive(Debug, Clone)]
pub struct MessageMetadata {
    pub topic: String,
    pub priority: MessagePriority,
    pub timestamp: std::time::Instant,
    pub sender: String,
}

/// Generic message type
#[derive(Debug, Clone)]
pub struct Message {
    pub payload: Vec<u8>,
    pub metadata: MessageMetadata,
}

/// Topic subscription
pub struct Subscription {
    pub topic: String,
    pub receiver: mpsc::Receiver<Message>,
}

/// Subscriber handle
pub struct Subscriber {
    pub id: String,
    pub subscriptions: Vec<String>,
}

/// Bus statistics
#[derive(Debug, Clone, Default)]
pub struct BusStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub subscriptions_count: usize,
}

/// Bus statistics snapshot
#[derive(Debug, Clone)]
pub struct BusStatsSnapshot {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub subscriptions_count: usize,
    pub topics: Vec<String>,
}

/// Bus configuration
#[derive(Debug, Clone)]
pub struct BusConfig {
    pub max_queue_size: usize,
    pub default_priority: MessagePriority,
}

impl Default for BusConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            default_priority: MessagePriority::Normal,
        }
    }
}

/// Request for request-response pattern
#[derive(Debug, Clone)]
pub struct Request {
    pub topic: String,
    pub payload: Vec<u8>,
    pub timeout: std::time::Duration,
}

/// Request builder
pub struct RequestBuilder {
    topic: String,
    payload: Vec<u8>,
    timeout: std::time::Duration,
}

impl RequestBuilder {
    pub fn new(topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            payload: Vec::new(),
            timeout: std::time::Duration::from_secs(30),
        }
    }

    pub fn payload(mut self, payload: Vec<u8>) -> Self {
        self.payload = payload;
        self
    }

    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> Request {
        Request {
            topic: self.topic,
            payload: self.payload,
            timeout: self.timeout,
        }
    }
}

/// Response type
#[derive(Debug, Clone)]
pub struct Response {
    pub payload: Vec<u8>,
    pub success: bool,
    pub error: Option<String>,
}

/// Topic handle for publishing
#[derive(Debug, Clone)]
pub struct Topic {
    pub name: String,
}

/// Message bus for pub/sub and request/response
#[derive(Debug)]
pub struct Bus {
    config: BusConfig,
    subscribers: Arc<RwLock<HashMap<String, Vec<mpsc::Sender<Message>>>>>,
    stats: Arc<RwLock<BusStats>>,
}

impl Bus {
    pub fn new(config: BusConfig) -> Self {
        Self {
            config,
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(BusStats::default())),
        }
    }

    pub fn subscribe(&self, topic: &str) -> Subscription {
        let (tx, rx) = mpsc::channel(self.config.max_queue_size);
        {
            let mut subs = self.subscribers.write();
            subs.entry(topic.to_string())
                .or_insert_with(Vec::new)
                .push(tx);
        }
        Subscription {
            topic: topic.to_string(),
            receiver: rx,
        }
    }

    pub fn publish(&self, topic: &str, message: Message) {
        let subs = self.subscribers.read();
        if let Some(senders) = subs.get(topic) {
            for sender in senders {
                let _ = sender.try_send(message.clone());
            }
            self.stats.write().messages_sent += 1;
        }
    }

    pub fn get_stats(&self) -> BusStatsSnapshot {
        let stats = self.stats.read();
        let subs = self.subscribers.read();
        BusStatsSnapshot {
            messages_sent: stats.messages_sent,
            messages_received: stats.messages_received,
            subscriptions_count: subs.values().map(|v| v.len()).sum(),
            topics: subs.keys().cloned().collect(),
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new(BusConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_create() {
        let bus = Bus::new(BusConfig::default());
        let stats = bus.get_stats();
        assert_eq!(stats.messages_sent, 0);
    }

    #[test]
    fn test_bus_subscribe() {
        let bus = Bus::default();
        let _sub = bus.subscribe("test-topic");
        let stats = bus.get_stats();
        assert!(stats.subscriptions_count > 0);
    }

    #[test]
    fn test_bus_publish() {
        let bus = Bus::default();
        let sub = bus.subscribe("test-topic");

        let message = Message {
            payload: b"hello".to_vec(),
            metadata: MessageMetadata {
                topic: "test-topic".to_string(),
                priority: MessagePriority::Normal,
                timestamp: std::time::Instant::now(),
                sender: "test".to_string(),
            },
        };

        bus.publish("test-topic", message);

        // Check stats were updated
        let stats = bus.get_stats();
        assert_eq!(stats.messages_sent, 1);
    }

    #[test]
    fn test_request_builder() {
        let request = RequestBuilder::new("test-topic")
            .payload(b"test data".to_vec())
            .timeout(std::time::Duration::from_secs(5))
            .build();

        assert_eq!(request.topic, "test-topic");
        assert_eq!(request.payload, b"test data".to_vec());
    }
}

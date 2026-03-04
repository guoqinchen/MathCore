//! MathCore Kernel - Micro-kernel for high-performance mathematical computing

pub mod bus;
pub mod core;
pub mod error;
pub mod protocol;
pub mod sandbox;

pub use bus::{
    Bus, BusConfig, BusStats, BusStatsSnapshot, Message, MessageMetadata, MessagePriority, Request,
    RequestBuilder, Response, Subscriber, Subscription, Topic,
};
pub use core::{
    Kernel, KernelConfig, KernelState, KernelStats, KernelStatsSnapshot, PluginInfo,
    PluginMetadata, PluginState, ResourceQuota, ResourceUsage, ShutdownSignal,
};
pub use error::{ErrorKind, MathCoreError, ResourceType};
pub use protocol::*;
pub use sandbox::{ExecutionResult, Sandbox, SandboxConfig, SandboxTrait};

pub type Result<T> = std::result::Result<T, MathCoreError>;

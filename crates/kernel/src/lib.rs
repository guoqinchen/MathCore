//! MathCore Kernel - Micro-kernel for high-performance mathematical computing
//!
//! This crate provides the core runtime environment for MathCore, including:
//! - **Plugin Management**: Load and manage computational plugins
//! - **Message Bus**: Inter-component communication with pub/sub pattern
//! - **Sandbox**: Secure execution environment for untrusted code
//! - **Protocol**: Message serialization and version negotiation
//! - **Streaming**: Real-time data streaming capabilities
//! - **Validation**: Input validation and type checking
//!
//! # Core Components
//!
//! - [`Kernel`] - Main kernel for plugin management and lifecycle
//! - [`Bus`] - Message bus for component communication
//! - [`Sandbox`] - Secure execution sandbox
//! - [`protocol`] - Message protocol definitions
//!
//! # Quick Start
//!
//! This crate provides kernel functionality for plugin management and execution.
//!
//! # Modules
//!
//! - [`bus`] - Message bus for pub/sub communication
//! - [`core`] - Kernel core and plugin management
//! - [`error`] - Error types and handling
//! - [`protocol`] - Message protocol definitions
//! - [`sandbox`] - Secure execution sandbox
//! - [`streaming`] - Real-time streaming
//! - [`validation`] - Input validation

pub mod bus;
pub mod core;
pub mod error;
pub mod protocol;
pub mod sandbox;
pub mod streaming;
pub mod validation;

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

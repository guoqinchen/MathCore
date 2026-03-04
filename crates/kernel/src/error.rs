//! Unified error types for MathCore kernel

use std::fmt;

/// Unified error type for MathCore kernel
#[derive(Debug)]
pub struct MathCoreError {
    kind: ErrorKind,
}

impl MathCoreError {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for MathCoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for MathCoreError {}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    BusTopicNotFound(String),
    BusSubscriptionFailed(String),
    BusPublishFailed(String),
    BusChannelClosed,
    BusFull,
    CoreInitFailed(String),
    CoreRunFailed(String),
    CoreShutdownFailed(String),
    CoreNotRunning,
    CoreAlreadyRunning,
    PluginNotFound(String),
    PluginLoadFailed(String),
    PluginUnloadFailed(String),
    PluginAlreadyLoaded(String),
    PluginInitFailed(String),
    PluginExecuteFailed(String),
    SandboxCreationFailed(String),
    SandboxExecutionDenied(String),
    SandboxResourceLimitExceeded(ResourceType),
    SandboxTimeout,
    SandboxProcessCrashed(i32),
    ResourceQuotaExceeded(String),
    ResourceNotAvailable(String),
    ResourceBusy(String),
    InvalidArgument(String),
    InvalidState(String),
    NotSupported(String),
    InternalError(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BusTopicNotFound(t) => write!(f, "Topic not found: {}", t),
            Self::BusSubscriptionFailed(m) => write!(f, "Subscription failed: {}", m),
            Self::BusPublishFailed(m) => write!(f, "Publish failed: {}", m),
            Self::BusChannelClosed => write!(f, "Bus channel closed"),
            Self::BusFull => write!(f, "Bus channel full"),
            Self::CoreInitFailed(m) => write!(f, "Core init failed: {}", m),
            Self::CoreRunFailed(m) => write!(f, "Core run failed: {}", m),
            Self::CoreShutdownFailed(m) => write!(f, "Core shutdown failed: {}", m),
            Self::CoreNotRunning => write!(f, "Core not running"),
            Self::CoreAlreadyRunning => write!(f, "Core already running"),
            Self::PluginNotFound(i) => write!(f, "Plugin not found: {}", i),
            Self::PluginLoadFailed(i) => write!(f, "Plugin load failed: {}", i),
            Self::PluginUnloadFailed(i) => write!(f, "Plugin unload failed: {}", i),
            Self::PluginAlreadyLoaded(i) => write!(f, "Plugin already loaded: {}", i),
            Self::PluginInitFailed(i) => write!(f, "Plugin init failed: {}", i),
            Self::PluginExecuteFailed(i) => write!(f, "Plugin execute failed: {}", i),
            Self::SandboxCreationFailed(m) => write!(f, "Sandbox creation failed: {}", m),
            Self::SandboxExecutionDenied(m) => write!(f, "Execution denied: {}", m),
            Self::SandboxResourceLimitExceeded(k) => write!(f, "Resource limit exceeded: {:?}", k),
            Self::SandboxTimeout => write!(f, "Sandbox timeout"),
            Self::SandboxProcessCrashed(c) => write!(f, "Process crashed: {}", c),
            Self::ResourceQuotaExceeded(n) => write!(f, "Resource quota exceeded: {}", n),
            Self::ResourceNotAvailable(n) => write!(f, "Resource not available: {}", n),
            Self::ResourceBusy(n) => write!(f, "Resource busy: {}", n),
            Self::InvalidArgument(m) => write!(f, "Invalid argument: {}", m),
            Self::InvalidState(m) => write!(f, "Invalid state: {}", m),
            Self::NotSupported(m) => write!(f, "Not supported: {}", m),
            Self::InternalError(m) => write!(f, "Internal error: {}", m),
        }
    }
}

impl std::error::Error for ErrorKind {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Memory,
    Cpu,
    Time,
    FileDescriptor,
    Thread,
    Storage,
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory => write!(f, "memory"),
            Self::Cpu => write!(f, "cpu"),
            Self::Time => write!(f, "time"),
            Self::FileDescriptor => write!(f, "file_descriptor"),
            Self::Thread => write!(f, "thread"),
            Self::Storage => write!(f, "storage"),
        }
    }
}

pub type Result<T> = std::result::Result<T, MathCoreError>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_error_display() {
        let err = MathCoreError::new(ErrorKind::BusTopicNotFound("test".into()));
        assert_eq!(err.to_string(), "Topic not found: test");
    }
}

//! Sandbox execution module

use crate::error::{ErrorKind, MathCoreError};
use std::collections::HashSet;

#[cfg(target_os = "linux")]
use seccomp::{Seccomp, Action, Rule};

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub max_memory: u64,
    pub max_cpu_time: u64,
    pub max_wall_time: u64,
    pub seccomp_enabled: bool,
    pub allowed_syscalls: HashSet<String>,
    pub max_processes: u32,
    pub max_file_size: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory: 256 * 1024 * 1024,
            max_cpu_time: 30_000,
            max_wall_time: 60_000,
            seccomp_enabled: false,
            allowed_syscalls: HashSet::new(),
            max_processes: 4,
            max_file_size: 64 * 1024 * 1024,
        }
    }
}

/// Execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub killed: bool,
    pub kill_reason: Option<String>,
}

/// Sandbox trait for code execution
pub trait SandboxTrait {
    fn execute(&self, code: &[u8]) -> Result<ExecutionResult, MathCoreError>;
    fn get_memory_usage(&self) -> u64;
    
    /// Initialize seccomp filter with allowed syscalls
    #[cfg(target_os = "linux")]
    fn init_seccomp(&mut self) -> Result<(), MathCoreError>;
    
    /// Check if seccomp is initialized
    #[cfg(target_os = "linux")]
    fn is_seccomp_initialized(&self) -> bool;
}

/// Sandbox implementation
#[derive(Debug)]
pub struct Sandbox {
    active: bool,
    config: SandboxConfig,
    memory_usage: u64,
    total_time: u64,
    #[cfg(target_os = "linux")]
    seccomp_filter: Option<Seccomp>,
}

impl Clone for Sandbox {
    fn clone(&self) -> Self {
        Self {
            active: self.active,
            config: self.config.clone(),
            memory_usage: self.memory_usage,
            total_time: self.total_time,
            #[cfg(target_os = "linux")]
            seccomp_filter: None, // Seccomp filter cannot be cloned, create a new one
        }
    }
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            active: true,
            config: SandboxConfig::default(),
            memory_usage: 0,
            total_time: 0,
            #[cfg(target_os = "linux")]
            seccomp_filter: None,
        }
    }

    pub fn with_config(config: SandboxConfig) -> Self {
        Self {
            active: true,
            config,
            memory_usage: 0,
            total_time: 0,
            #[cfg(target_os = "linux")]
            seccomp_filter: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn execute(&self, code: &[u8]) -> Result<ExecutionResult, MathCoreError> {
        if code.is_empty() {
            return Err(MathCoreError::new(ErrorKind::InvalidArgument(
                "Empty code".to_string(),
            )));
        }

        // Apply seccomp filter if enabled
        #[cfg(target_os = "linux")]
        if self.config.seccomp_enabled && self.seccomp_filter.is_none() {
            tracing::warn!("Seccomp is enabled but not initialized");
            return Err(MathCoreError::new(ErrorKind::SandboxError(
                "Seccomp filter not initialized".to_string(),
            )));
        }

        // Check resource limits
        if self.memory_usage > self.config.max_memory {
            return Ok(ExecutionResult {
                exit_code: -1,
                stdout: b"".to_vec(),
                stderr: b"Memory limit exceeded".to_vec(),
                killed: true,
                kill_reason: Some("Memory limit exceeded".to_string()),
            });
        }

        // Execute code
        let result = ExecutionResult {
            exit_code: 0,
            stdout: b"Executed".to_vec(),
            stderr: b"".to_vec(),
            killed: false,
            kill_reason: None,
        };

        // Update resource usage
        // TODO: Implement actual resource usage tracking

        Ok(result)
    }

    pub fn get_memory_usage(&self) -> u64 {
        self.memory_usage
    }

    pub fn get_total_execution_time(&self) -> u64 {
        self.total_time
    }

    /// Initialize seccomp filter with allowed syscalls
    #[cfg(target_os = "linux")]
    pub fn init_seccomp(&mut self) -> Result<(), MathCoreError> {
        if !self.config.seccomp_enabled {
            return Ok(());
        }

        tracing::debug!("Initializing seccomp filter with allowed syscalls: {:?}", self.config.allowed_syscalls);
        
        // Create a new seccomp filter with default action to kill
        let mut seccomp = Seccomp::new(Action::Kill)?;
        
        // Allow basic syscalls that most programs need
        let default_allowed = vec!["read", "write", "exit", "exit_group", "nanosleep"];
        
        // Add default allowed syscalls
        for syscall in default_allowed {
            if let Ok(rule) = Rule::new(syscall, Action::Allow) {
                seccomp.add_rule(rule)?;
            }
        }
        
        // Add user-specified allowed syscalls
        for syscall in &self.config.allowed_syscalls {
            if let Ok(rule) = Rule::new(syscall, Action::Allow) {
                seccomp.add_rule(rule)?;
            }
        }
        
        // Load the filter
        seccomp.load()?;
        
        // Store the seccomp filter
        self.seccomp_filter = Some(seccomp);
        Ok(())
    }

    /// Check if seccomp is initialized
    #[cfg(target_os = "linux")]
    pub fn is_seccomp_initialized(&self) -> bool {
        self.seccomp_filter.is_some()
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl SandboxTrait for Sandbox {
    fn execute(&self, code: &[u8]) -> Result<ExecutionResult, MathCoreError> {
        Sandbox::execute(self, code)
    }

    fn get_memory_usage(&self) -> u64 {
        Sandbox::get_memory_usage(self)
    }
    
    #[cfg(target_os = "linux")]
    fn init_seccomp(&mut self) -> Result<(), MathCoreError> {
        Sandbox::init_seccomp(self)
    }
    
    #[cfg(target_os = "linux")]
    fn is_seccomp_initialized(&self) -> bool {
        Sandbox::is_seccomp_initialized(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_create() {
        let sandbox = Sandbox::new();
        assert!(sandbox.is_active());
    }

    #[test]
    fn test_sandbox_execution() {
        let sandbox = Sandbox::new();
        let result = sandbox.execute(b"1 + 2").unwrap();
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.max_memory, 256 * 1024 * 1024);
        assert_eq!(config.max_cpu_time, 30_000);
        assert_eq!(config.max_wall_time, 60_000);
        assert!(!config.seccomp_enabled);
        assert_eq!(config.max_processes, 4);
        assert_eq!(config.max_file_size, 64 * 1024 * 1024);
    }

    #[test]
    fn test_sandbox_config_custom() {
        let mut syscalls = HashSet::new();
        syscalls.insert("read".to_string());
        syscalls.insert("write".to_string());

        let config = SandboxConfig {
            max_memory: 512 * 1024 * 1024,
            max_cpu_time: 60_000,
            max_wall_time: 120_000,
            seccomp_enabled: true,
            allowed_syscalls: syscalls,
            max_processes: 8,
            max_file_size: 128 * 1024 * 1024,
        };

        assert_eq!(config.max_memory, 512 * 1024 * 1024);
        assert!(config.seccomp_enabled);
        assert!(config.allowed_syscalls.contains("read"));
    }

    #[test]
    fn test_sandbox_memory_tracking() {
        let sandbox = Sandbox::new();
        assert_eq!(sandbox.get_memory_usage(), 0);
    }

    #[test]
    fn test_sandbox_execution_empty() {
        let sandbox = Sandbox::new();
        let result = sandbox.execute(b"");
        assert!(result.is_err());
    }

    #[test]
    fn test_execution_result() {
        let result = ExecutionResult {
            exit_code: 0,
            stdout: b"hello".to_vec(),
            stderr: b"".to_vec(),
            killed: false,
            kill_reason: None,
        };
        assert_eq!(result.exit_code, 0);
        assert!(!result.killed);
    }

    #[test]
    fn test_execution_result_killed() {
        let result = ExecutionResult {
            exit_code: -1,
            stdout: b"".to_vec(),
            stderr: b"timeout".to_vec(),
            killed: true,
            kill_reason: Some("Timeout".to_string()),
        };
        assert!(result.killed);
        assert_eq!(result.kill_reason, Some("Timeout".to_string()));
    }

    #[test]
    fn test_sandbox_trait() {
        let sandbox = Sandbox::new();
        let result = SandboxTrait::execute(&sandbox, b"test").unwrap();
        assert_eq!(result.exit_code, 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_seccomp_initialization() {
        let mut config = SandboxConfig::default();
        config.seccomp_enabled = true;
        let mut sandbox = Sandbox::with_config(config);
        
        // Initialize seccomp
        let result = sandbox.init_seccomp();
        assert!(result.is_ok());
        
        // Check if seccomp is initialized
        assert!(sandbox.is_seccomp_initialized());
    }

    #[test]
    fn test_resource_limit_memory() {
        let mut config = SandboxConfig::default();
        config.max_memory = 100;
        
        // Create a sandbox with memory usage exceeding the limit
        let sandbox = Sandbox {
            active: true,
            config,
            memory_usage: 200, // Exceed the limit
            total_time: 0,
            #[cfg(target_os = "linux")]
            seccomp_filter: None,
        };
        
        let result = sandbox.execute(b"test");
        assert!(result.is_ok());
        let execution_result = result.unwrap();
        assert!(execution_result.killed);
        assert_eq!(execution_result.kill_reason, Some("Memory limit exceeded".to_string()));
    }
}

//! Core runtime for MathCore kernel

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, info, warn};

use crate::bus::{Bus, BusConfig};
use crate::error::{ErrorKind, MathCoreError, ResourceType, Result};
use crate::sandbox::Sandbox;

#[derive(Debug, Clone)]
pub struct KernelConfig {
    pub max_plugin_memory: u64,
    pub max_execution_time: u64,
    pub max_plugins: usize,
    pub sandbox_enabled: bool,
    pub shutdown_timeout: Duration,
    pub tick_interval: Duration,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            max_plugin_memory: 256 * 1024 * 1024,
            max_execution_time: 30_000,
            max_plugins: 64,
            sandbox_enabled: true,
            shutdown_timeout: Duration::from_secs(10),
            tick_interval: Duration::from_millis(100),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Loading,
    Loaded,
    Running,
    Stopped,
    Failed,
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub metadata: PluginMetadata,
    pub state: PluginState,
    pub loaded_at: Instant,
    pub sandbox: Option<Sandbox>,
}

#[derive(Debug, Default)]
pub struct ResourceUsage {
    pub memory_used: u64,
    pub cpu_time_used: u64,
    pub executions: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelState {
    Created,
    Running,
    ShuttingDown,
    Stopped,
}

pub struct Kernel {
    config: KernelConfig,
    state: RwLock<KernelState>,
    bus: RwLock<Option<Arc<Bus>>>,
    plugins: RwLock<HashMap<String, PluginInfo>>,
    resource_quotas: RwLock<HashMap<String, ResourceQuota>>,
    shutdown_tx: RwLock<Option<mpsc::Sender<ShutdownSignal>>>,
    running: AtomicBool,
    stats: KernelStats,
}

#[derive(Debug, Default)]
pub struct KernelStats {
    pub uptime_seconds: AtomicU64,
    pub total_executions: AtomicU64,
    pub failed_executions: AtomicU64,
    pub messages_processed: AtomicU64,
}

impl Kernel {
    pub fn new() -> Self {
        Self::with_config(KernelConfig::default())
    }

    pub fn with_config(config: KernelConfig) -> Self {
        Self {
            config,
            state: RwLock::new(KernelState::Created),
            bus: RwLock::new(None),
            plugins: RwLock::new(HashMap::new()),
            resource_quotas: RwLock::new(HashMap::new()),
            shutdown_tx: RwLock::new(None),
            running: AtomicBool::new(false),
            stats: KernelStats::default(),
        }
    }

    pub async fn init(&self) -> Result<()> {
        if self.bus.read().is_some() {
            return Err(MathCoreError::new(ErrorKind::InvalidState(
                "Kernel already initialized".into(),
            )));
        }

        info!("Initializing MathCore kernel");
        let bus = Arc::new(Bus::new(BusConfig::default()));
        *self.bus.write() = Some(bus);
        info!("Kernel initialized successfully");
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(MathCoreError::new(ErrorKind::CoreAlreadyRunning));
        }

        let state = *self.state.read();
        if state != KernelState::Created && state != KernelState::Stopped {
            return Err(MathCoreError::new(ErrorKind::InvalidState(
                "Kernel cannot run from current state".into(),
            )));
        }

        *self.state.write() = KernelState::Running;
        info!("Starting MathCore kernel event loop");

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<ShutdownSignal>(16);
        *self.shutdown_tx.write() = Some(shutdown_tx);

        let config = self.config.clone();
        let running = Arc::new(AtomicBool::new(true));

        let tick_interval = interval(config.tick_interval);
        tokio::pin!(tick_interval);

        loop {
            tokio::select! {
                _ = tick_interval.tick() => {
                    if !running.load(Ordering::SeqCst) {
                        break;
                    }
                    self.process_tick().await;
                }
                _ = shutdown_rx.recv() => {
                    debug!("Shutdown signal received");
                    break;
                }
            }
        }

        *self.state.write() = KernelState::Stopped;
        self.running.store(false, Ordering::SeqCst);
        info!("Kernel event loop stopped");

        Ok(())
    }

    async fn process_tick(&self) {
        self.stats.uptime_seconds.fetch_add(1, Ordering::Relaxed);
        self.check_plugin_resources();
    }

    fn check_plugin_resources(&self) {
        let plugins = self.plugins.read();
        let quotas = self.resource_quotas.read();

        for (id, plugin) in plugins.iter() {
            if let Some(quota) = quotas.get(id) {
                if let Some(sandbox) = &plugin.sandbox {
                    let memory = sandbox.get_memory_usage();
                    if memory > quota.max_memory {
                        warn!(
                            "Plugin {} exceeded memory quota: {} > {}",
                            id, memory, quota.max_memory
                        );
                    }
                }
            }
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        let state = *self.state.read();
        if state != KernelState::Running {
            return Err(MathCoreError::new(ErrorKind::CoreNotRunning));
        }

        info!("Initiating kernel shutdown");
        *self.state.write() = KernelState::ShuttingDown;

        if let Some(tx) = self.shutdown_tx.read().as_ref() {
            let _ = tx.send(ShutdownSignal).await;
        }

        tokio::time::sleep(self.config.shutdown_timeout).await;
        self.unload_all_plugins().await?;

        if let Some(bus) = self.bus.write().take() {
            drop(bus);
        }

        *self.state.write() = KernelState::Stopped;
        info!("Kernel shutdown complete");
        Ok(())
    }

    pub async fn load_plugin(&self, metadata: PluginMetadata, sandbox_enabled: bool) -> Result<()> {
        let state = *self.state.read();
        if state != KernelState::Running && state != KernelState::Created {
            return Err(MathCoreError::new(ErrorKind::InvalidState(
                "Cannot load plugin in current state".into(),
            )));
        }

        let id = metadata.id.clone();

        if self.plugins.read().contains_key(&id) {
            return Err(MathCoreError::new(ErrorKind::PluginAlreadyLoaded(id)));
        }

        if self.plugins.read().len() >= self.config.max_plugins {
            return Err(MathCoreError::new(ErrorKind::ResourceQuotaExceeded(
                "Maximum plugins reached".into(),
            )));
        }

        info!("Loading plugin: {} v{}", id, metadata.version);

        let sandbox = if sandbox_enabled || self.config.sandbox_enabled {
            Some(Sandbox::new())
        } else {
            None
        };

        let info = PluginInfo {
            metadata,
            state: PluginState::Loading,
            loaded_at: Instant::now(),
            sandbox,
        };

        self.plugins.write().insert(id.clone(), info);

        self.resource_quotas.write().insert(
            id.clone(),
            ResourceQuota {
                max_memory: self.config.max_plugin_memory,
                max_cpu_time: self.config.max_execution_time,
                max_executions: u64::MAX,
            },
        );

        if let Some(plugin) = self.plugins.write().get_mut(&id) {
            plugin.state = PluginState::Loaded;
        }

        info!("Plugin loaded: {}", id);
        Ok(())
    }

    pub async fn unload_plugin(&self, id: &str) -> Result<()> {
        let mut plugins = self.plugins.write();

        let plugin = plugins
            .get_mut(id)
            .ok_or_else(|| MathCoreError::new(ErrorKind::PluginNotFound(id.into())))?;

        info!("Unloading plugin: {}", id);

        if plugin.state == PluginState::Running {
            plugin.state = PluginState::Stopped;
        }

        plugins.remove(id);
        drop(plugins);

        self.resource_quotas.write().remove(id);

        info!("Plugin unloaded: {}", id);
        Ok(())
    }

    async fn unload_all_plugins(&self) -> Result<()> {
        let ids: Vec<String> = self.plugins.read().keys().cloned().collect();

        for id in ids {
            if let Err(e) = self.unload_plugin(&id).await {
                warn!("Failed to unload plugin {}: {}", id, e);
            }
        }
        Ok(())
    }

    pub fn get_plugin(&self, id: &str) -> Option<PluginInfo> {
        self.plugins.read().get(id).cloned()
    }

    pub fn list_plugins(&self) -> Vec<(String, PluginMetadata)> {
        self.plugins
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.metadata.clone()))
            .collect()
    }

    pub fn set_resource_quota(&self, plugin_id: &str, quota: ResourceQuota) -> Result<()> {
        if !self.plugins.read().contains_key(plugin_id) {
            return Err(MathCoreError::new(ErrorKind::PluginNotFound(
                plugin_id.into(),
            )));
        }
        self.resource_quotas
            .write()
            .insert(plugin_id.to_string(), quota);
        Ok(())
    }

    pub fn get_resource_quota(&self, plugin_id: &str) -> Option<ResourceQuota> {
        self.resource_quotas.read().get(plugin_id).cloned()
    }

    pub fn stats(&self) -> KernelStatsSnapshot {
        KernelStatsSnapshot {
            uptime_seconds: self.stats.uptime_seconds.load(Ordering::Relaxed),
            total_executions: self.stats.total_executions.load(Ordering::Relaxed),
            failed_executions: self.stats.failed_executions.load(Ordering::Relaxed),
            messages_processed: self.stats.messages_processed.load(Ordering::Relaxed),
            plugins_loaded: self.plugins.read().len(),
        }
    }

    pub fn state(&self) -> KernelState {
        *self.state.read()
    }

    pub fn config(&self) -> &KernelConfig {
        &self.config
    }

    pub fn bus(&self) -> Option<Arc<Bus>> {
        self.bus.read().clone()
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        if *self.state.read() == KernelState::Running {
            warn!("Kernel dropped while still running");
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceQuota {
    pub max_memory: u64,
    pub max_cpu_time: u64,
    pub max_executions: u64,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        Self {
            max_memory: 256 * 1024 * 1024,
            max_cpu_time: 60_000,
            max_executions: 1000,
        }
    }
}

impl ResourceQuota {
    pub fn is_exceeded(&self, usage: &ResourceUsage) -> Option<ResourceType> {
        if usage.memory_used > self.max_memory {
            return Some(ResourceType::Memory);
        }
        if usage.cpu_time_used > self.max_cpu_time {
            return Some(ResourceType::Cpu);
        }
        None
    }
}

#[derive(Debug)]
pub struct ShutdownSignal;

#[derive(Debug, Clone)]
pub struct KernelStatsSnapshot {
    pub uptime_seconds: u64,
    pub total_executions: u64,
    pub failed_executions: u64,
    pub messages_processed: u64,
    pub plugins_loaded: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kernel_init() {
        let kernel = Kernel::new();
        kernel.init().await.unwrap();
        assert_eq!(kernel.state(), KernelState::Created);
    }

    #[tokio::test]
    async fn test_kernel_init_twice() {
        let kernel = Kernel::new();
        kernel.init().await.unwrap();
        let result = kernel.init().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kernel_with_config() {
        let config = KernelConfig {
            max_plugin_memory: 512 * 1024 * 1024,
            max_execution_time: 60_000,
            max_plugins: 32,
            sandbox_enabled: false,
            shutdown_timeout: Duration::from_secs(5),
            tick_interval: Duration::from_millis(50),
        };
        let kernel = Kernel::with_config(config.clone());
        kernel.init().await.unwrap();
        assert_eq!(kernel.config().max_plugin_memory, 512 * 1024 * 1024);
        assert_eq!(kernel.config().max_plugins, 32);
    }

    #[tokio::test]
    async fn test_plugin_load_unload() {
        let kernel = Kernel::new();
        kernel.init().await.unwrap();

        let metadata = PluginMetadata {
            id: "test_plugin".into(),
            version: "1.0.0".into(),
            name: "Test Plugin".into(),
            description: Some("A test plugin".into()),
        };

        kernel.load_plugin(metadata.clone(), false).await.unwrap();

        let plugins = kernel.list_plugins();
        assert_eq!(plugins.len(), 1);

        kernel.unload_plugin("test_plugin").await.unwrap();

        let plugins = kernel.list_plugins();
        assert_eq!(plugins.len(), 0);
    }

    #[tokio::test]
    async fn test_plugin_load_with_sandbox() {
        let kernel = Kernel::new();
        kernel.init().await.unwrap();

        let metadata = PluginMetadata {
            id: "sandboxed_plugin".into(),
            version: "1.0.0".into(),
            name: "Sandboxed Plugin".into(),
            description: None,
        };

        kernel.load_plugin(metadata.clone(), true).await.unwrap();

        let plugin = kernel.get_plugin("sandboxed_plugin");
        assert!(plugin.is_some());
        assert!(plugin.unwrap().sandbox.is_some());
    }

    #[tokio::test]
    async fn test_duplicate_plugin_load() {
        let kernel = Kernel::new();
        kernel.init().await.unwrap();

        let metadata = PluginMetadata {
            id: "duplicate_test".into(),
            version: "1.0.0".into(),
            name: "Duplicate Test".into(),
            description: None,
        };

        kernel.load_plugin(metadata.clone(), false).await.unwrap();
        let result = kernel.load_plugin(metadata.clone(), false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_max_plugins_limit() {
        let config = KernelConfig {
            max_plugins: 2,
            ..Default::default()
        };
        let kernel = Kernel::with_config(config);
        kernel.init().await.unwrap();

        let metadata1 = PluginMetadata {
            id: "plugin1".into(),
            version: "1.0.0".into(),
            name: "Plugin 1".into(),
            description: None,
        };
        let metadata2 = PluginMetadata {
            id: "plugin2".into(),
            version: "1.0.0".into(),
            name: "Plugin 2".into(),
            description: None,
        };
        let metadata3 = PluginMetadata {
            id: "plugin3".into(),
            version: "1.0.0".into(),
            name: "Plugin 3".into(),
            description: None,
        };

        kernel.load_plugin(metadata1, false).await.unwrap();
        kernel.load_plugin(metadata2, false).await.unwrap();
        let result = kernel.load_plugin(metadata3, false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unload_nonexistent_plugin() {
        let kernel = Kernel::new();
        kernel.init().await.unwrap();

        let result = kernel.unload_plugin("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_plugin() {
        let kernel = Kernel::new();
        kernel.init().await.unwrap();

        let metadata = PluginMetadata {
            id: "get_test".into(),
            version: "2.0.0".into(),
            name: "Get Test".into(),
            description: Some("Test get_plugin".into()),
        };

        kernel.load_plugin(metadata.clone(), false).await.unwrap();

        let plugin = kernel.get_plugin("get_test");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().metadata.version, "2.0.0");

        let nonexistent = kernel.get_plugin("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[tokio::test]
    async fn test_set_resource_quota() {
        let kernel = Kernel::new();
        kernel.init().await.unwrap();

        let metadata = PluginMetadata {
            id: "quota_test".into(),
            version: "1.0.0".into(),
            name: "Quota Test".into(),
            description: None,
        };

        kernel.load_plugin(metadata, false).await.unwrap();

        let quota = ResourceQuota {
            max_memory: 100 * 1024 * 1024,
            max_cpu_time: 30_000,
            max_executions: 500,
        };

        kernel.set_resource_quota("quota_test", quota).unwrap();

        let retrieved = kernel.get_resource_quota("quota_test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().max_memory, 100 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_set_quota_nonexistent_plugin() {
        let kernel = Kernel::new();
        kernel.init().await.unwrap();

        let quota = ResourceQuota::default();
        let result = kernel.set_resource_quota("nonexistent", quota);
        assert!(result.is_err());
    }

    #[test]
    fn test_resource_quota_exceeded() {
        let quota = ResourceQuota {
            max_memory: 100,
            max_cpu_time: 50,
            max_executions: 10,
        };

        let usage_exceeded = ResourceUsage {
            memory_used: 150,
            cpu_time_used: 30,
            executions: 5,
        };
        assert_eq!(quota.is_exceeded(&usage_exceeded), Some(ResourceType::Memory));

        let cpu_exceeded = ResourceUsage {
            memory_used: 50,
            cpu_time_used: 100,
            executions: 5,
        };
        assert_eq!(quota.is_exceeded(&cpu_exceeded), Some(ResourceType::Cpu));

        let within_quota = ResourceUsage {
            memory_used: 50,
            cpu_time_used: 30,
            executions: 5,
        };
        assert_eq!(quota.is_exceeded(&within_quota), None);
    }

    #[tokio::test]
    async fn test_kernel_stats() {
        let kernel = Kernel::new();
        kernel.init().await.unwrap();

        let stats = kernel.stats();
        assert_eq!(stats.plugins_loaded, 0);
    }

    #[test]
    fn test_kernel_config_default() {
        let config = KernelConfig::default();
        assert_eq!(config.max_plugin_memory, 256 * 1024 * 1024);
        assert_eq!(config.max_execution_time, 30_000);
        assert_eq!(config.max_plugins, 64);
        assert!(config.sandbox_enabled);
    }

    #[test]
    fn test_plugin_metadata() {
        let meta = PluginMetadata {
            id: "test".into(),
            version: "1.0.0".into(),
            name: "Test".into(),
            description: Some("Test plugin".into()),
        };
        assert_eq!(meta.id, "test");
        assert_eq!(meta.version, "1.0.0");
    }

    #[test]
    fn test_plugin_states() {
        assert_eq!(PluginState::Loading, PluginState::Loading);
        assert_eq!(PluginState::Loaded, PluginState::Loaded);
        assert_eq!(PluginState::Running, PluginState::Running);
        assert_eq!(PluginState::Stopped, PluginState::Stopped);
        assert_eq!(PluginState::Failed, PluginState::Failed);
    }

    #[test]
    fn test_kernel_states() {
        assert_eq!(KernelState::Created, KernelState::Created);
        assert_eq!(KernelState::Running, KernelState::Running);
        assert_eq!(KernelState::ShuttingDown, KernelState::ShuttingDown);
        assert_eq!(KernelState::Stopped, KernelState::Stopped);
    }
}

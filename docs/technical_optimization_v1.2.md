# MathCore v6.0 技术方案优化版 (v1.2)

> **版本**: 1.2  
> **基于**: MathCore v6.0 完整技术设计方案 + v1.1优化  
> **优化日期**: 2026-03-04  
> **状态**: 待用户审批

---

## 1. 深度反思与优化

### 1.1 原方案核心优势（保留）

| 优势 | 说明 |
|------|------|
| 微内核架构 | 插件隔离，热更新能力 |
| 分层序列化 | MessagePack/Arrow/FlatBuffers各司其职 |
| 五重验证体系 | 从语法到形式化的完整链路 |
| MathCore主导 | 大模型仅作为接口层 |

### 1.2 发现的问题

| # | 问题 | 影响 | 优先级 |
|---|------|------|--------|
| 1 | IPC技术选型待定 | 性能差异大 | P0 |
| 2 | 验证层级命名不一致 | 与原方案冲突 | P1 |
| 3 | 风险评估不全面 | 可能踩坑 | P0 |
| 4 | 缺少监控/可观测性 | 运维困难 | P1 |
| 5 | 热更新策略模糊 | 可用性风险 | P1 |
| 6 | 版本号不准确 | 依赖管理问题 | P2 |

### 1.3 针对性优化

#### 优化1: IPC技术选型

**决策**: 使用 **NNG (nanomsg-next-gen)** + **tokio-uring**

**理由**:
- NNG: 比ZeroMQ更轻量，async原生支持
- tokio-uring: Linux下io_uring支持，超低延迟
- Unix Domain Socket: 简单场景首选，<1μs

**混合策略**:
```rust
#[derive(Clone)]
pub enum IpcBackend {
    /// 同一主机，优先使用
    Uds(UnixDomainSocket),
    /// 跨主机/低延迟
    Nng(nng::Socket),
    /// 高吞吐场景
    #[cfg(feature = "io-uring")]
    Uring(tokio_uring::net::TcpSocket),
}
```

#### 优化2: 验证层级对齐

| 原方案层级 | 本方案 | 说明 |
|-----------|--------|------|
| L0 | NanoCheck | 语法/规范形 |
| L1 | SMT Solver | 等式/不等式 |
| L2 | MetaMath | 公理化 |
| L3 | Lean 4 | 依赖类型 |

#### 优化3: 风险评估扩展

| 新增风险 | 缓解措施 |
|---------|---------|
| Wolfram许可证限制 | 仅作可选后端，默认SymPy |
| Symbolica商业许可 | 评估meval作备选 |
| 16周工期可能不足 | 预留2周缓冲 |
| 外部依赖构建失败 | 容器化构建环境 |

#### 优化4: 可观测性增强

```rust
/// Prometheus 指标定义
pub mod metrics {
    // 计算延迟分桶
    pub const COMPUTE_LATENCY: Histogram = Histogram::new(
        "mathcore_compute_latency_ms",
        "Compute latency in milliseconds",
        vec![0.1, 1.0, 10.0, 100.0, 1000.0]
    );
    
    // 缓存命中率
    pub const CACHE_HIT_RATE = Gauge::new(
        "mathcore_cache_hit_rate",
        "Cache hit rate (0-1)"
    );
    
    // 验证失败（红线）
    pub const VALIDATION_FAILURES = Counter::new(
        "mathcore_validation_failures_total",
        "Total validation failures"
    );
}
```

#### 优化5: 热更新策略

```rust
/// 插件热更新状态机
pub enum PluginState {
    Loading,
    Ready(PluginHandle),
    Updating { old: PluginHandle, new: PluginHandle },
    Unloading,
    Error(Error),
}

/// 热更新协议
trait HotReload {
    /// 检查更新
    fn check_update(&self) -> Result<Option<Version>>;
    /// 准备更新（预加载）
    fn prepare_update(&self, new_version: Version) -> Result<()>;
    /// 原子切换
    fn switch(&self) -> Result<()>;
    /// 回滚
    fn rollback(&self) -> Result<()>;
}
```

---

## 2. 依赖库版本校正

| 功能 | 原选型 | 校正版本 | 备注 |
|------|--------|----------|------|
| MessagePack | rmp_serde 1.0+ | rmp-serde 1.3+ | 确认包名 |
| GPU渲染 | wgpu 0.18+ | wgpu 0.19+ | 当前稳定版 |
| IPC | nng | nng 1.8+ | 确认版本 |
| 符号计算 | symengine | symbolica 1.3+ | 包名校正 |
| Arrow | arrow 45+ | arrow 45.0+ | 确认版本号 |

---

## 3. 任务时间缓冲

### 3.1 调整后的里程碑

| 阶段 | 原周期 | 调整后 | 缓冲 |
|------|--------|--------|------|
| Phase 1 | Week 1-4 | Week 1-5 | +1周 |
| Phase 2 | Week 5-8 | Week 6-9 | +1周 |
| Phase 3 | Week 9-12 | Week 10-13 | +1周 |
| Phase 4 | Week 13-16 | Week 14-16 | 0周 |

**总周期**: 16周 (不变，预留缓冲重新分配)

### 3.2 高风险任务额外缓冲

| 任务 | 风险 | 额外缓冲 |
|------|------|---------|
| T14 Lean Bridge | 高 | +3天 |
| T17 MCP Bridge | 中 | +2天 |
| T18 计算回放 | 中 | +2天 |

---

## 4. 新增关键组件

### 4.1 配置管理系统

```rust
/// 配置热更新
pub struct ConfigManager {
    watcher: notify::RecommendedWatcher,
    curriculum: Curriculum,
    extensions: ExtensionConfig,
}

/// 支持运行时热切换
impl ConfigManager {
    pub fn reload_curriculum(&mut self, name: &str) -> Result<()> {
        let new = self.load_curriculum(name)?;
        *self.curriculum = new;
        // 广播配置变更事件
        self.event_bus.broadcast(ConfigChanged);
        Ok(())
    }
}
```

### 4.2 统一错误处理

```rust
/// 错误分类
pub enum MathCoreError {
    /// 解析错误
    Parse(ParseError),
    /// 计算错误
    Compute(ComputeError),
    /// 验证失败
    Verification(VerificationError),
    /// 插件错误
    Plugin(PluginError),
    /// 超时
    Timeout(Duration),
    /// 超出能力
    BeyondCapability,
}

impl MathCoreError {
    pub fn code(&self) -> ErrorCode;
    pub fn retryable(&self) -> bool;
}
```

### 4.3 安全沙箱（扩展）

```rust
/// Seccomp 系统调用白名单
const SYSCALL_WHITELIST: &[u32] = &[
    // 文件操作
    sys_read, sys_write, sys_openat, sys_close,
    // 内存
    sys_mmap, sys_mprotect, sys_brk,
    // 进程
    sys_exit, sys_exit_group,
    // 时间
    sys_clock_gettime, sys_gettimeofday,
];
```

---

## 5. 架构决策记录 (ADR)

### ADR-001: 使用NNG作为默认IPC

**状态**: 拟议  
**决策者**: 待定  
**影响**: 高

### ADR-002: Lean作为可选验证层

**状态**: 拟议  
**决策者**: 待定  
**影响**: 中

### ADR-003: 监控优先于日志

**状态**: 拟议  
**决策者**: 待定  
**影响**: 中

---

## 6. 验收标准增强

### 6.1 新增验收点

| 阶段 | 新增验收 | 优先级 |
|------|---------|--------|
| Phase 1 | 错误处理单元测试 > 90% | P1 |
| Phase 2 | 监控指标暴露正常 | P1 |
| Phase 3 | 配置热切换测试通过 | P1 |
| Phase 4 | 安全审计通过 | P0 |

### 6.2 性能基线

```yaml
性能基线:
  L0计算:
    - 目标: <10ms
    - 基线: <5ms (预留50%余量)
  
  消息序列化:
    - 目标: <1ms
    - 基线: <500μs
  
  10MB矩阵传输:
    - 目标: <10ms
    - 基线: <5ms
```

---

## 7. 下一步行动

1. [ ] 审批v1.2优化方案
2. [ ] 确认ADR-001/002/003
3. [ ] 更新Phase任务时间表
4. [ ] 开始Phase 1 - T1

---

## 8. 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.0 | - | 原始方案 |
| 1.1 | 2026-03-04 | 初始优化 |
| 1.2 | 2026-03-04 | 深度反思版 |

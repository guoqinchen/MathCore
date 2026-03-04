# MathCore v6.0 技术方案优化版

> **版本**: 1.1  
> **基于**: MathCore v6.0 完整技术设计方案  
> **优化日期**: 2026-03-04

---

## 1. 架构优化建议

### 1.1 核心模块拆分

| 原方案模块 | 优化建议 | 理由 |
|-----------|---------|------|
| MathCore Kernel | 拆分为 kernel-core + kernel-bus | 职责分离，提高可维护性 |
| ComputeExt | 拆分为 compute-local + compute-external | 本地/外部计算隔离 |
| BridgeExt | MCP Bridge 独立 | 便于单独测试和版本管理 |

### 1.2 依赖库选择

| 功能 | 推荐库 | 备选 | 理由 |
|------|--------|------|------|
| MessagePack | rmp_serde 1.0+ | none | 成熟稳定，async支持 |
| IPC | nng (nanomsg-ng) | tokio + UDS | 更轻量，async原生 |
| GPU | wgpu 0.18+ | none | 跨平台事实标准 |
| Arrow | arrow 45+ | none | 完整生态 |
| 符号计算 | symengine | meval | 性能优先 |
| 验证 | z3 + rustfst | smt_solver | 成熟SMT |

---

## 2. 关键设计决策

### 2.1 协议分层

```
┌─────────────────────────────────────────────┐
│  MCP Bridge (JSON-RPC over stdio/unix-socket) │
├─────────────────────────────────────────────┤
│  MathCore Protocol (MessagePack)             │
├─────────────────────────────────────────────┤
│  Zero-Copy Data Plane (Arrow/FlatBuffers)   │
└─────────────────────────────────────────────┘
```

### 2.2 插件接口

```rust
/// 核心插件 trait
trait MathCorePlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&mut self, ctx: &PluginContext) -> Result<()>;
    fn execute(&self, req: &ComputeRequest) -> Result<ComputeResponse>;
    fn shutdown(&mut self) -> Result<()>;
}
```

### 2.3 验证策略

- **L0 (高中)**: 本地Rust计算 + 数值采样验证
- **L1 (大学)**: SMT求解器(Z3) + 符号验证
- **L2 (研究)**: 形式化证明(Lean)集成
- **L3 (严格)**: 依赖类型理论

---

## 3. 性能目标细化

| 场景 | 指标 | 实现方案 |
|------|------|---------|
| L0计算 | <10ms | Rust本地计算，无IPC |
| 消息序列化 | <1ms | MessagePack直接编解码 |
| 矩阵传输 | <10ms | DMA-Buf + 共享内存 |
| 3D渲染 | 60fps | wgpu增量渲染 |
| 插件热更新 | <100ms | 动态so加载 |

---

## 4. 安全隔离方案

### 4.1 进程隔离级别

```
Level 0: 同一进程，线程隔离 (本地Rust计算)
Level 1: 同一进程，async task隔离 (外部引擎)
Level 2: 子进程，cgroup隔离 (Wolfram/SymPy)
Level 3: 容器隔离 (Docker, 可选)
```

### 4.2 资源配额

| 资源 | 限制 | 触发动作 |
|------|------|---------|
| 内存 | 2GB/plugin | OOM Kill + 重启 |
| CPU | 30s/任务 | Timeout |
| 网络 | 禁用(默认) | Seccomp过滤 |
| 磁盘 | /tmp/* | 临时文件限制 |

---

## 5. 实施检查点

### Phase 1 交付物
- [ ] kernel-core < 5000行
- [ ] MessagePack协议v6稳定
- [ ] ComputeExt (Rust本地计算)
- [ ] 基本单元测试 > 80%覆盖率

### Phase 2 交付物
- [ ] VizEngine wgpu集成
- [ ] DMA-Buf零拷贝
- [ ] Apache Arrow集成
- [ ] 10MB矩阵传输<10ms

### Phase 3 交付物
- [ ] NanoCheck L0验证
- [ ] Lean Bridge L3集成
- [ ] Verification Mesh
- [ ] 验证失败率<0.01%

### Phase 4 交付物
- [ ] pip install mathcore
- [ ] MCP Bridge稳定
- [ ] 计算回放GUI
- [ ] 安装<5分钟

---

## 6. 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| Wolfram集成困难 | 中 | 高 | 优先SymPy，降级方案 |
| GPU跨平台兼容性 | 中 | 中 | wgpu抽象，三后端适配 |
| Lean集成复杂度 | 高 | 中 | L3作为可选，非强制 |
| 性能目标未达成 | 中 | 高 | 持续性能测试，原型验证 |

---

## 7. 技术栈总结

```yaml
核心语言: Rust (stable)
构建工具: Cargo + cargo-binstall
测试:    tokio-test + proptest
文档:    mdbook
发布:    cargo-release + pip

关键依赖:
  - rmp_serde: MessagePack
  - nng: IPC
  - wgpu: GPU渲染
  - arrow: 数据平面
  - symengine: 符号计算
  - z3-solver: SMT
  - tokio: 异步运行时
  - serde: 序列化框架
```

---

## 8. 下一步行动

1. 创建项目骨架 (Cargo.toml, 目录结构)
2. 实现 Phase 1 内核原型
3. 搭建CI/CD流水线
4. 编写核心模块单元测试

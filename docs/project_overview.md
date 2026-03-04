# MathCore v6.0 项目执行总览

> **版本**: 1.0  
> **创建日期**: 2026-03-04  
> **总周期**: 16周 (4个月)

---

## 1. 项目概览

### 1.1 核心理念

**MathCore** 是面向高中至大学研究级数学的本地高性能计算核心。采用微内核+插件架构，将大模型定位为自然语言接口层，核心计算完全本地自治，确保数学严格性与零延迟响应。

### 1.2 关键考核指标

| 指标 | 目标值 |
| ---- | ------ |
| 可用性 | 99.99% (年度停机<52分钟) |
| L0计算 | <10ms |
| L2计算 | <1s |
| 10MB矩阵传输 | <10ms |
| 验证失败率 | <0.01% |
| 安装时间 | <5分钟 |

---

## 2. 技术架构

### 2.1 分层架构

```
┌─────────────────────────────────────────┐
│        用户界面层 (Presentation)          │
│  TUI │ GUI │ Jupyter │ VS Code │ MCP   │
├─────────────────────────────────────────┤
│      MathCore Bus (ZeroMQ/nanomsg)      │
├─────────────────────────────────────────┤
│        MathCore Kernel (微内核)          │
│    进程隔离 │ 资源配额 │ 插件生命周期   │
├─────────────────────────────────────────┤
│           插件域 (Extensions)            │
│  ComputeExt │ RenderExt │ KnowledgeExt   │
├─────────────────────────────────────────┤
│        Zero-Copy Data Plane             │
│  Shared Memory │ GPU DMA-Buf │ Arrow    │
└─────────────────────────────────────────┘
```

### 2.2 技术栈

| 层级 | 技术选择 |
|------|----------|
| 核心语言 | Rust (stable) |
| 消息协议 | MessagePack (rmp-serde 1.3+) |
| IPC | Unix Domain Sockets + NNG |
| GPU渲染 | wgpu 28+ |
| 数据平面 | Apache Arrow |
| 符号计算 | Symbolica 1.3+ |
| SMT验证 | Z3 0.19+ |
| 形式化 | Lean 4 (Leo3) |
| Python绑定 | PyO3 |

---

## 3. 实施阶段

### Phase 1: 内核与MessagePack (Week 1-4)

**目标**: 微内核稳定，MessagePack协议定型

| 任务 | 内容 | 交付物 |
|------|------|--------|
| T1 | 项目骨架搭建 | workspace, CI/CD |
| T2 | 微内核核心 | kernel-core + kernel-bus |
| T3 | MessagePack协议 | MathMessage v6 |
| T4 | 计算扩展 | ComputeExt (Rust) |
| T5 | CLI接口 | 命令行工具 |
| T6 | 单元测试 | >80%覆盖率 |

### Phase 2: 性能与GPU (Week 5-8)

**目标**: 零拷贝数据传输，GPU渲染

| 任务 | 内容 | 交付物 |
|------|------|--------|
| T7 | VizEngine | wgpu渲染引擎 |
| T8 | 零拷贝数据 | Arrow + DMA-Buf |
| T9 | 实时流协议 | FlatBuffers |
| T10 | 性能优化 | SIMD + 缓存 |

### Phase 3: 严格性与验证 (Week 9-12)

**目标**: 分层验证，形式化证明

| 任务 | 内容 | 交付物 |
|------|------|--------|
| T11 | NanoCheck | L0语法验证 |
| T12 | SMT集成 | Z3求解器 |
| T13 | Verification Mesh | 三级验证 |
| T14 | Lean Bridge | L3形式化 |
| T15 | 符号系统 | Unicode支持 |

### Phase 4: 生态与分发 (Week 13-16)

**目标**: 易安装，完整文档

| 任务 | 内容 | 交付物 |
|------|------|--------|
| T16 | Python包 | pip install |
| T17 | MCP Bridge | 协议集成 |
| T18 | 计算回放 | 调试GUI |
| T19 | 完整文档 | API + 教程 |
| T20 | 生态分发 | 多平台构建 |

---

## 4. 任务依赖图

```
Phase 1 (Week 1-4)
├── T1 (骨架)
│     ├── T2 (内核) → T4 (计算) → T6 (测试)
│     └── T3 (协议) → T5 (CLI) → T6 (测试)
│
Phase 2 (Week 5-8)
├── T7 (VizEngine) → T8 (零拷贝) → T9 (流) → T10 (优化)
│
Phase 3 (Week 9-12)
├── T11 (NanoCheck) → T12 (SMT) → T13 (Mesh) → T14 (Lean)
│
Phase 4 (Week 13-16)
├── T16 (Python) → T17 (MCP)
├── T18 (回放)
├── T19 (文档)
└── T20 (分发)
```

---

## 5. 角色分配

| 角色 | 主要任务 |
|------|----------|
| Hephaestus (深度工程兵) | T1-T5, T7-T10, T11-T15, T16-T18, T20 |
| Scribe (文书兵) | T19 |
| Reviewer (审查兵) | 代码审查 |

---

## 6. 验收里程碑

| 周次 | 检查点 |
|------|--------|
| Week 1 | T1完成，内核开始 |
| Week 2 | T2+T3完成，计算开始 |
| Week 3 | T4+T5完成，测试开始 |
| Week 4 | T6完成，Phase 1交付 |
| Week 5 | T7开始，wgpu集成 |
| Week 6 | T7完成，T8开始 |
| Week 7 | T8完成，T9完成 |
| Week 8 | T10完成，Phase 2交付 |
| Week 9 | T11开始，验证开始 |
| Week 10 | T11+T12完成，T13开始 |
| Week 11 | T13完成，T14开始 |
| Week 12 | T14+T15完成，Phase 3交付 |
| Week 13 | T16开始，Python包 |
| Week 14 | T16+T17完成，T18开始 |
| Week 15 | T18+T19完成，T20开始 |
| Week 16 | T20完成，**项目交付** |

---

## 7. 输出文件清单

| 文件 | 描述 |
|------|------|
| `docs/technical_optimization.md` | 技术方案优化 |
| `docs/phase1_tasks.md` | Phase 1任务清单 |
| `docs/phase2_tasks.md` | Phase 2任务清单 |
| `docs/phase3_tasks.md` | Phase 3任务清单 |
| `docs/phase4_tasks.md` | Phase 4任务清单 |
| `opencode_init.md` | Sisyphus初始化 |

---

## 8. 风险评估

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| Wolfram集成困难 | 中 | 高 | 优先SymPy |
| GPU跨平台兼容 | 中 | 中 | wgpu抽象 |
| Lean集成复杂度 | 高 | 中 | L3可选 |
| 性能目标未达成 | 中 | 高 | 持续测试 |

---

## 9. 成功标准

- [ ] 所有20个任务完成
- [ ] 核心指标达成
- [ ] 文档完整
- [ ] 测试通过
- [ ] 可安装运行

---

**项目启动**: 立即开始 Phase 1 - T1 (项目骨架搭建)

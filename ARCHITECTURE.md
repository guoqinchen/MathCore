# MathCore 项目架构文档

> **项目**: MathCore v6.0  
> **版本**: 1.0  
> **日期**: 2026-03-04

---

## 1. 项目概述

### 1.1 核心理念

MathCore 是面向高中至大学研究级数学的本地高性能计算核心。采用微内核+插件架构，将大模型定位为自然语言接口层，核心计算完全本地自治，确保数学严格性与零延迟响应。

### 1.2 关键目标

| 指标 | 目标值 |
| ---- | ------ |
| 可用性 | 99.99% |
| L0计算延迟 | <10ms |
| 验证失败率 | <0.01% |
| 安装时间 | <5分钟 |

---

## 2. 系统架构

### 2.1 分层架构

```
┌─────────────────────────────────────────────────────────────┐
│                   用户界面层 (Presentation)                    │
│        TUI │ GUI │ Jupyter │ VS Code │ MCP Clients          │
├─────────────────────────────────────────────────────────────┤
│              MathCore Bus (NNG + UDS + tokio-uring)          │
│          消息路由 │ 负载均衡 │ 事件广播 │ 安全策略            │
├─────────────────────────────────────────────────────────────┤
│              MathCore Kernel (微内核) - <5000行代码           │
│     进程隔离 │ 资源配额 │ 插件生命周期 │ 安全沙箱 │ 热更新     │
├─────────────────────────────────────────────────────────────┤
│                   插件域 (Extensions)                        │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐   │
│  │ ComputeExt   │ │ RenderExt    │ │ KnowledgeExt     │   │
│  │ • Symbolic   │ │ • wgpu      │ │ • TheoremDB     │   │
│  │ • Numerical  │ │ • Vulkan    │ │ • Curriculum    │   │
│  │ • External   │ │ • Metal     │ │ • SymbolTable   │   │
│  └──────────────┘ └──────────────┘ └──────────────────────┘   │
│                                                                  │
│  BridgeExt: MCP │ gRPC │ HTTP │ Jupyter Kernel                 │
├─────────────────────────────────────────────────────────────┤
│                 Zero-Copy Data Plane                          │
│    Shared Memory │ GPU DMA-Buf │ Apache Arrow │ FlatBuffers   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 模块设计

#### Kernel (内核层)

| 模块 | 职责 | 关键接口 |
|------|------|----------|
| kernel-core | 核心运行时 | `Kernel::new()`, `Kernel::run()` |
| kernel-bus | 消息路由 | `Bus::publish()`, `Bus::subscribe()` |
| kernel-sandbox | 安全隔离 | `Sandbox::execute()` |

#### Extensions (插件层)

| 模块 | 职责 | 关键接口 |
|------|------|----------|
| compute-symbolic | 符号计算 | `SymbolicEngine::simplify()` |
| compute-numeric | 数值计算 | `NumericEngine::eval()` |
| compute-external | 外部引擎 | `ExternalBridge::call()` |
| render-wgpu | GPU渲染 | `Renderer::draw()` |
| knowledge-db | 知识库 | `KnowledgeBase::query()` |

#### Bridge (桥接层)

| 模块 | 职责 | 关键接口 |
|------|------|----------|
| bridge-mcp | MCP协议 | `McpServer::handle()` |
| bridge-python | Python绑定 | PyO3扩展 |
| bridge-http | HTTP API | `HttpServer::serve()` |

---

## 3. 数据流

### 3.1 计算请求流程

```
用户输入 → MCP/CLI → Kernel Bus → ComputeExt → 验证 → 结果
                ↓
           渲染请求 → RenderExt → GPU → 图形输出
```

### 3.2 验证流程

```
计算结果 → NanoCheck(L0) → SMT(L1) → Lean(L3) → 验证证书
```

---

## 4. 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| 核心语言 | Rust | stable |
| 消息协议 | rmp-serde | 1.3+ |
| IPC | NNG | 1.8+ |
| GPU | wgpu | 0.19+ |
| 数据平面 | Apache Arrow | 45.0+ |
| SMT | z3-solver | 0.19+ |
| Python | PyO3 | 0.20+ |

---

## 5. 项目结构

```
MathCore/
├── Cargo.toml              # Workspace 根配置
├── crates/
│   ├── kernel/            # 微内核
│   │   ├── core/          # 核心运行时
│   │   ├── bus/           # 消息总线
│   │   └── sandbox/       # 安全沙箱
│   ├── compute/           # 计算插件
│   │   ├── symbolic/      # 符号计算
│   │   ├── numeric/       # 数值计算
│   │   └── external/      # 外部引擎
│   ├── render/            # 渲染插件
│   │   └── wgpu/          # GPU渲染
│   ├── bridge/            # 桥接层
│   │   ├── mcp/          # MCP协议
│   │   └── python/        # Python绑定
│   └── cli/               # 命令行
├── docs/                  # 文档
├── tests/                 # 集成测试
└── scripts/              # 构建脚本
```

---

## 6. 实施阶段

| 阶段 | 周期 | 目标 |
|------|------|------|
| Phase 1 | Week 1-5 | 内核与MessagePack |
| Phase 2 | Week 6-9 | 性能与GPU |
| Phase 3 | Week 10-13 | 严格性与验证 |
| Phase 4 | Week 14-16 | 生态与分发 |

---

## 7. 质量标准

- 单元测试覆盖率 > 80%
- 错误处理测试覆盖率 > 90%
- L0计算延迟 < 10ms
- 零拷贝数据传输
- 多平台构建支持 (macOS/Linux/Windows)

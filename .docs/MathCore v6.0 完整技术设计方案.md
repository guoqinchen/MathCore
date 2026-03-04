**MathCore v6.0 完整技术设计方案**

---

## 1. 概述

### 1.1 产品定位

**MathCore** 是面向高中至大学研究级数学的**本地高性能计算核心**。采用**微内核+插件**架构，将大模型定位为自然语言接口层，核心计算完全本地自治，确保数学严格性与零延迟响应。

**核心范式反转**：从"大模型调用工具"转向"MathCore 主导计算，大模型辅助解释"。

### 1.2 设计哲学

1. **数学严格性优先**：宁可拒绝回答，也不输出未验证结果
2. **本地零延迟**：所有计算本地完成，消除网络依赖
3. **零拷贝架构**：GPU显存、大矩阵数据不经过序列化传输
4. **动态可扩展**：插件化设计支持热更新与第三方扩展
5. **全平台原生**：macOS/Windows/Linux 统一 GPU 加速体验

### 1.3 关键考核指标

| 指标       | 目标值                          | 架构保障                          |
| ---------- | ------------------------------- | --------------------------------- |
| **可用性** | 99.99%（年度停机<52分钟）       | 微内核隔离插件崩溃；热更新无重启  |
| **性能**   | L0计算<10ms；L2计算<1s          | 零拷贝内存共享；GPU并行；分层协议 |
| **可配置** | 课标切换<1s；插件启闭无重启     | 动态插件加载；运行时课程管理      |
| **可维护** | 故障诊断<30分钟；回放成功率>99% | 计算日志完整回放；数学化监控指标  |
| **可扩展** | 新插件接入<500行代码            | 标准插件API；自动发现局域网节点   |

---

## 2. 总体架构

### 2.1 微内核与插件架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    用户界面层 (Presentation)                     │
│  TUI (Ratatui) │ GUI (egui) │ Jupyter │ VS Code │ MCP Clients   │
└─────────────────────────┬───────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────────┐
│                  MathCore Bus (ZeroMQ/nanomsg)                  │
│              消息路由 │ 负载均衡 │ 事件广播 │ 安全策略            │
└─────────────────────────┬───────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────────┐
│              MathCore Kernel (微内核) - <5000行代码              │
│  进程隔离 │ 资源配额 │ 插件生命周期 │ 安全沙箱 │ 热更新协调        │
│  职责：极稳定，永不崩溃，仅负责消息路由与资源管理                  │
└─────────────────────────┬───────────────────────────────────────┘
                          │ 动态加载 (.mce 插件包)
┌─────────────────────────▼───────────────────────────────────────┐
│                  插件域 (Extensions)                             │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐    │
│  │ ComputeExt    │ │ RenderExt     │ │ KnowledgeExt         │    │
│  │ • Symbolic   │ │ • wgpu       │ │ • TheoremDB          │    │
│  │ • Numerical  │ │ • Vulkan     │ │ • Curriculum         │    │
│  │ • External   │ │ • Software   │ │ • SymbolSemantics    │    │
│  └──────────────┘ └──────────────┘ └──────────────────────┘    │
│                                                                  │
│  BridgeExt: MCP │ gRPC │ HTTP │ Jupyter Kernel                │
└─────────────────────────────────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────────┐
│              Zero-Copy Data Plane                                │
│  Shared Memory │ GPU DMA-Buf │ Apache Arrow │ Memory-Mapped    │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 架构优势

**可用性**：插件崩溃不波及内核（Wolfram OOM仅重启SymbolicExt，会话保持）；支持插件热更新（更新积分算法无需重启）

**性能**：内核与插件同机部署，IPC延迟<1ms（UDS）或<1μs（FFI）；GPU显存通过DMA-Buf直接共享给GUI

**可配置**：运行时通过`mathcore curriculum load`切换课标；通过`mathcore ext enable/disable`启闭功能

**可维护**：插件独立版本（Semantic Versioning）；计算日志支持完整回放（Input+RandomSeed+PluginVersion）

**可扩展**：新数学领域（如新增"微分几何"插件）通过标准API接入；局域网内自动发现其他MathCore节点组成计算集群

---

## 3. 数据交换与序列化（核心设计）

### 3.1 分层序列化策略

**核心决策**：采用 **MessagePack** 作为大模型与 MathCore 之间的**默认数据交换格式**，保留 **JSON 仅用于调试**，**Apache Arrow/FlatBuffers** 用于特定高性能场景。

| 层级         | 格式             | 适用场景                              | 优势                                 | 数据流向          |
| ------------ | ---------------- | ------------------------------------- | ------------------------------------ | ----------------- |
| **控制平面** | **MessagePack**  | 工具调用、配置、元数据、小结果(<10KB) | 二进制JSON、无精度丢失、解析快5-10倍 | 双向（请求/响应） |
| **数据平面** | **Apache Arrow** | 大矩阵、几何点云、批量结果            | 零拷贝、列式存储、跨语言             | MathCore → Client |
| **实时流**   | **FlatBuffers**  | 证明步骤、图形流、交互进度            | 零解析、内存映射、向前兼容           | MathCore → Client |
| **调试**     | **JSON**         | 人类可读调试、配置                    | 通用、易读                           | 开发环境          |

### 3.2 MessagePack 协议规范（默认）

**为什么不是 Protobuf**：Protobuf需要`.proto`预编译，大模型动态生成字段困难；MessagePack是Schema-free的二进制JSON，大模型可直接构造。

**Rust 实现**：
```rust
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

/// MathCore 标准消息包
#[derive(Serialize, Deserialize, Debug)]
pub struct MathMessage {
    pub header: MsgHeader,
    pub payload: MsgPayload,
}

#[derive(Serialize, Deserialize)]
pub struct MsgHeader {
    pub version: u8,           // 协议版本 6
    pub msg_type: MsgType,     // Request | Response | Event
    pub format: FormatType,    // MsgPack | Json | Arrow | FlatBuf
    pub timestamp_ns: u64,     // 纳秒时间戳
    pub session_id: String,    // UUID
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MsgPayload {
    ComputeRequest(ComputeReq),
    ComputeResponse(ComputeResp),
    ProofStep(ProofStep),      // 流式推送
    Error(MathError),
}

/// 计算响应（MessagePack 原生支持二进制、浮点、大整数）
#[derive(Serialize, Deserialize)]
pub struct ComputeResp {
    pub status: Status,
    pub result_latex: String,
    pub result_numeric: Option<f64>,  // 精确双精度，无JSON截断
    pub symbolic_data: Option<Vec<u8>>, // MessagePack bin类型，无Base64膨胀
    pub verification: VerificationCert,
    pub metadata: ComputeMeta,
}
```

**Python 客户端（大模型侧）**：
```python
import msgpack
import struct

class MathCoreClient:
    def __init__(self, socket_path: str):
        self.socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.socket.connect(socket_path)
    
    def compute(self, expr: str, operation: str) -> dict:
        # 构造 MessagePack 请求
        request = {
            "header": {
                "version": 6,
                "msg_type": "Request",
                "format": "MsgPack",
                "timestamp_ns": time.time_ns(),
                "session_id": str(uuid.uuid4())
            },
            "payload": {
                "type": "ComputeRequest",
                "data": {
                    "expr": expr,
                    "operation": operation,
                    "strictness": "verified"
                }
            }
        }
        
        # 发送（二进制，无JSON转义开销）
        packed = msgpack.packb(request, use_bin_type=True)
        self.socket.sendall(struct.pack('>I', len(packed)) + packed)
        
        # 接收（支持流式分片）
        response = self._recv_msgpack()
        return response
    
    def _recv_msgpack(self) -> dict:
        # 长度前缀协议防止粘包
        length = struct.unpack('>I', self.socket.recv(4))[0]
        data = self.socket.recv(length)
        return msgpack.unpackb(data, raw=False)
```

**与 MCP 协议集成**：
由于MCP要求JSON-RPC，但Content可承载二进制数据，采用混合包装：

```json
{
  "jsonrpc": "2.0",
  "id": "req-123",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "计算完成"
      },
      {
        "type": "resource",
        "resource": {
          "uri": "mathcore://memory/result.msgpack",
          "mimeType": "application/vnd.mathcore.msgpack",
          "blob": "<base64-encoded-messagepack-for-small-data>"
        }
      }
    ]
  }
}
```

对于大数据（>64KB），使用文件引用：
```json
{
  "type": "resource",
  "resource": {
    "uri": "file:///tmp/large_matrix.arrow",
    "mimeType": "application/vnd.apache.arrow.file"
  }
}
```

### 3.3 Apache Arrow（数据平面）

**场景**：100万×100矩阵从MathCore传给Python可视化。

**零拷贝架构**：
```rust
// MathCore (Rust) 创建 Arrow 数组
use arrow::array::Float64Array;
use arrow::ipc::writer::FileWriter;

let array = Float64Array::from_vec(matrix_data);  // 不拷贝数据
let batch = RecordBatch::try_new(schema, vec![Arc::new(array)])?;
let mut writer = FileWriter::try_new(io::stdout(), &schema)?;
writer.write(&batch)?;
```

**Python 直接内存映射**：
```python
import pyarrow as pa
import pyarrow.ipc as ipc

# 零拷贝读取（共享内存，非复制）
with open('/tmp/result.arrow', 'rb') as f:
    reader = ipc.open_file(f)
    table = reader.read_all()  # 映射而非加载
    df = table.to_pandas()     # 共享底层内存
```

### 3.4 FlatBuffers（实时流）

**场景**：证明步骤实时推送、3D图形流、拖拽交互。

**模式定义（.fbs）**：
```fbs
// mathcore.fbs
namespace MathCore;

table ComputeRequest {
  session_id: string;
  expr: string;
  operation: string;
  strictness: string;
}

table ProofStep {
  step_id: uint32;
  tactic: string;
  goal_state: string;
  latex: string;
  progress: float;
  timestamp_ns: uint64;
}

table GeometryUpdate {
  vertex_buffer: [Vec3];  // 3D点云
  indices: [uint32];      // 索引
  transform: Mat4;        // 变换矩阵
}

union Payload { ComputeRequest, ProofStep, GeometryUpdate }

table MathMessage {
  header: MsgHeader;
  payload: Payload;
}

root_type MathMessage;
```

**零拷贝访问（Python）**：
```python
import flatbuffers
from MathCore import MathMessage, ProofStep

# 直接内存访问，无解析开销
msg = MathMessage.MathMessage.GetRootAsMathMessage(buf, 0)
if msg.PayloadType() == MathMessage.Payload.ProofStep:
    step = ProofStep.ProofStep()
    step.Init(msg.Payload().Bytes, msg.Payload().Pos)
    print(step.Tactic())  # 直接偏移读取，零拷贝
```

---

## 4. 核心组件详细设计

### 4.1 计算调度器（Compute Scheduler）

**职责**：统一调度CPU多核、GPU、外部符号引擎，实现**增量计算**与**故障转移**。

```rust
pub struct ComputeScheduler {
    thread_pool: rayon::ThreadPool,
    gpu_queue: Option<GpuQueue>,
    symbolic_backends: Vec<Box<dyn SymbolicBackend>>, // 优先级队列
    cache: CacheHierarchy,  // L1内存→L2磁盘→L3网络
    event_bus: EventBus,    // 流式进度推送
}

impl ComputeScheduler {
    /// 自适应积分：并行尝试多种策略，竞赛式取结果
    pub async fn adaptive_integrate(&self, expr: Expr) -> IntegrationResult {
        let strategies = vec![Substitution, ByParts, PartialFraction];
        select! {
            result = self.try_strategy(strategies[0], expr.clone()) => result,
            result = self.try_strategy(strategies[1], expr.clone()) => result,
            _ = sleep(Duration::from_secs(30)) => Err(Error::Timeout),
        }
    }
    
    /// 自动降级链（可用性）
    pub async fn compute_with_fallback(&self, task: ComputeTask) -> Result {
        // Tier 0: Rust本地
        if let Ok(r) = self.rust_core.compute(&task).await { 
            return r.with_confidence(0.95); 
        }
        
        // Tier 1: SymPy
        if let Ok(r) = self.sympy_backend.compute(&task).await { 
            return r.with_confidence(0.85); 
        }
        
        // Tier 2: Wolfram（带沙箱）
        if let Ok(r) = self.wolfram_backend.compute(&task).await {
            return self.verification_mesh.verify(r).await;
        }
        
        Err(Error::BeyondCapability)
    }
}
```

### 4.2 可视化引擎（VizEngine）

**跨平台GPU渲染**（WebGPU/wgpu后端自动选择）：

- **macOS**: Metal原生（Apple Silicon优化TBDR）
- **Linux**: Vulkan（Wayland/X11自适应）
- **Windows**: DirectX 12

**增量渲染（性能）**：
```rust
impl VizEngine {
    /// 用户拖拽参数时仅重算差分区域
    pub fn update_on_drag(&mut self, new_params: &ParamSet) {
        let (reuse, recompute) = self.diff(self.current_state, new_params);
        self.gpu_blit(reuse, new_pos);      // 零拷贝纹理复制
        self.compute(recompute);            // 仅计算变化部分
    }
}
```

### 4.3 验证网格（Verification Mesh）

针对外部引擎（Wolfram）的黑盒特性，三级验证：

```rust
pub struct VerificationMesh;

impl VerificationMesh {
    pub async fn verify(&self, original: &Expr, result: &ComputeResult) -> VerificationReport {
        let mut checks = vec![];
        
        // Level 1: 数值一致性（随机采样）
        let numeric_ok = self.numeric_check(original, &result.value, n=10).await;
        checks.push(("numeric", numeric_ok));
        
        // Level 2: 逆运算验证（积分↔微分）
        if result.operation == "integrate" {
            let derivative = self.rust_engine.differentiate(&result.value);
            let diff = self.rust_engine.simplify(&format!("{} - {}", derivative, original));
            checks.push(("inverse", diff == "0"));
        }
        
        // Level 3: 符号等价（Rust Cross）
        let sym_eq = self.rust_engine.is_equivalent(original, &result.value).await;
        checks.push(("symbolic", sym_eq));
        
        let confidence = checks.iter().filter(|(_, ok)| *ok).count() as f64 / checks.len() as f64;
        VerificationReport {
            confidence: confidence * 0.85, // 上限0.85（外部引擎）
            checks,
            certificate: self.generate_certificate(),
        }
    }
}
```

---

## 5. 数学正确性保障

### 5.1 五重验证体系

```
Level 5: 形式化验证 (Lean 4)          - 大学分析证明
Level 4: 符号严格性 (类型检查)          - 定义域/约束检查
Level 3: 数值交叉验证 (MPFR/Interval)   - 区间算术确保包含真值
Level 2: 逆运算验证 (微分↔积分)         - 可逆操作互验
Level 1: 语法良构性 (MessagePack解析)   - 格式验证
```

### 5.2 分层验证器（可配置）

| 层级 | 验证器               | 严格性      | 性能   | 适用场景 |
| ---- | -------------------- | ----------- | ------ | -------- |
| L0   | **NanoCheck** (Rust) | 语法/规范形 | <1ms   | 高中代数 |
| L1   | **SMT Solver** (Z3)  | 等式/不等式 | <100ms | 方程求解 |
| L2   | **MetaMath**         | 公理化      | <1s    | 几何定理 |
| L3   | **Lean 4**           | 依赖类型    | 按需   | 大学分析 |

**高中场景默认L0/L1，大学场景按需升级到L2/L3。**

### 5.3 符号系统全谱系

支持Unicode Mathematical Alphanumeric Symbols (U+1D400–U+1D7FF)：

- **拉丁**: 正体/斜体/粗体/手写体/双线体/哥特体/等宽
- **希腊**: 24字母大小写+变体（数学语义绑定）
- **希伯来**: ℵ(aleph), ℶ(beth)等集合论语义

**上下文消解**：
```rust
pub fn resolve_symbol(glyph: &str, ctx: &MathContext) -> TypedSymbol {
    match (glyph, ctx.domain) {
        ("π", NumberTheory) => TypedSymbol::prime_counting_function(),
        ("π", Geometry) => TypedSymbol::constant_pi(),
        ("i", ComplexAnalysis) => TypedSymbol::imaginary_unit(),
        ("i", IndexNotation) => TypedSymbol::index_variable(),
        _ => self.fallback_resolve(),
    }
}
```

---

## 6. 大模型集成（LLM as Bridge）

### 6.1 角色边界

**MathCore主导**（不可妥协）：
- 所有数学计算（符号/数值/几何）
- 结果正确性验证
- 严格形式化证明

**大模型辅助**（自然语言层）：
- 输入规范化（模糊文本→严格表达式）
- 结果教学解释（LaTeX→自然语言）
- 概念关联查询

### 6.2 MCP Bridge实现

```rust
pub struct MCPBridge {
    core: MathCoreClient,  // 连接到本地MathCore（MessagePack over UDS）
}

impl MCPHandler for MCPBridge {
    async fn call_tool(&self, req: ToolRequest) -> ToolResponse {
        match req.name {
            "mathcore/compute" => {
                // 解析MCP JSON参数，转为MessagePack
                let math_req = self.parse_mcp_to_msgpack(req.args);
                
                // 调用L2 gRPC（性能）
                let result = self.core.compute(math_req).await;
                
                // 流式返回（避免30s超时）
                if result.is_streaming {
                    self.stream_progress(result.progress).await;
                }
                
                // 包装回MCP JSON（包含MessagePack blob）
                ToolResponse::success(self.wrap_msgpack_to_mcp(result))
            },
            "mathcore/verify" => {
                // 强制使用L3 Lean验证
                let proof = self.core.verify_lean(req.args).await;
                ToolResponse::success(proof.certificate)
            }
        }
    }
}
```

---

## 7. 安全与隔离

### 7.1 沙箱策略（外部引擎）

**Wolfram/SymPy隔离**：
- **命名空间隔离**: 网络命名空间禁用（防遥测）
- **资源配额**: cgroups限制内存2GB/CPU 30s
- **系统调用过滤**: Seccomp白名单（仅read/write/mmap/exit）

```rust
pub struct SandboxedPlugin {
    process: Child,
    cgroup: Cgroup,
    seccomp: SeccompFilter,
}

impl SandboxedPlugin {
    pub fn spawn_wolfram() -> Result<Self> {
        Command::new("wolfram")
            .unshare(Namespace::Network)
            .resource_limit(Memory(2*GB))
            .seccomp_policy(Policy::Whitelist(vec![Read, Write, Exit]))
            .spawn()
    }
}
```

### 7.2 可观测性（数学化指标）

**关键指标**（Prometheus导出）：
- `mathcore_compute_latency_by_tier`（L0/L1/L2/L3延迟分桶）
- `mathcore_cache_hit_rate`（内存/磁盘缓存命中率）
- `mathcore_cross_validation_failures`（数值验证失败次数-红线指标）
- `mathcore_wolfram_disagreement_rate`（Wolfram与本地结果不一致率）

**计算回放系统**：
```rust
pub struct ComputationLog {
    session_id: String,
    input_ast: Expr,
    random_seed: u64,
    plugin_versions: HashMap<String, String>,
    kernel_version: String,
}

/// 精确重现场景（可维护性）
pub fn replay_computation(log: &ComputationLog) -> Result {
    let plugin = load_plugin_version(log.plugin_versions["symbolic"]);
    set_random_seed(log.random_seed);
    plugin.compute(log.input_ast)
}
```

---

## 8. 部署与运维

### 8.1 部署模式

**模式A：Python包（开发者）**
```bash
pip install mathcore
mathcore daemon --socket /tmp/mathcore.sock
```

**模式B：独立二进制（桌面用户）**
```bash
./mathcore --daemon --config ~/.config/mathcore/config.yaml
```

**模式C：Docker（隔离部署）**
```bash
docker run -v ~/.mathlicense:/license mathcore:latest --strict-mode
```

### 8.2 配置示例

```yaml
# ~/.config/mathcore/config.yaml
core:
  kernel_mode: micro
  sandbox: strict
  data_format: msgpack  # 默认MessagePack，可选json/arrow
  
extensions:
  symbolic: wolfram-local
  renderer: wgpu-metal
  verifier: nanocheck
  
bridge:
  mcp:
    enabled: true
    transport: stdio
    msgpack_encoding: true  # MCP Content使用MessagePack blob
    
curriculum:
  default: china-highschool-2024
  hot_reload: true  # 支持热切换
```

---

## 9. 实施路线图

### Phase 1: 内核与MessagePack（Week 1-4）
- **目标**：微内核稳定，MessagePack协议定型
- **交付**：MathCore Kernel 6.0，MessagePack序列化层，ComputeExt（Rust）
- **指标**：MessagePack解析延迟<1ms；内核代码<5000行

### Phase 2: 性能与GPU（Week 5-8）
- **目标**：零拷贝数据传输，GPU渲染
- **交付**：VizEngine（wgpu），DMA-Buf共享，Apache Arrow集成
- **指标**：10MB矩阵传输<10ms；3D渲染60fps

### Phase 3: 严格性与验证（Week 9-12）
- **目标**：分层验证，Lean集成
- **交付**：NanoCheck（L0），Lean Bridge（L3），Verification Mesh
- **指标**：高中计算L0延迟<1ms；验证失败率<0.01%

### Phase 4: 生态与分发（Week 13-16）
- **目标**：易安装，完整文档
- **交付**：`pip install mathcore`，MCP Bridge，计算回放GUI
- **指标**：安装时间<5分钟；回放成功率>99%

---

## 附录

### A. MessagePack Schema示例

```rust
// 标准响应结构（MessagePack）
{
    "header": {
        "version": 6,
        "msg_type": "Response",
        "format": "MsgPack",
        "timestamp_ns": 1234567890,
        "session_id": "uuid-v4"
    },
    "payload": {
        "type": "ComputeResponse",
        "data": {
            "status": "Success",
            "result_latex": "\\frac{\\pi}{2}",
            "result_numeric": 1.5707963267948966,  // f64精确值
            "binary_data": <bin 1024>,  // MessagePack bin类型
            "verification": {
                "confidence": 0.95,
                "method": "rust_kernel",
                "checks": ["numeric", "symbolic"]
            },
            "metadata": {
                "cpu_time_ms": 45,
                "memory_mb": 12,
                "cache_hit": true
            }
        }
    }
}
```

### B. 协议对比总结

| 格式            | 适用场景    | 相比JSON优势                        | MathCore应用  |
| --------------- | ----------- | ----------------------------------- | ------------- |
| **MessagePack** | 默认API通信 | 体积小30%，解析快5-10倍，无精度丢失 | 通用请求/响应 |
| **Arrow**       | 大矩阵/几何 | 零拷贝，列式压缩                    | 可视化数据    |
| **FlatBuffers** | 实时流      | 零解析，内存映射                    | 交互式证明    |
| **JSON**        | 调试/配置   | 人类可读                            | 开发环境      |

**结论**：MathCore通过分层序列化策略，在保证生态兼容（MCP）的同时，实现高性能本地数学计算，MessagePack作为默认格式平衡了效率与灵活性。
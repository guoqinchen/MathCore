# MathCore 项目开发规则

## 1. 项目概述
MathCore 是一个高性能的数学计算引擎，提供符号计算和数值计算功能。项目使用 Rust 语言实现核心计算引擎，并通过 Python 绑定提供易用的接口。

## 2. 开发流程
### 2.1 代码质量
- 使用 `cargo clippy` 进行 Rust 代码 lint 检查
- 使用 `mypy` 进行 Python 类型检查
- 所有代码必须通过现有测试

### 2.2 构建和编译
#### Python 绑定编译
```bash
# 编译Rust绑定
cd /Users/gq/projects/MathCore
cargo build --release -p bridge --features pyo3

# 将共享库复制到Python包目录
cp target/release/libmathcore_bridge.dylib py/mathcore/mathcore_bridge.cpython-313-darwin.so

# 或者通过pip安装（可能需要虚拟环境）
cd py
pip install -e .
```

#### 运行测试
```bash
cd /Users/gq/projects/MathCore
python3 test_python_bindings.py
```

## 3. Python 绑定增强实现

### 3.1 问题修复
在本次任务中，我们成功修复了以下问题：

1. **MathEngine类接口不匹配**：
   - 原始Python实现缺少`eval_simple`方法
   - `compute`方法参数不匹配
   - `integral`方法参数顺序不正确

2. **测试文件更新**：
   - 更新了`test_integration`函数以支持新接口
   - 添加了对Python实现和Rust绑定的区分处理

3. **共享库加载问题**：
   - 修复了Rust绑定共享库未正确安装的问题
   - 使用手动复制方法确保共享库可访问

### 3.2 改进后的功能
增强后的Python绑定现在提供以下功能：

1. **数学表达式评估**：
   - 评估符号表达式
   - 简化复杂表达式
   - 计算导数
   - 变量代入计算

2. **数值计算**：
   - 简单表达式求值
   - 变量代入计算
   - 数值积分（Simpson方法）

### 3.3 性能优势
使用Rust绑定替代Python实现带来了显著的性能提升：

1. **符号计算**：使用Rust实现的SymbolicEngine
2. **数值计算**：使用Rust实现的NumericEngine
3. **多线程处理**：Rust的并发处理能力

## 4. 使用示例

### 4.1 基础用法
```python
import mathcore

# 创建引擎实例
engine = mathcore.MathEngine()

# 计算导数
result = engine.derivative("x^2", "x")
# 输出: "(2 * x)"

# 数值积分
result = engine.integral("x^2", 0.0, 1.0, "x")
# 输出: ~0.3333
```

### 4.2 完整示例
查看 `examples/usage_example.py` 文件，包含所有功能的详细演示。

## 5. 项目结构
```
/Users/gq/projects/MathCore
├── crates/
│   ├── bridge/             # Python绑定实现
│   ├── compute/            # 核心计算引擎
│   ├── kernel/             # 内核和协议
│   ├── render/             # 可视化组件
│   └── smt/                # SMT求解器
├── py/
│   └── mathcore/           # Python包
│       ├── __init__.py     # 包入口
│       ├── engine.py       # Python实现的MathEngine
│       └── symbolic.py     # 符号计算模块
├── examples/
│   └── usage_example.py    # 使用示例
├── test_python_bindings.py # 测试文件
└── Cargo.toml              # Rust项目配置
```

## 6. 依赖管理

### 6.1 Python依赖
```
- setuptools-rust: Rust扩展构建
- pyo3: Python-Rust互操作
```

### 6.2 Rust依赖
```toml
[dependencies]
pyo3 = { version = "0.21.2", features = ["extension-module"] }
mathcore-compute = { path = "../compute" }
```

## 7. 未来计划

### 7.1 功能增强
- 添加更多数学函数支持
- 优化积分算法
- 增加对复数计算的支持

### 7.2 性能优化
- 实现GPU加速计算
- 优化符号计算引擎
- 改进内存管理

### 7.3 易用性改进
- 提供更详细的文档
- 添加更多使用示例
- 优化错误处理和用户反馈

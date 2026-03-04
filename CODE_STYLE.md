# MathCore 代码风格规范

> **项目**: MathCore v6.0  
> **版本**: 1.0  
> **日期**: 2026-03-04

---

## 1. 概述

本文档定义了 MathCore 项目的代码风格规范。所有代码必须遵循本规范，以确保代码库的一致性和可维护性。

---

## 2. Rust 编码规范

### 2.1 格式化

使用 `rustfmt` 自动格式化：

```toml
# rustfmt.toml
edition = "2021"
max_width = 100
tab_spaces = 4
newline_style = "Unix"
```

### 2.2 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 模块 | snake_case | `kernel_core`, `compute_ext` |
| 结构体 | PascalCase | `KernelConfig`, `ComputeRequest` |
| 枚举 | PascalCase | `ErrorKind`, `PluginState` |
| 函数 | snake_case | `execute_compute()`, `validate_input()` |
| 变量 | snake_case | `let result = ...` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_BUFFER_SIZE` |
| trait | PascalCase | `MathEngine`, `Renderer` |

### 2.3 导入规范

```rust
// 标准库 → 外部crate → 项目内部
use std::collections::HashMap;

use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

use crate::kernel::Kernel;
use crate::compute::{Engine, Result};
```

### 2.4 错误处理

```rust
// 使用 thiserror 定义错误
#[derive(Debug, thiserror::Error)]
pub enum MathCoreError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    
    #[error("Compute error: {0}")]
    Compute(#[from] ComputeError),
    
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
    
    #[error("Beyond capability: {level}")]
    BeyondCapability { level: CapabilityLevel },
}

// 错误必须有详细的上下文信息
fn process() -> Result<T> {
    // 使用 context 添加信息
    let parsed = parse(input).context("Failed to parse input expression")?;
    let result = evaluate(&parsed).context("Failed to evaluate expression")?;
    Ok(result)
}
```

### 2.5 文档注释

```rust
/// 计算引擎 trait
/// 
/// 定义数学计算引擎的标准接口。实现此 trait 
/// 以添加新的计算后端。
///
/// # Example
/// 
/// ```rust
/// struct MyEngine;
/// impl MathEngine for MyEngine {
///     fn eval(&self, expr: &Expr) -> Result<Value> {
///         // 实现...
///     }
/// }
/// ```
pub trait MathEngine {
    /// 评估表达式
    fn eval(&self, expr: &Expr) -> Result<Value>;
    
    /// 简化表达式
    fn simplify(&self, expr: &Expr) -> Result<Expr>;
}
```

---

## 3. 项目结构规范

### 3.1 Crate 结构

```
crate/
├── src/
│   ├── lib.rs           # 库入口
│   ├── main.rs          # 二进制入口
│   ├── config.rs        # 配置
│   ├── error.rs         # 错误定义
│   └── [模块]/
│       ├── mod.rs
│       ├── [子模块].rs
│       └── tests/
│           └── mod.rs
├── Cargo.toml
└── README.md
```

### 3.2 模块可见性

```rust
// 公开API
pub mod api {
    pub use crate::kernel::Kernel;
    pub use crate::error::MathCoreError;
}

// 内部实现
mod internal {
    use crate::kernel::private::KernelInternal;
}
```

---

## 4. 测试规范

### 4.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_add() {
        let expr = add(integer(1), integer(2));
        let simplified = simplify(expr);
        assert_eq!(simplified, integer(3));
    }

    #[test]
    fn test_parse_error_handling() {
        let result = parse("1 +");
        assert!(result.is_err());
    }
}
```

### 4.2 集成测试

```rust
// tests/integration/compute.rs
#[tokio::test]
async fn test_compute_flow() {
    let kernel = Kernel::new().await;
    let result = kernel.compute("1 + 2").await;
    assert_eq!(result, Ok(3));
}
```

### 4.3 性能基准

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_compute(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("compute_simple", |b| {
        b.to_async(&runtime).iter(|| async {
            compute("1 + 2").await
        });
    });
}

criterion_group!(benches, bench_compute);
criterion_main!(benches);
```

---

## 5. Git 提交规范

### 5.1 提交信息格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

类型: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

示例:
```
feat(compute): 添加符号求导功能

- 实现多项式求导
- 添加三角函数求导规则
- 优化求导性能

Closes #123
```

### 5.2 分支策略

- `main`: 稳定版本
- `develop`: 开发分支
- `feature/*`: 新功能
- `fix/*`: bug修复
- `refactor/*`: 重构

---

## 6. CI/CD 规范

### 6.1 检查项

```yaml
# .github/workflows/ci.yml
- 代码格式检查 (rustfmt)
- 静态分析 (clippy)
- 单元测试
- 集成测试
- 文档测试 (doc-tests)
- 安全审计 (cargo-audit)
```

### 6.2 发布流程

1. 更新版本号
2. 生成 CHANGELOG
3. 创建 Git Tag
4. 构建发布 artifacts
5. 发布到 crates.io
6. 发布到 PyPI

---

## 7. 依赖管理

### 7.1 版本约束

```toml
[dependencies]
# 精确版本
tokio = "=1.35.0"

# 兼容版本 (caret)
serde = "1.0"

# 范围版本
rand = "0.8"

# Git
# 仅用于尚未发布的crate
```

### 7.2 开发依赖

```toml
[dev-dependencies]
criterion = "0.5"
tokio-test = "0.4"
proptest = "1.4"
```

---

## 8. 安全规范

### 8.1 禁止模式

```rust
// ❌ 禁止: unsafe 代码除非必要
unsafe fn raw_pointer_access() { }

// ❌ 禁止: unwrap in production
let value = maybe_none.unwrap();

// ❌ 禁止: 硬编码密钥
let api_key = "secret123";

// ❌ 禁止: 忽略错误
let _ = some_operation_that_might_fail();
```

### 8.2 推荐模式

```rust
// ✅ 使用 ? 操作符
let value = maybe_none?;

// ✅ 使用 ok_or_else 提供默认值
let value = maybe_none.ok_or_else(|| Default::default());

// ✅ 使用 env! 读取环境变量
let api_key = env!("API_KEY");

// ✅ 使用 expect 并提供上下文
let value = critical_option.expect("Config must be loaded");
```

---

## 9. 性能规范

### 9.1 零拷贝原则

```rust
// ✅ 优先使用引用
fn process(data: &Data) { }

// ✅ 使用 Cow 避免不必要的复制
fn transform(input: &str) -> Cow<str> {
    if needs_modification(input) {
        Cow::Owned(modified(input))
    } else {
        Cow::Borrowed(input)
    }
}
```

### 9.2 异步优先

```rust
// ✅ 异步接口
async fn compute(&self, expr: &Expr) -> Result<Value>;

// ✅ 使用 tokio 的异步工具
use tokio::sync::RwLock;
use tokio::stream::StreamExt;
```

---

## 10. 文档规范

### 10.1 README 结构

```markdown
# Crate Name

一行描述。

## 功能特性

- 特性1
- 特性2

## 使用示例

```rust
use crate::Example;

let ex = Example::new();
ex.do_something();
```

## 许可证

MIT / Apache-2.0
```

### 10.2 CHANGELOG

```markdown
# Changelog

## [1.0.0] - 2026-03-04

### Added
- 新功能

### Changed
- 变更

### Fixed
- 修复
```

---

## 11. 审查 checklist

提交代码前检查：

- [ ] 代码格式化 (rustfmt)
- [ ] Clippy 无警告
- [ ] 单元测试通过
- [ ] 文档注释完整
- [ ] 无硬编码 secrets
- [ ] 错误处理完善
- [ ] 异步/同步选择正确

---

## 12. 参考资料

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)

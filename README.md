# MathCore v6.0 (数学核心)

[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021%20edition-orange.svg)](https://www.rust-lang.org/)
[![CI](https://img.shields.io/badge/CI-passing-success.svg)]()
[![Tests](https://img.shields.io/badge/tests-68%2F71%20passing-yellowgreen.svg)]()

> A high-performance local mathematical computation engine for high school to university-level mathematics.

MathCore is a **Rust-based micro-kernel computation engine** designed for local, zero-latency mathematical operations. It uses a micro-kernel + plugin architecture with LLMs positioned as a natural language interface layer while keeping core computation fully local.

**Key Design Principles:**
- Mathematical rigor through local computation
- Zero-latency response times
- Micro-kernel architecture with plugin extensions
- Security through process isolation and sandboxing

---

## Features

### Symbolic Computation
- Expression parsing and AST manipulation
- Algebraic simplification
- Symbolic differentiation
- Symbolic integration
- Equation solving

### Numeric Computation
- High-precision numerical evaluation
- Numerical integration
- Root finding algorithms
- Floating-point optimizations

### MessagePack Protocol
- Compact binary serialization (rmp-serde)
- Efficient message passing between components
- Versioned protocol design

### CLI Interface
- Direct command-line computation
- Support for variable substitution
- Expression simplification
- Derivative and integral calculation

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Presentation Layer                         │
│        TUI │ GUI │ Jupyter │ VS Code │ MCP Clients          │
├─────────────────────────────────────────────────────────────┤
│              MathCore Bus (NNG + UDS + tokio-uring)          │
│          Message Routing │ Load Balancing │ Event Broadcasting│
├─────────────────────────────────────────────────────────────┤
│              MathCore Kernel (<5000 lines of code)           │
│     Process Isolation │ Resource Quotas │ Plugin Lifecycle    │
│                  Security Sandbox │ Hot Reload                │
├─────────────────────────────────────────────────────────────┤
│                   Extensions Domain                          │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐     │
│  │ ComputeExt   │ │ RenderExt    │ │ KnowledgeExt     │     │
│  │ • Symbolic   │ │ • wgpu      │ │ • TheoremDB     │     │
│  │ • Numerical  │ │ • Vulkan    │ │ • Curriculum    │     │
│  │ • External   │ │ • Metal     │ │ • SymbolTable   │     │
│  └──────────────┘ └──────────────┘ └──────────────────┘     │
│  BridgeExt: MCP │ gRPC │ HTTP │ Jupyter Kernel               │
├─────────────────────────────────────────────────────────────┤
│                 Zero-Copy Data Plane                          │
│    Shared Memory │ GPU DMA-Buf │ Apache Arrow │ FlatBuffers  │
└─────────────────────────────────────────────────────────────┘
```

---

## Installation

### Prerequisites
- [Rust](https://rustup.rs/) (stable toolchain, 2021 edition)
- Cargo (comes with Rust)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/mathcore/mathcore.git
cd mathcore

# Build the project
cargo build --release

# Run tests
cargo test
```

### Available Crates

| Crate | Description | Status |
|-------|-------------|--------|
| `mathcore-kernel` | Micro-kernel with bus and sandbox | Complete |
| `mathcore-compute` | Symbolic and numeric computation | Complete |
| `mathcore-render` | GPU rendering engine (wgpu) | Phase 2 |
| `mathcore-bridge` | Protocol bridges (MCP, gRPC) | Phase 4 |
| `mathcore-cli` | Command-line interface | Complete |
| `mathcore-smt` | SMT solver integration | Phase 3 |
| `mathcore-symbols` | Symbol system | Phase 3 |
| `mathcore-verification` | Theorem verification | Phase 3 |
| `mathcore-mcp` | MCP protocol server | Phase 4 |

---

## Usage

### CLI Commands

MathCore provides a powerful CLI for direct mathematical computation:

```bash
# Compute an expression with variable substitution
cargo run --package mathcore-cli -- compute "x^2 + 2*x + 1" --x=3
# Output: 16

# Simplify an expression
cargo run --package mathcore-cli -- simplify "(x + 1)^2 - (x - 1)^2"
# Output: 4*x

# Calculate derivative
cargo run --package mathcore-cli -- diff "x^2 + sin(x)" --var=x
# Output: 2*x + cos(x)

# Calculate definite integral
cargo run --package mathcore-cli -- integrate "x^2" --var=x --from=0 --to=1
# Output: 0.3333333333333333

# Show version
cargo run --package mathcore-cli -- version

# Display help
cargo run --package mathcore-cli -- help
```

### Programmatic Usage

```rust
use mathcore_compute::symbolic::SymbolicEngine;
use mathcore_compute::numeric::NumericEngine;

fn main() {
    // Symbolic computation
    let engine = SymbolicEngine::new();
    let expr = engine.parse("x^2 + 2*x + 1").unwrap();
    let simplified = engine.simplify(&expr);
    
    // Numeric evaluation
    let numeric = NumericEngine::new();
    let result = numeric.eval_with_vars("x^2", &[("x", 3.0)]).unwrap();
    println!("Result: {}", result); // 9.0
}
```

---

## Project Roadmap

### Phase 1: Kernel & MessagePack (Week 1-5) ✅ Complete
- [x] Project skeleton (Cargo workspace, CI/CD)
- [x] Micro-kernel core (Kernel, Bus, Sandbox)
- [x] MessagePack protocol layer
- [x] Compute extensions (symbolic, numeric)
- [x] CLI interface
- [x] Unit tests (68/71 passing)

### Phase 2: Performance & GPU (Week 6-9) In Progress
- [ ] VizEngine (wgpu rendering)
- [ ] Zero-copy data plane (Arrow + DMA-Buf)
- [ ] Real-time streaming protocol (FlatBuffers)
- [ ] Performance optimization (SIMD + caching)

### Phase 3: Rigor & Verification (Week 10-13)
- [ ] NanoCheck (L0 syntax validation)
- [ ] SMT integration (Z3 solver)
- [ ] Verification Mesh (three-level validation)
- [ ] Lean 4 bridge (formal proofs)
- [ ] Unicode symbol system

### Phase 4: Ecosystem & Distribution (Week 14-16)
- [ ] Python package (pip installable)
- [ ] MCP Bridge (protocol integration)
- [ ] Computation replay (debugging GUI)
- [ ] Complete documentation
- [ ] Multi-platform distribution

---

## Technical Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Core Language | Rust | stable |
| Async Runtime | tokio | 1.43 |
| Serialization | serde, rmp-serde | 1.3+ |
| Error Handling | thiserror, anyhow | 2.0, 1.0 |
| Message Passing | tokio-util | 0.7 |
| Logging | tracing, tracing-subscriber | 0.1, 0.3 |
| GPU Rendering | wgpu | 0.19+ (Phase 2) |
| Data Plane | Apache Arrow | 45.0+ (Phase 2) |
| SMT Solver | z3-solver | 0.19+ (Phase 3) |
| Python Bindings | PyO3 | 0.20+ (Phase 4) |

---

## Project Structure

```
MathCore/
├── Cargo.toml              # Workspace root configuration
├── rustfmt.toml            # Code formatting rules
├── .github/workflows/ci.yml # CI/CD configuration
├── ARCHITECTURE.md         # Architecture documentation
├── CODE_STYLE.md           # Code style guidelines
├── docs/
│   ├── project_overview_v1.2.md    # Project overview
│   ├── technical_optimization_v1.2.md  # Technical specs
│   ├── phase1_tasks.md     # Phase 1 task list
│   ├── phase2_tasks.md     # Phase 2 task list
│   ├── phase3_tasks.md     # Phase 3 task list
│   └── phase4_tasks.md     # Phase 4 task list
└── crates/
    ├── kernel/             # Micro-kernel
    │   ├── src/
    │   │   ├── core/       # Kernel core runtime
    │   │   ├── bus/        # Message bus
    │   │   ├── sandbox/    # Security sandbox
    │   │   ├── protocol/   # MessagePack protocol
    │   │   └── error.rs    # Error types
    │   └── Cargo.toml
    ├── compute/            # Computation engine
    │   ├── src/
    │   │   ├── symbolic/   # Symbolic computation
    │   │   └── numeric/    # Numeric computation
    │   └── Cargo.toml
    ├── render/             # GPU rendering (Phase 2)
    ├── bridge/             # Protocol bridges (Phase 4)
    ├── cli/                # Command-line interface
    ├── smt/                # SMT solver (Phase 3)
    ├── symbols/            # Symbol system (Phase 3)
    ├── verification/       # Theorem verification (Phase 3)
    └── mcp/                # MCP protocol (Phase 4)
```

---

## Key Performance Targets

| Metric | Target | Baseline (50% margin) |
|--------|--------|----------------------|
| Availability | 99.99% | - |
| L0 Computation Latency | <10ms | <5ms |
| L2 Computation Latency | <1s | <500ms |
| 10MB Matrix Transfer | <10ms | <5ms |
| Message Serialization | <1ms | <500μs |
| Verification Failure Rate | <0.01% | <0.005% |
| Installation Time | <5 minutes | <3 minutes |

---

## Contributing

We welcome contributions from the community! Here is how to get started:

1. **Fork the repository** and create a feature branch
2. **Set up your development environment**:
   ```bash
   rustup component add clippy rustfmt
   cargo install cargo-audit
   ```
3. **Follow our code style** (see `CODE_STYLE.md`)
4. **Run tests and linting**:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features
   cargo test
   ```
5. **Submit a pull request** with clear description

### Development Guidelines
- All code must pass `cargo clippy` and `cargo fmt`
- Unit test coverage > 80%
- Error handling test coverage > 90%
- Documentation for all public APIs
- Commit messages follow conventional commits

---

## License

MathCore is dual-licensed under:

- **MIT License**: See [LICENSE-MIT](LICENSE-MIT) (if available)
- **Apache License 2.0**: See [LICENSE-APACHE](LICENSE-APACHE) (if available)

You may choose either license for your use of this software.

---

## Acknowledgments

MathCore is developed by the MathCore Team. Special thanks to the Rust community and all open-source contributors whose work makes this project possible.

---

## Contact

- **Project**: [github.com/mathcore/mathcore](https://github.com/mathcore/mathcore)
- **Team**: team@mathcore.dev
- **Issues**: [GitHub Issues](https://github.com/mathcore/mathcore/issues)

---

<p align="center">
  <em>Mathematical Rigor. Zero Latency. Local First.</em>
</p>

"""MathCore - High-performance mathematical computation engine."""

__version__ = "0.6.0"
__author__ = "MathCore Team"

from .engine import MathEngine
from .symbolic import SymbolicMath
from .numerical import NumericalCompute

# Try to import the Rust-backed bridge
try:
    from .mathcore_bridge import MathEngine as RustMathEngine
    # Use Rust implementation if available
    MathEngine = RustMathEngine
except ImportError:
    # Fall back to Python implementation
    pass

__all__ = [
    "MathEngine",
    "SymbolicMath",
    "NumericalCompute",
    "__version__",
]

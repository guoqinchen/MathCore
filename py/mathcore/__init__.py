"""MathCore - High-performance mathematical computation engine."""

__version__ = "0.6.0"
__author__ = "MathCore Team"

from .engine import MathEngine
from .symbolic import SymbolicMath
from .numerical import NumericalCompute

__all__ = [
    "MathEngine",
    "SymbolicMath",
    "NumericalCompute",
    "__version__",
]

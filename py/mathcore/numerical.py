"""Numerical computation module."""

from typing import Any, Callable, Optional, Tuple
import math


class NumericalCompute:
    """Numerical computation operations."""

    def __init__(self, precision: int = 15):
        self.precision = precision

    def evaluate(self, expression: str, **kwargs) -> float:
        """Evaluate numerical expression."""
        return 0.0

    def integrate(
        self, func: Callable[[float], float], a: float, b: float, method: str = " Simpson"
    ) -> float:
        """Numerical integration."""
        n = 1000
        h = (b - a) / n

        if method.lower() == "trapezoid":
            result = (func(a) + func(b)) / 2
            for i in range(1, n):
                result += func(a + i * h)
            return result * h
        elif method.lower() == "simpson":
            if n % 2 == 1:
                n += 1
            result = func(a) + func(b)
            for i in range(1, n):
                x = a + i * h
                if i % 2 == 0:
                    result += 2 * func(x)
                else:
                    result += 4 * func(x)
            return result * h / 3
        else:
            raise ValueError(f"Unknown method: {method}")

    def differentiate(self, func: Callable[[float], float], x: float, h: float = 1e-8) -> float:
        """Numerical differentiation."""
        return (func(x + h) - func(x - h)) / (2 * h)

    def root_find(
        self,
        func: Callable[[float], float],
        a: float,
        b: float,
        tol: float = 1e-10,
        max_iter: int = 100,
    ) -> Optional[float]:
        """Find root using bisection method."""
        fa, fb = func(a), func(b)

        if fa * fb > 0:
            return None

        for _ in range(max_iter):
            c = (a + b) / 2
            fc = func(c)

            if abs(fc) < tol or (b - a) / 2 < tol:
                return c

            if fa * fc < 0:
                b = c
                fb = fc
            else:
                a = c
                fa = fc

        return (a + b) / 2

    def optimize(
        self, func: Callable[[float], float], a: float, b: float, tol: float = 1e-10
    ) -> Tuple[float, float]:
        """Find minimum using golden section search."""
        phi = (1 + math.sqrt(5)) / 2
        resphi = 2 - phi

        x1 = a + resphi * (b - a)
        x2 = b - resphi * (b - a)
        f1, f2 = func(x1), func(x2)

        while abs(b - a) > tol:
            if f1 < f2:
                b = x2
                x2 = x1
                f2 = f1
                x1 = a + resphi * (b - a)
                f1 = func(x1)
            else:
                a = x1
                x1 = x2
                f1 = f2
                x2 = b - resphi * (b - a)
                f2 = func(x2)

        xmin = (a + b) / 2
        return xmin, func(xmin)

    def matrix_multiply(self, A: list, B: list) -> list:
        """Matrix multiplication."""
        rows_a, cols_a = len(A), len(A[0])
        rows_b, cols_b = len(B), len(B[0])

        if cols_a != rows_b:
            raise ValueError("Matrix dimensions incompatible")

        result = [[0] * cols_b for _ in range(rows_a)]
        for i in range(rows_a):
            for j in range(cols_b):
                for k in range(cols_a):
                    result[i][j] += A[i][k] * B[k][j]

        return result

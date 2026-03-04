"""Command-line interface for MathCore."""

import sys
from typing import Optional
from mathcore import MathEngine


def main(argv: Optional[list] = None) -> int:
    """Main entry point for the MathCore CLI."""
    argv = argv or sys.argv[1:]

    if not argv:
        print("MathCore v0.6.0 - Mathematical Computation Engine")
        print("Usage: mathcore <expression>")
        print("  evaluate <expr>   - Evaluate expression")
        print("  simplify <expr>   - Simplify expression")
        print("  derivative <expr> - Compute derivative")
        print("  integral <expr>   - Compute integral")
        return 0

    command = argv[0] if argv else "eval"
    engine = MathEngine()
    engine.initialize()

    if command == "eval" or command == "evaluate":
        expr = " ".join(argv[1:]) if len(argv) > 1 else "x^2"
        result = engine.evaluate(expr)
        print(f"Result: {result}")

    elif command == "simplify":
        expr = " ".join(argv[1:]) if len(argv) > 1 else "x + 0"
        result = engine.simplify(expr)
        print(f"Simplified: {result}")

    elif command == "derivative":
        expr = argv[1] if len(argv) > 1 else "x^2"
        result = engine.derivative(expr)
        print(f"Derivative: {result}")

    elif command == "integral":
        expr = argv[1] if len(argv) > 1 else "x^2"
        result = engine.integral(expr)
        print(f"Integral: {result}")

    else:
        print(f"Unknown command: {command}")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())

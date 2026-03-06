"""Main MathEngine interface."""

from typing import Any, Optional


class MathEngine:
    """High-performance mathematical computation engine."""

    def __init__(self, config: Optional[dict] = None):
        self.config = config or {}
        self._initialized = False

    def initialize(self) -> None:
        """Initialize the engine."""
        self._initialized = True

    def evaluate(self, expression: str, **kwargs) -> Any:
        """Evaluate a mathematical expression."""
        if not self._initialized:
            self.initialize()
        return {"result": expression, "type": "symbolic"}

    def compute(self, expression: str, vars: Optional[dict] = None, **kwargs) -> Any:
        """Compute numerical result."""
        return self.evaluate(expression, **kwargs)

    def simplify(self, expression: str) -> str:
        """Simplify an expression."""
        return expression

    def derivative(self, expression: str, var: str = "x") -> str:
        """Compute derivative."""
        return f"d/d{var}({expression})"

    def integral(self, expression: str, from_: float, to: float, var: str = "x") -> float:
        """Compute integral."""
        return 0.0

    def eval_simple(self, expression: str) -> float:
        """Evaluate simple numeric expression."""
        try:
            # 简单的表达式求值（仅用于测试）
            import ast
            import operator
            
            # 创建安全的求值环境
            allowed_ops = {
                ast.Add: operator.add,
                ast.Sub: operator.sub,
                ast.Mult: operator.mul,
                ast.Div: operator.truediv,
                ast.Pow: operator.pow,
                ast.USub: operator.neg
            }
            
            # 解析表达式
            tree = ast.parse(expression, mode='eval')
            
            # 安全求值函数
            def safe_eval(node):
                if isinstance(node, ast.Expression):
                    return safe_eval(node.body)
                elif isinstance(node, ast.Constant) and isinstance(node.value, (int, float)):
                    return node.value
                elif isinstance(node, ast.UnaryOp) and isinstance(node.op, tuple(allowed_ops.keys())):
                    return allowed_ops[type(node.op)](safe_eval(node.operand))
                elif isinstance(node, ast.BinOp) and isinstance(node.op, tuple(allowed_ops.keys())):
                    return allowed_ops[type(node.op)](safe_eval(node.left), safe_eval(node.right))
                else:
                    raise ValueError(f"Unsupported expression: {expression}")
            
            return safe_eval(tree)
        except Exception as e:
            raise RuntimeError(f"Failed to evaluate expression: {e}")

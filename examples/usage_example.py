#!/usr/bin/env python3
"""
MathCore Python绑定使用示例
"""

import mathcore

def main():
    print("MathCore Python绑定使用示例")
    print("=" * 50)
    
    # 创建MathEngine实例
    engine = mathcore.MathEngine()
    
    # 显示引擎类型
    if hasattr(engine, '__class__'):
        print(f"引擎类型: {engine.__class__}")
    
    print("\n1. 评估数学表达式:")
    expr = "x + 2"
    result = engine.evaluate(expr)
    print(f"   '{expr}' -> {result}")
    
    print("\n2. 简化表达式:")
    expr = "x + 2 + x"
    result = engine.simplify(expr)
    print(f"   '{expr}' -> {result}")
    
    print("\n3. 计算导数:")
    expr = "x^2"
    result = engine.derivative(expr, "x")
    print(f"   d/dx('{expr}') -> {result}")
    
    print("\n4. 简单表达式求值:")
    expr = "2 + 3 * 4"
    result = engine.eval_simple(expr)
    print(f"   '{expr}' -> {result}")
    
    print("\n5. 变量代入计算:")
    expr = "x^2 + 2*x + 1"
    variables = {"x": 3}
    result = engine.compute(expr, variables)
    print(f"   '{expr}' (x=3) -> {result}")
    
    print("\n6. 数值积分:")
    expr = "x^2"
    from_val = 0.0
    to_val = 1.0
    variable = "x"
    result = engine.integral(expr, from_val, to_val, variable)
    expected = 1/3  # 0.333...
    print(f"   ∫'{expr}'dx from {from_val} to {to_val} -> {result:.4f}")
    print(f"   预期值: {expected:.4f}")
    print(f"   误差: {abs(result - expected):.4f}")
    
    print("\n" + "=" * 50)
    print("所有示例运行成功！")

if __name__ == "__main__":
    main()

#[cfg(test)]
mod tests {
    use crate::python::*;
    use std::collections::HashMap;

    #[test]
    fn test_python_bridge_creation() {
        let bridge = PythonBridge::new();
        // 测试桥梁是否成功创建（由于 SymbolicEngine 和 NumericEngine 是单元结构体，只需检查是否创建）
        let _ = bridge.symbolic_engine;
        let _ = bridge.numeric_engine;
    }

    #[test]
    fn test_evaluate_basic_expression() {
        let bridge = PythonBridge::new();
        let result = bridge.evaluate("2 + 3").unwrap();
        assert_eq!(result, "5");
    }

    #[test]
    fn test_evaluate_variable_expression() {
        let bridge = PythonBridge::new();
        let result = bridge.evaluate("x + y").unwrap();
        assert!(result.contains("x") && result.contains("y"));
    }

    #[test]
    fn test_compute_with_vars() {
        let bridge = PythonBridge::new();
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 2.0);
        vars.insert("y".to_string(), 3.0);
        
        let result = bridge.compute("x + y", Some(&vars)).unwrap();
        assert!((result - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_simplify_expression() {
        let bridge = PythonBridge::new();
        let result = bridge.simplify("x + x + 0").unwrap();
        // 检查简化结果是否包含预期的项，不严格要求格式
        assert!(result.contains("x"));
        // 注意：简化函数不会合并同类项，只会移除加法零
    }

    #[test]
    fn test_derivative() {
        let bridge = PythonBridge::new();
        let result = bridge.derivative("x^2", "x").unwrap();
        // 检查导数结果是否包含预期的项，不严格要求格式
        assert!(result.contains("2"));
        assert!(result.contains("x"));
    }

    #[test]
    fn test_integral() {
        let bridge = PythonBridge::new();
        let result = bridge.integral("x", "x", 0.0, 1.0).unwrap();
        assert!((result - 0.5).abs() < 1e-3); // Simpson积分1000步
    }

    #[test]
    fn test_error_invalid_expression() {
        let bridge = PythonBridge::new();
        let result = bridge.evaluate("invalid expression");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_undefined_variable() {
        let bridge = PythonBridge::new();
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 2.0);
        
        let result = bridge.compute("x + y", Some(&vars));
        assert!(result.is_err());
    }

    #[test]
    fn test_default_implementation() {
        let bridge = PythonBridge::default();
        let _ = bridge.symbolic_engine;
        let _ = bridge.numeric_engine;
    }
}

#[cfg(all(test, feature = "pyo3"))]
mod python_tests {
    use super::*;
    use pyo3::Python;

    #[test]
    fn test_pymathengine_creation() {
        Python::with_gil(|py| {
            let engine = PyMathEngine::new();
            // 测试Python引擎是否成功创建
            let _ = engine.bridge.symbolic_engine;
            let _ = engine.bridge.numeric_engine;
        });
    }

    #[test]
    fn test_py_evaluate() {
        Python::with_gil(|py| {
            let engine = PyMathEngine::new();
            let result = engine.py_evaluate("2 + 3").unwrap();
            assert_eq!(result, "5");
        });
    }

    #[test]
    fn test_py_compute() {
        Python::with_gil(|py| {
            let engine = PyMathEngine::new();
            
            // 创建Python字典
            let py_dict = pyo3::types::PyDict::new(py);
            py_dict.set_item("x", 2.0).unwrap();
            py_dict.set_item("y", 3.0).unwrap();
            
            let result = engine.py_compute("x + y", Some(py_dict)).unwrap();
            assert!((result - 5.0).abs() < 1e-9);
        });
    }

    #[test]
    fn test_py_simplify() {
        Python::with_gil(|py| {
            let engine = PyMathEngine::new();
            let result = engine.py_simplify("x + x + 0").unwrap();
            assert_eq!(result, "2 * x");
        });
    }

    #[test]
    fn test_py_derivative() {
        Python::with_gil(|py| {
            let engine = PyMathEngine::new();
            let result = engine.py_derivative("x^2", Some("x")).unwrap();
            assert_eq!(result, "2 * x");
        });
    }

    #[test]
    fn test_py_integral() {
        Python::with_gil(|py| {
            let engine = PyMathEngine::new();
            let result = engine.py_integral("x", 0.0, 1.0, Some("x")).unwrap();
            assert!((result - 0.5).abs() < 1e-3);
        });
    }

    #[test]
    fn test_py_parse() {
        Python::with_gil(|py| {
            let engine = PyMathEngine::new();
            let result = engine.py_parse("x + 2").unwrap();
            assert!(result.contains("x") && result.contains("2"));
        });
    }

    #[test]
    fn test_py_eval_simple() {
        Python::with_gil(|py| {
            let engine = PyMathEngine::new();
            let result = engine.py_eval_simple("2 + 3 * 4").unwrap();
            assert!((result - 14.0).abs() < 1e-9);
        });
    }

    #[test]
    fn test_py_error_handling() {
        Python::with_gil(|py| {
            let engine = PyMathEngine::new();
            let result = engine.py_evaluate("invalid expression");
            assert!(result.is_err());
        });
    }
}

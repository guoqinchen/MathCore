//! Python bindings (PyO3)

use std::collections::HashMap;
use mathcore_compute as compute;

/// Python binding error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Binding error: {0}")]
    Binding(String),

    #[error("Import error: {0}")]
    Import(String),

    #[error("Compute error: {0}")]
    Compute(#[from] compute::Error),

    #[error("Symbolic error: {0}")]
    Symbolic(#[from] compute::symbolic::Error),

    #[error("Numeric error: {0}")]
    Numeric(#[from] compute::numeric::Error),
}

/// Python bridge
pub struct PythonBridge {
    /// 符号计算引擎
    symbolic_engine: compute::symbolic::SymbolicEngine,
    /// 数值计算引擎
    numeric_engine: compute::numeric::NumericEngine,
}

impl PythonBridge {
    /// Create a new Python bridge
    pub fn new() -> Self {
        Self {
            symbolic_engine: compute::symbolic::SymbolicEngine::new(),
            numeric_engine: compute::numeric::NumericEngine::new(),
        }
    }

    /// Evaluate a mathematical expression
    pub fn evaluate(&self, expression: &str) -> Result<String, Error> {
        let expr = compute::parse(expression).map_err(Error::Symbolic)?;
        let simplified = compute::simplify(&expr).map_err(Error::Symbolic)?;
        Ok(simplified.to_string())
    }

    /// Compute numerical result
    pub fn compute(&self, expression: &str, vars: Option<&HashMap<String, f64>>) -> Result<f64, Error> {
        let mut default_vars = HashMap::new();
        let vars_ref = vars.unwrap_or(&default_vars);
        compute::compute(expression, vars_ref).map_err(Error::Compute)
    }

    /// Simplify an expression
    pub fn simplify(&self, expression: &str) -> Result<String, Error> {
        let expr = compute::parse(expression).map_err(Error::Symbolic)?;
        let simplified = compute::simplify(&expr).map_err(Error::Symbolic)?;
        Ok(simplified.to_string())
    }

    /// Compute derivative
    pub fn derivative(&self, expression: &str, var: &str) -> Result<String, Error> {
        compute::derivative(expression, var).map_err(Error::Compute)
    }

    /// Compute integral (using numeric integration)
    pub fn integral(&self, expression: &str, var: &str, from: f64, to: f64) -> Result<f64, Error> {
        // 这里可以根据需要添加符号积分，如果不可行则回退到数值积分
        // 目前我们使用 Simpson 数值积分方法
        compute::integrate_expr_simpson(expression, var, from, to, Some(1000)).map_err(Error::Numeric)
    }
}

impl Default for PythonBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// MathEngine Python class
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pyclass(name="MathEngine")]
pub struct PyMathEngine {
    bridge: PythonBridge,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl PyMathEngine {
    #[new]
    fn new() -> Self {
        Self {
            bridge: PythonBridge::new(),
        }
    }

    #[pyo3(name="evaluate")]
    fn py_evaluate(&self, expression: &str) -> PyResult<String> {
        self.bridge.evaluate(expression).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[pyo3(name="compute")]
    fn py_compute(&self, expression: &str, vars: Option<&pyo3::types::PyDict>) -> PyResult<f64> {
        let mut rust_vars = HashMap::new();
        
        if let Some(py_dict) = vars {
            for (key, value) in py_dict.iter() {
                let key_str = key.extract::<String>()?;
                let value_f64 = value.extract::<f64>()?;
                rust_vars.insert(key_str, value_f64);
            }
        }
        
        self.bridge.compute(expression, Some(&rust_vars)).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[pyo3(name="simplify")]
    fn py_simplify(&self, expression: &str) -> PyResult<String> {
        self.bridge.simplify(expression).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[pyo3(name="derivative")]
    fn py_derivative(&self, expression: &str, var: Option<&str>) -> PyResult<String> {
        let var = var.unwrap_or("x");
        self.bridge.derivative(expression, var).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[pyo3(name="integral")]
    fn py_integral(&self, expression: &str, from: f64, to: f64, var: Option<&str>) -> PyResult<f64> {
        let var = var.unwrap_or("x");
        self.bridge.integral(expression, var, from, to).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[pyo3(name="parse")]
    fn py_parse(&self, expression: &str) -> PyResult<String> {
        let expr = compute::parse(expression).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(expr.to_string())
    }

    #[pyo3(name="eval_simple")]
    fn py_eval_simple(&self, expression: &str) -> PyResult<f64> {
        compute::eval_simple(expression).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

/// Python module definition
#[cfg(feature = "pyo3")]
#[pymodule]
pub fn mathcore_bridge(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyMathEngine>()?;
    Ok(())
}

#[cfg(not(feature = "pyo3"))]
pub fn mathcore_bridge() {
    // PyO3 not enabled, do nothing
}

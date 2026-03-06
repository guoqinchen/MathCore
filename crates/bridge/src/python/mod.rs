//! Python bindings (PyO3)

/// Python binding error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Binding error: {0}")]
    Binding(String),

    #[error("Import error: {0}")]
    Import(String),

    #[error("Compute error: {0}")]
    Compute(String),
}

/// Python bridge
pub struct PythonBridge;

impl PythonBridge {
    /// Create a new Python bridge
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a mathematical expression
    pub fn evaluate(&self, expression: &str) -> Result<String, Error> {
        Ok(expression.to_string())
    }

    /// Compute numerical result
    pub fn compute(&self, expression: &str) -> Result<String, Error> {
        Ok(expression.to_string())
    }

    /// Simplify an expression
    pub fn simplify(&self, expression: &str) -> Result<String, Error> {
        Ok(expression.to_string())
    }

    /// Compute derivative
    pub fn derivative(&self, expression: &str, var: &str) -> Result<String, Error> {
        Ok(format!("d/d{var}({expression})"))
    }

    /// Compute integral
    pub fn integral(&self, expression: &str, var: &str) -> Result<String, Error> {
        Ok(format!("∫{expression}d{var}"))
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
    fn py_compute(&self, expression: &str) -> PyResult<String> {
        self.bridge.compute(expression).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
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
    fn py_integral(&self, expression: &str, var: Option<&str>) -> PyResult<String> {
        let var = var.unwrap_or("x");
        self.bridge.integral(expression, var).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
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

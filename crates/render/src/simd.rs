//! SIMD module - Matrix operations with SIMD-like acceleration
//!
//! Provides high-performance matrix operations using manual loop unrolling
//! and cache-friendly access patterns.

use std::ops::{Add, Mul, Sub};

/// Matrix structure
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<T> {
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

/// Trait for scalar types
pub trait Scalar: Copy + Add<Output = Self> + Mul<Output = Self> + Sub<Output = Self> {
    fn zero() -> Self;
    fn one() -> Self;
}

impl Scalar for f32 {
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
}

impl Scalar for f64 {
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
}

impl Scalar for i32 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}

impl<T: Scalar> Matrix<T> {
    /// Create a new matrix
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![T::zero(); rows * cols],
            rows,
            cols,
        }
    }

    /// Create a matrix from flat data
    pub fn from_flat(rows: usize, cols: usize, data: Vec<T>) -> Self {
        assert_eq!(data.len(), rows * cols);
        Self { data, rows, cols }
    }

    /// Create an identity matrix
    pub fn identity(n: usize) -> Self {
        let mut m = Self::new(n, n);
        for i in 0..n {
            m[(i, i)] = T::one();
        }
        m
    }

    /// Get matrix dimensions
    pub fn dims(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    fn index(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    /// Get element at (row, col)
    pub fn get(&self, row: usize, col: usize) -> T {
        self.data[self.index(row, col)]
    }

    /// Get raw data reference
    pub fn data(&self) -> &[T] {
        &self.data
    }
}

/// Mutable indexing
impl<T: Scalar> std::ops::Index<(usize, usize)> for Matrix<T> {
    type Output = T;
    fn index(&self, (row, col): (usize, usize)) -> &T {
        &self.data[self.index(row, col)]
    }
}

impl<T: Scalar> std::ops::IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut T {
        let idx = self.index(row, col);
        &mut self.data[idx]
    }
}

/// Matrix multiplication using loop unrolling (f32)
impl Matrix<f32> {
    /// Multiply two matrices using optimized unrolled loops
    pub fn mul_simd(&self, other: &Matrix<f32>) -> Matrix<f32> {
        assert_eq!(self.cols, other.rows);

        let mut result = Matrix::new(self.rows, other.cols);
        let cols = self.cols;

        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut sum = 0.0f32;
                let mut k = 0;

                // Unroll by 4
                while k + 4 <= cols {
                    let a0 = self[(i, k)];
                    let a1 = self[(i, k + 1)];
                    let a2 = self[(i, k + 2)];
                    let a3 = self[(i, k + 3)];
                    let b0 = other[(k, j)];
                    let b1 = other[(k + 1, j)];
                    let b2 = other[(k + 2, j)];
                    let b3 = other[(k + 3, j)];
                    sum = sum + a0 * b0 + a1 * b1 + a2 * b2 + a3 * b3;
                    k += 4;
                }

                while k < cols {
                    sum = sum + self[(i, k)] * other[(k, j)];
                    k += 1;
                }

                result[(i, j)] = sum;
            }
        }
        result
    }
}

/// Matrix multiplication for f64
impl Matrix<f64> {
    /// Multiply two matrices using optimized unrolled loops
    pub fn mul_simd(&self, other: &Matrix<f64>) -> Matrix<f64> {
        assert_eq!(self.cols, other.rows);

        let mut result = Matrix::new(self.rows, other.cols);
        let cols = self.cols;

        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut sum = 0.0f64;
                let mut k = 0;

                while k + 4 <= cols {
                    let a0 = self[(i, k)];
                    let a1 = self[(i, k + 1)];
                    let a2 = self[(i, k + 2)];
                    let a3 = self[(i, k + 3)];
                    let b0 = other[(k, j)];
                    let b1 = other[(k + 1, j)];
                    let b2 = other[(k + 2, j)];
                    let b3 = other[(k + 3, j)];
                    sum = sum + a0 * b0 + a1 * b1 + a2 * b2 + a3 * b3;
                    k += 4;
                }

                while k < cols {
                    sum = sum + self[(i, k)] * other[(k, j)];
                    k += 1;
                }

                result[(i, j)] = sum;
            }
        }
        result
    }
}

/// Vector dot product with loop unrolling
pub fn dot_product_f32(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let len = a.len();
    let mut sum = 0.0f32;
    let mut i = 0;
    while i + 4 <= len {
        sum = sum + a[i] * b[i] + a[i + 1] * b[i + 1] + a[i + 2] * b[i + 2] + a[i + 3] * b[i + 3];
        i += 4;
    }
    while i < len {
        sum = sum + a[i] * b[i];
        i += 1;
    }
    sum
}

/// Vector dot product for f64
pub fn dot_product_f64(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    let len = a.len();
    let mut sum = 0.0f64;
    let mut i = 0;
    while i + 4 <= len {
        sum = sum + a[i] * b[i] + a[i + 1] * b[i + 1] + a[i + 2] * b[i + 2] + a[i + 3] * b[i + 3];
        i += 4;
    }
    while i < len {
        sum = sum + a[i] * b[i];
        i += 1;
    }
    sum
}

/// Vector add with loop unrolling
pub fn vec_add_f32(a: &[f32], b: &[f32]) -> Vec<f32> {
    assert_eq!(a.len(), b.len());
    let len = a.len();
    let mut result = vec![0.0; len];
    let mut i = 0;
    while i + 4 <= len {
        result[i] = a[i] + b[i];
        result[i + 1] = a[i + 1] + b[i + 1];
        result[i + 2] = a[i + 2] + b[i + 2];
        result[i + 3] = a[i + 3] + b[i + 3];
        i += 4;
    }
    while i < len {
        result[i] = a[i] + b[i];
        i += 1;
    }
    result
}

/// Vector scale with loop unrolling
pub fn vec_scale_f32(v: &[f32], scalar: f32) -> Vec<f32> {
    let len = v.len();
    let mut result = vec![0.0; len];
    let mut i = 0;
    while i + 4 <= len {
        result[i] = v[i] * scalar;
        result[i + 1] = v[i + 1] * scalar;
        result[i + 2] = v[i + 2] * scalar;
        result[i + 3] = v[i + 3] * scalar;
        i += 4;
    }
    while i < len {
        result[i] = v[i] * scalar;
        i += 1;
    }
    result
}

/// Benchmark matrix multiplication
pub fn benchmark_matmul(size: usize, iterations: usize) -> BenchmarkResult {
    use std::time::Instant;

    let a = Matrix::from_flat(
        size,
        size,
        (0..size * size).map(|i| (i as f32 * 0.1).sin()).collect(),
    );
    let b = Matrix::from_flat(
        size,
        size,
        (0..size * size).map(|i| (i as f32 * 0.2).cos()).collect(),
    );

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = a.mul_simd(&b);
    }
    let elapsed = start.elapsed();

    BenchmarkResult {
        iterations,
        size,
        total_ms: elapsed.as_millis() as f64,
        avg_ns_per_op: elapsed.as_nanos() as f64 / iterations as f64,
    }
}

#[derive(Debug)]
pub struct BenchmarkResult {
    pub iterations: usize,
    pub size: usize,
    pub total_ms: f64,
    pub avg_ns_per_op: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_creation() {
        let m: Matrix<f32> = Matrix::new(3, 3);
        assert_eq!(m.dims(), (3, 3));
        assert_eq!(m[(0, 0)], 0.0);
    }

    #[test]
    fn test_matrix_identity() {
        let m = Matrix::<f32>::identity(3);
        assert_eq!(m[(0, 0)], 1.0);
        assert_eq!(m[(1, 1)], 1.0);
    }

    #[test]
    fn test_matrix_mul_f32() {
        let a: Matrix<f32> = Matrix::from_flat(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let b: Matrix<f32> = Matrix::from_flat(3, 2, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let b = Matrix::from_flat(3, 2, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let c = a.mul_simd(&b);
        assert!((c[(0, 0)] - 22.0).abs() < 0.001);
    }

    #[test]
    fn test_dot_product_f32() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let result = dot_product_f32(&a, &b);
        assert!((result - 70.0).abs() < 0.001);
    }

    #[test]
    fn test_vec_add_f32() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let result = vec_add_f32(&a, &b);
        assert_eq!(result, vec![6.0, 8.0, 10.0, 12.0]);
    }

    #[test]
    fn test_vec_scale_f32() {
        let v = vec![1.0, 2.0, 3.0, 4.0];
        let result = vec_scale_f32(&v, 2.0);
        assert_eq!(result, vec![2.0, 4.0, 6.0, 8.0]);
    }
}

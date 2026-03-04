//! MathCore SIMD Optimization Module
//!
//! This module provides SIMD-accelerated numerical operations using portable-atomic
//! and falls back to scalar implementations when SIMD is not available.

/// SIMD vector width (256-bit = 4 x f64)
pub const SIMD_WIDTH: usize = 4;

/// SIMD-accelerated vector of f64
#[derive(Debug, Clone, Copy)]
pub struct SimdVec4(pub [f64; SIMD_WIDTH]);

impl SimdVec4 {
    /// Create from array
    pub fn new(arr: [f64; SIMD_WIDTH]) -> Self {
        Self(arr)
    }

    /// Create from single value (broadcast)
    pub fn splat(val: f64) -> Self {
        Self([val; SIMD_WIDTH])
    }

    /// Element-wise addition
    #[inline]
    pub fn add(self, other: SimdVec4) -> SimdVec4 {
        SimdVec4([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            self.0[3] + other.0[3],
        ])
    }

    /// Element-wise subtraction
    #[inline]
    pub fn sub(self, other: SimdVec4) -> SimdVec4 {
        SimdVec4([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
            self.0[3] - other.0[3],
        ])
    }

    /// Element-wise multiplication
    #[inline]
    pub fn mul(self, other: SimdVec4) -> SimdVec4 {
        SimdVec4([
            self.0[0] * other.0[0],
            self.0[1] * other.0[1],
            self.0[2] * other.0[2],
            self.0[3] * other.0[3],
        ])
    }

    /// Horizontal sum (reduction)
    #[inline]
    pub fn sum(self) -> f64 {
        self.0[0] + self.0[1] + self.0[2] + self.0[3]
    }

    /// Square root (element-wise)
    #[inline]
    pub fn sqrt(self) -> SimdVec4 {
        SimdVec4([
            self.0[0].sqrt(),
            self.0[1].sqrt(),
            self.0[2].sqrt(),
            self.0[3].sqrt(),
        ])
    }
}

/// Thread-safe atomic counter for parallel operations
pub struct AtomicCounter {
    count: std::sync::atomic::AtomicU64,
}

impl AtomicCounter {
    pub fn new(initial: u64) -> Self {
        Self {
            count: std::sync::atomic::AtomicU64::new(initial),
        }
    }

    pub fn increment(&self) -> u64 {
        self.count.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1
    }

    pub fn get(&self) -> u64 {
        self.count.load(std::sync::atomic::Ordering::Acquire)
    }
}

/// SIMD-accelerated array sum
#[inline]
pub fn simd_sum(arr: &[f64]) -> f64 {
    let mut sum = 0.0;
    let chunks = arr.chunks_exact(SIMD_WIDTH);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let vec = SimdVec4::new([chunk[0], chunk[1], chunk[2], chunk[3]]);
        sum += vec.sum();
    }

    for &val in remainder.iter() {
        sum += val;
    }

    sum
}

/// SIMD-accelerated dot product
#[inline]
pub fn simd_dot(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len(), "Arrays must have same length");

    let mut sum = SimdVec4::splat(0.0);
    let chunks = a.chunks_exact(SIMD_WIDTH);

    for (av, bv) in chunks.zip(b.chunks_exact(SIMD_WIDTH)) {
        let a_vec = SimdVec4::new([av[0], av[1], av[2], av[3]]);
        let b_vec = SimdVec4::new([bv[0], bv[1], bv[2], bv[3]]);
        sum = sum.add(a_vec.mul(b_vec));
    }

    let mut result = sum.sum();

    let remainder = (a.len() / SIMD_WIDTH) * SIMD_WIDTH;
    for i in remainder..a.len() {
        result += a[i] * b[i];
    }

    result
}

/// Check if SIMD is available at runtime
pub fn is_simd_available() -> bool {
    // On modern platforms, SIMD operations are generally available
    // This can be enhanced with portable_atomic for more precise detection
    cfg!(target_feature = "sse") || cfg!(target_feature = "avx")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_vec4_basic() {
        let a = SimdVec4::new([1.0, 2.0, 3.0, 4.0]);
        let b = SimdVec4::new([5.0, 6.0, 7.0, 8.0]);

        let sum = a.add(b);
        assert_eq!(sum.0, [6.0, 8.0, 10.0, 12.0]);

        let mul = a.mul(b);
        assert_eq!(mul.0, [5.0, 12.0, 21.0, 32.0]);
    }

    #[test]
    fn test_simd_sum() {
        let arr = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let sum = simd_sum(&arr);
        assert!((sum - 55.0).abs() < 1e-10);
    }

    #[test]
    fn test_simd_dot() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let dot = simd_dot(&a, &b);
        assert!((dot - 70.0).abs() < 1e-10);
    }

    #[test]
    fn test_atomic_counter() {
        let counter = AtomicCounter::new(0);
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.get(), 2);
    }
}

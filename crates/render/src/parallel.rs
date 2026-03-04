//! Parallel module - Multi-core parallel processing utilities
//!
//! Provides parallel processing using Rayon.

use rayon::prelude::*;

/// Parallel matrix-vector multiplication
pub fn par_matvec_mul(matrix: &[Vec<f64>], vector: &[f64]) -> Vec<f64> {
    matrix
        .par_iter()
        .map(|row| row.iter().zip(vector.iter()).map(|(a, b)| a * b).sum())
        .collect()
}

/// Parallel matrix-matrix multiplication  
pub fn par_matmul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let cols = b[0].len();
    a.par_iter()
        .map(|row| {
            (0..cols)
                .map(|j| row.iter().zip(b.iter()).map(|(ai, bj)| ai * bj[j]).sum())
                .collect()
        })
        .collect()
}

/// Parallel vector sum
pub fn par_sum(data: &[f64]) -> f64 {
    data.par_iter().fold(|| 0.0, |a, b| a + b).sum()
}

/// Parallel vector product
pub fn par_product(data: &[f64]) -> f64 {
    data.par_iter().fold(|| 1.0, |a, b| a * b).product()
}

/// Thread pool configuration
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    pub num_threads: Option<usize>,
    pub thread_name: Option<String>,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            num_threads: None,
            thread_name: None,
        }
    }
}

/// Initialize custom thread pool
pub fn init_thread_pool(config: ThreadPoolConfig) -> rayon::ThreadPool {
    let mut builder = rayon::ThreadPoolBuilder::new();

    if let Some(n) = config.num_threads {
        builder = builder.num_threads(n);
    }

    if let Some(name) = config.thread_name {
        builder = builder.thread_name(move |i| format!("{}_{}", name, i));
    }

    builder.build().unwrap()
}

/// Benchmark parallel vs sequential
pub fn benchmark_parallel(data: &[f64], iterations: usize) -> BenchmarkResult {
    use std::time::Instant;

    // Sequential
    let start = Instant::now();
    for _ in 0..iterations {
        let _: f64 = data.iter().sum();
    }
    let sequential_ns = start.elapsed().as_nanos() as f64 / iterations as f64;

    // Parallel
    let start = Instant::now();
    for _ in 0..iterations {
        let _: f64 = par_sum(data);
    }
    let parallel_ns = start.elapsed().as_nanos() as f64 / iterations as f64;

    BenchmarkResult {
        sequential_ns,
        parallel_ns,
        speedup: sequential_ns / parallel_ns,
    }
}

#[derive(Debug)]
pub struct BenchmarkResult {
    pub sequential_ns: f64,
    pub parallel_ns: f64,
    pub speedup: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_par_matvec_mul() {
        let matrix = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let vector = vec![5.0, 6.0];
        let result = par_matvec_mul(&matrix, &vector);
        assert!((result[0] - 17.0).abs() < 0.001);
    }

    #[test]
    fn test_par_sum() {
        let data: Vec<f64> = (0..1000).map(|i| i as f64).collect();
        let sum = par_sum(&data);
        assert!((sum - 499500.0).abs() < 0.001);
    }

    #[test]
    fn test_benchmark_parallel() {
        let data: Vec<f64> = (0..10000).map(|i| i as f64 * 0.1).collect();
        let result = benchmark_parallel(&data, 100);
        println!(
            "Seq: {:.0}ns, Par: {:.0}ns, Speedup: {:.2}x",
            result.sequential_ns, result.parallel_ns, result.speedup
        );
        assert!(result.speedup > 0.1);
    }
}

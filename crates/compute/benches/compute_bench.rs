//! MathCore Compute Benchmarks
//!
//! Run with: cargo bench --package mathcore-compute

use criterion::{criterion_group, criterion_main, Criterion};
use mathcore_compute::{differentiate, evaluate, parse, simplify, Expr};
use std::collections::HashMap;

// Symbolic parsing benchmarks

pub fn symbolic_parse_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("symbolic_parse");

    group.bench_function("simple_addition", |b| b.iter(|| parse("x + y").unwrap()));

    group.bench_function("polynomial", |b| b.iter(|| parse("x^2 + 2*x + 1").unwrap()));

    group.bench_function("complex_expression", |b| {
        b.iter(|| parse("sin(x) * cos(y) + sqrt(z^2 + 1)").unwrap())
    });

    group.finish();
}

pub fn symbolic_simplify(c: &mut Criterion) {
    let mut group = c.benchmark_group("symbolic_simplify");

    let expr = parse("x + 0").unwrap();
    group.bench_function("add_zero", |b| b.iter(|| simplify(&expr).unwrap()));

    let expr = parse("x * 1").unwrap();
    group.bench_function("mult_one", |b| b.iter(|| simplify(&expr).unwrap()));

    let expr = parse("x * 0").unwrap();
    group.bench_function("mult_zero", |b| b.iter(|| simplify(&expr).unwrap()));

    let expr = parse("(x + y) + z").unwrap();
    group.bench_function("nested_add", |b| b.iter(|| simplify(&expr).unwrap()));

    group.finish();
}

pub fn symbolic_differentiate(c: &mut Criterion) {
    let mut group = c.benchmark_group("symbolic_differentiate");

    let expr = parse("x").unwrap();
    group.bench_function("variable", |b| {
        b.iter(|| differentiate(&expr, "x").unwrap())
    });

    let expr = parse("x^2").unwrap();
    group.bench_function("power", |b| b.iter(|| differentiate(&expr, "x").unwrap()));

    let expr = parse("x^2 + 2*x + 1").unwrap();
    group.bench_function("polynomial", |b| {
        b.iter(|| differentiate(&expr, "x").unwrap())
    });

    let expr = parse("sin(x)").unwrap();
    group.bench_function("trig", |b| b.iter(|| differentiate(&expr, "x").unwrap()));

    group.finish();
}

pub fn symbolic_evaluate(c: &mut Criterion) {
    let mut group = c.benchmark_group("symbolic_evaluate");

    let expr = parse("x^2 + 2*x + 1").unwrap();
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), 3.0);

    group.bench_function("polynomial", |b| b.iter(|| evaluate(&expr, &vars).unwrap()));

    let expr = parse("sin(x) + cos(x)").unwrap();
    group.bench_function("trig", |b| b.iter(|| evaluate(&expr, &vars).unwrap()));

    group.finish();
}

// Numeric evaluation benchmarks

pub fn numeric_eval(c: &mut Criterion) {
    let mut group = c.benchmark_group("numeric_eval");

    group.bench_function("simple_arithmetic", |b| {
        b.iter(|| mathcore_compute::numeric::eval_simple("2 + 3 * 4").unwrap())
    });

    group.bench_function("power", |b| {
        b.iter(|| mathcore_compute::numeric::eval_simple("2^10").unwrap())
    });

    group.bench_function("trig", |b| {
        b.iter(|| mathcore_compute::numeric::eval_simple("sin(0.5) + cos(0.5)").unwrap())
    });

    group.finish();
}

pub fn numeric_differentiate(c: &mut Criterion) {
    let mut group = c.benchmark_group("numeric_differentiate");

    let f = |x: f64| Ok(x * x);
    group.bench_function("quadratic", |b| {
        b.iter(|| mathcore_compute::numeric::differentiate(f, 2.0, None).unwrap())
    });

    group.finish();
}

pub fn numeric_integrate(c: &mut Criterion) {
    let mut group = c.benchmark_group("numeric_integrate");

    let f = |x: f64| Ok(x * x);
    group.bench_function("simpson_quadratic", |b| {
        b.iter(|| mathcore_compute::numeric::integrate_simpson(f, 0.0, 1.0, Some(100)).unwrap())
    });

    group.finish();
}

pub fn numeric_solve(c: &mut Criterion) {
    let mut group = c.benchmark_group("numeric_solve");

    let f = |x: f64| Ok(x * x - 4.0);
    group.bench_function("bisection", |b| {
        b.iter(|| mathcore_compute::numeric::solve_bisection(f, 0.0, 3.0, None, None).unwrap())
    });

    let f = |x: f64| Ok(x * x - 4.0);
    let df = |x: f64| Ok(2.0 * x);
    group.bench_function("newton", |b| {
        b.iter(|| mathcore_compute::numeric::solve_newton(f, df, 3.0, None, None).unwrap())
    });

    group.finish();
}

criterion_group!(
    benches,
    symbolic_parse_simple,
    symbolic_simplify,
    symbolic_differentiate,
    symbolic_evaluate,
    numeric_eval,
    numeric_differentiate,
    numeric_integrate,
    numeric_solve
);
criterion_main!(benches);

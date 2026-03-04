//! MathCore Protocol Benchmarks
//!
//! Run with: cargo bench --package mathcore-kernel

use criterion::{criterion_group, criterion_main, Criterion};
use mathcore_kernel::protocol::*;

// Protocol serialization benchmarks

pub fn protocol_serialize_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_serialize");

    let msg = ProtocolMessage::new(
        MsgPayload::Compute(ComputeRequest {
            expression: "x + y".to_string(),
            params: ComputeParams::default(),
        }),
        1,
    );

    group.bench_function("small_expression", |b| b.iter(|| msg.to_msgpack().unwrap()));

    group.finish();
}

pub fn protocol_serialize_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_serialize_medium");

    let msg = ProtocolMessage::new(
        MsgPayload::Compute(ComputeRequest {
            expression: "sin(x) * cos(y) + tan(z) / sqrt(x^2 + y^2)".to_string(),
            params: ComputeParams {
                precision: Some(256),
                timeout_ms: Some(5000),
                simplify: Some(true),
                options: Some(
                    [
                        ("cache".to_string(), "true".to_string()),
                        ("parallel".to_string(), "true".to_string()),
                    ]
                    .into_iter()
                    .collect(),
                ),
            },
        }),
        1,
    );

    group.bench_function("medium_expression", |b| {
        b.iter(|| msg.to_msgpack().unwrap())
    });

    group.finish();
}

pub fn protocol_serialize_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_serialize_large");

    let msg = ProtocolMessage::new(
        MsgPayload::ValidateResponse(ValidateResponse {
            valid: false,
            errors: (0..100)
                .map(|i| ValidationError {
                    message: format!("Error {}: Something went wrong with validation", i),
                    position: Some(i * 10),
                    severity: if i % 3 == 0 {
                        ValidationSeverity::Warning
                    } else {
                        ValidationSeverity::Error
                    },
                })
                .collect(),
        }),
        1,
    );

    group.bench_function("large_response", |b| b.iter(|| msg.to_msgpack().unwrap()));

    group.finish();
}

pub fn protocol_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_deserialize");

    let bytes = {
        let msg = ProtocolMessage::new(
            MsgPayload::Compute(ComputeRequest {
                expression: "x + y".to_string(),
                params: ComputeParams::default(),
            }),
            1,
        );
        msg.to_msgpack().unwrap()
    };

    group.bench_function("small_message", |b| {
        b.iter(|| ProtocolMessage::from_msgpack(&bytes).unwrap())
    });

    group.finish();
}

pub fn protocol_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_roundtrip");

    let msg = ProtocolMessage::new(
        MsgPayload::Compute(ComputeRequest {
            expression: "x^2 + 2*x + 1".to_string(),
            params: ComputeParams::default(),
        }),
        1,
    );

    group.bench_function("compute_roundtrip", |b| {
        b.iter(|| {
            let bytes = msg.to_msgpack().unwrap();
            ProtocolMessage::from_msgpack(&bytes).unwrap()
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    protocol_serialize_small,
    protocol_serialize_medium,
    protocol_serialize_large,
    protocol_deserialize,
    protocol_roundtrip
);
criterion_main!(benches);

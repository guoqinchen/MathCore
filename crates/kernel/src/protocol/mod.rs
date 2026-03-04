//! MessagePack Protocol v6 for MathCore

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u8 = 6;
pub const SUPPORTED_VERSIONS: &[u8] = &[6, 5, 4];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgHeader {
    pub version: u8,
    pub msg_type: MsgType,
    pub request_id: u64,
    pub timestamp: u64,
}

impl MsgHeader {
    pub fn new(msg_type: MsgType, request_id: u64) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            msg_type,
            request_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    pub fn is_version_supported(&self) -> bool {
        SUPPORTED_VERSIONS.contains(&self.version)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MsgType {
    Compute = 0x01,
    ComputeResponse = 0x02,
    Validate = 0x03,
    ValidateResponse = 0x04,
    Subscribe = 0x05,
    Unsubscribe = 0x06,
    Heartbeat = 0x07,
    Error = 0xFF,
}

impl MsgType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Compute => "Compute",
            Self::ComputeResponse => "ComputeResponse",
            Self::Validate => "Validate",
            Self::ValidateResponse => "ValidateResponse",
            Self::Subscribe => "Subscribe",
            Self::Unsubscribe => "Unsubscribe",
            Self::Heartbeat => "Heartbeat",
            Self::Error => "Error",
        }
    }
}

impl std::fmt::Display for MsgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequest {
    pub expression: String,
    pub params: ComputeParams,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComputeParams {
    pub precision: Option<u32>,
    pub timeout_ms: Option<u64>,
    pub simplify: Option<bool>,
    pub options: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResponse {
    pub result: String,
    pub result_type: String,
    pub success: bool,
    pub error: Option<String>,
    pub compute_time_us: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub expression: String,
    pub rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub name: String,
    pub params: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub message: String,
    pub position: Option<usize>,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic(pub String);

impl Topic {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl AsRef<str> for Topic {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub client_id: String,
    pub last_msg_id: Option<u64>,
    pub capabilities: Option<ClientCapabilities>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub compression: bool,
    pub batching: bool,
    pub max_message_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
    pub request_id: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    UnsupportedVersion,
    InvalidFormat,
    ComputeError,
    ValidationFailed,
    Timeout,
    InternalError,
    NotFound,
    PermissionDenied,
}

impl ErrorCode {
    pub fn code(&self) -> u16 {
        match self {
            Self::UnsupportedVersion => 1001,
            Self::InvalidFormat => 1002,
            Self::ComputeError => 2001,
            Self::ValidationFailed => 2002,
            Self::Timeout => 3001,
            Self::InternalError => 5001,
            Self::NotFound => 4001,
            Self::PermissionDenied => 4003,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MsgPayload {
    Compute(ComputeRequest),
    ComputeResponse(ComputeResponse),
    Validate(ValidateRequest),
    ValidateResponse(ValidateResponse),
    Subscribe(Topic),
    Unsubscribe(Topic),
    Heartbeat(Heartbeat),
    Error(ErrorResponse),
}

impl MsgPayload {
    pub fn msg_type(&self) -> MsgType {
        match self {
            Self::Compute(_) => MsgType::Compute,
            Self::ComputeResponse(_) => MsgType::ComputeResponse,
            Self::Validate(_) => MsgType::Validate,
            Self::ValidateResponse(_) => MsgType::ValidateResponse,
            Self::Subscribe(_) => MsgType::Subscribe,
            Self::Unsubscribe(_) => MsgType::Unsubscribe,
            Self::Heartbeat(_) => MsgType::Heartbeat,
            Self::Error(_) => MsgType::Error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub header: MsgHeader,
    pub payload: MsgPayload,
}

impl ProtocolMessage {
    pub fn new(payload: MsgPayload, request_id: u64) -> Self {
        Self {
            header: MsgHeader::new(payload.msg_type(), request_id),
            payload,
        }
    }

    pub fn to_msgpack(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::with_capacity(256);
        rmp_serde::encode::write(&mut buf, self).map_err(ProtocolError::EncodeError)?;
        Ok(buf)
    }

    pub fn from_msgpack(data: &[u8]) -> Result<Self, ProtocolError> {
        rmp_serde::decode::from_slice(data).map_err(ProtocolError::DecodeError)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Encode error: {0}")]
    EncodeError(#[from] rmp_serde::encode::Error),

    #[error("Decode error: {0}")]
    DecodeError(#[from] rmp_serde::decode::Error),

    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(u8),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionNegotiation {
    pub client_versions: Vec<u8>,
    pub server_versions: Vec<u8>,
    pub negotiated_version: Option<u8>,
}

impl VersionNegotiation {
    pub fn request(client_versions: Vec<u8>) -> Self {
        Self {
            client_versions,
            server_versions: SUPPORTED_VERSIONS.to_vec(),
            negotiated_version: None,
        }
    }

    pub fn negotiate(&mut self) -> Option<u8> {
        let mut common: Vec<u8> = self
            .client_versions
            .iter()
            .filter(|v| SUPPORTED_VERSIONS.contains(v))
            .cloned()
            .collect();
        common.sort_by(|a, b| b.cmp(a));

        self.negotiated_version = common.first().copied();
        self.negotiated_version
    }

    pub fn is_compatible(version: u8) -> bool {
        SUPPORTED_VERSIONS.contains(&version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msg_header_new() {
        let header = MsgHeader::new(MsgType::Compute, 123);
        assert_eq!(header.version, PROTOCOL_VERSION);
        assert_eq!(header.msg_type, MsgType::Compute);
        assert_eq!(header.request_id, 123);
    }

    #[test]
    fn test_msg_type_name() {
        assert_eq!(MsgType::Compute.name(), "Compute");
        assert_eq!(MsgType::Heartbeat.name(), "Heartbeat");
    }

    #[test]
    fn test_protocol_message_serde() {
        let msg = ProtocolMessage::new(
            MsgPayload::Compute(ComputeRequest {
                expression: "x + y".to_string(),
                params: ComputeParams::default(),
            }),
            1,
        );

        let encoded = msg.to_msgpack().unwrap();
        let decoded = ProtocolMessage::from_msgpack(&encoded).unwrap();

        assert_eq!(decoded.header.request_id, 1);
        assert_eq!(decoded.header.msg_type, MsgType::Compute);

        if let MsgPayload::Compute(req) = decoded.payload {
            assert_eq!(req.expression, "x + y");
        } else {
            panic!("Expected Compute payload");
        }
    }

    #[test]
    fn test_compute_roundtrip() {
        let original = ProtocolMessage::new(
            MsgPayload::ComputeResponse(ComputeResponse {
                result: "42".to_string(),
                result_type: "Integer".to_string(),
                success: true,
                error: None,
                compute_time_us: Some(100),
            }),
            42,
        );

        let bytes = original.to_msgpack().unwrap();
        let decoded = ProtocolMessage::from_msgpack(&bytes).unwrap();

        if let MsgPayload::ComputeResponse(resp) = decoded.payload {
            assert!(resp.success);
            assert_eq!(resp.result, "42");
            assert_eq!(resp.compute_time_us, Some(100));
        }
    }

    #[test]
    fn test_error_response_roundtrip() {
        let original = ProtocolMessage::new(
            MsgPayload::Error(ErrorResponse {
                code: ErrorCode::ComputeError,
                message: "Division by zero".to_string(),
                request_id: Some(123),
            }),
            999,
        );

        let bytes = original.to_msgpack().unwrap();
        let decoded = ProtocolMessage::from_msgpack(&bytes).unwrap();

        if let MsgPayload::Error(err) = decoded.payload {
            assert_eq!(err.code, ErrorCode::ComputeError);
            assert_eq!(err.request_id, Some(123));
        }
    }

    #[test]
    fn test_version_negotiation() {
        let mut negotiation = VersionNegotiation::request(vec![6, 5, 4]);
        let version = negotiation.negotiate();
        assert_eq!(version, Some(6));
        assert!(VersionNegotiation::is_compatible(6));
        assert!(!VersionNegotiation::is_compatible(99));
    }

    #[test]
    fn test_heartbeat_roundtrip() {
        let original = ProtocolMessage::new(
            MsgPayload::Heartbeat(Heartbeat {
                client_id: "test-client".to_string(),
                last_msg_id: Some(42),
                capabilities: Some(ClientCapabilities {
                    compression: true,
                    batching: false,
                    max_message_size: Some(1024 * 1024),
                }),
            }),
            1,
        );

        let bytes = original.to_msgpack().unwrap();
        let decoded = ProtocolMessage::from_msgpack(&bytes).unwrap();

        if let MsgPayload::Heartbeat(hb) = decoded.payload {
            assert_eq!(hb.client_id, "test-client");
            assert_eq!(hb.last_msg_id, Some(42));
            assert!(hb.capabilities.unwrap().compression);
        }
    }

    #[test]
    fn test_validate_roundtrip() {
        let original = ProtocolMessage::new(
            MsgPayload::Validate(ValidateRequest {
                expression: "x +".to_string(),
                rules: vec![ValidationRule {
                    name: "syntax".to_string(),
                    params: None,
                }],
            }),
            5,
        );

        let bytes = original.to_msgpack().unwrap();
        let decoded = ProtocolMessage::from_msgpack(&bytes).unwrap();

        if let MsgPayload::Validate(req) = decoded.payload {
            assert_eq!(req.expression, "x +");
            assert_eq!(req.rules.len(), 1);
        }
    }

    #[test]
    fn test_subscribe_unsubscribe() {
        let sub_msg = ProtocolMessage::new(MsgPayload::Subscribe(Topic::new("prices")), 1);
        let sub_bytes = sub_msg.to_msgpack().unwrap();
        let sub_decoded = ProtocolMessage::from_msgpack(&sub_bytes).unwrap();
        assert!(matches!(sub_decoded.payload, MsgPayload::Subscribe(t) if t.0 == "prices"));

        let unsub_msg = ProtocolMessage::new(MsgPayload::Unsubscribe(Topic::new("prices")), 2);
        let unsub_bytes = unsub_msg.to_msgpack().unwrap();
        let unsub_decoded = ProtocolMessage::from_msgpack(&unsub_bytes).unwrap();
        assert!(matches!(unsub_decoded.payload, MsgPayload::Unsubscribe(t) if t.0 == "prices"));
    }

    #[test]
    fn test_performance_target() {
        let msg = ProtocolMessage::new(
            MsgPayload::Compute(ComputeRequest {
                expression: "x + y * z".to_string(),
                params: ComputeParams::default(),
            }),
            1,
        );

        let iterations = 1000;
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let bytes = msg.to_msgpack().unwrap();
            let _ = ProtocolMessage::from_msgpack(&bytes).unwrap();
        }
        let elapsed = start.elapsed();

        let per_iter = elapsed.as_nanos() as f64 / iterations as f64;
        println!(
            "Average time per round-trip: {:.0} ns ({:.3} ms)",
            per_iter,
            per_iter / 1000.0
        );

        assert!(
            per_iter < 1_000_000.0,
            "Performance target not met: {} ns",
            per_iter
        );
    }

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, 6);
        assert!(SUPPORTED_VERSIONS.contains(&6));
    }

    #[test]
    fn test_msg_header_version_supported() {
        let header = MsgHeader::new(MsgType::Compute, 1);
        assert!(header.is_version_supported());
    }

    #[test]
    fn test_msg_header_version_not_supported() {
        let mut header = MsgHeader::new(MsgType::Compute, 1);
        header.version = 99;
        assert!(!header.is_version_supported());
    }

    #[test]
    fn test_msg_type_display() {
        assert_eq!(format!("{}", MsgType::Compute), "Compute");
        assert_eq!(format!("{}", MsgType::Heartbeat), "Heartbeat");
    }

    #[test]
    fn test_compute_params_default() {
        let params = ComputeParams::default();
        assert!(params.precision.is_none());
        assert!(params.timeout_ms.is_none());
        assert!(params.simplify.is_none());
    }

    #[test]
    fn test_compute_params_with_values() {
        let params = ComputeParams {
            precision: Some(256),
            timeout_ms: Some(5000),
            simplify: Some(true),
            options: Some([("key".to_string(), "value".to_string())].into_iter().collect()),
        };
        assert_eq!(params.precision, Some(256));
        assert_eq!(params.timeout_ms, Some(5000));
    }

    #[test]
    fn test_compute_response() {
        let response = ComputeResponse {
            result: "42".to_string(),
            result_type: "Integer".to_string(),
            success: true,
            error: None,
            compute_time_us: Some(1000),
        };
        assert!(response.success);
        assert_eq!(response.result, "42");
    }

    #[test]
    fn test_compute_response_with_error() {
        let response = ComputeResponse {
            result: String::new(),
            result_type: "Error".to_string(),
            success: false,
            error: Some("Division by zero".to_string()),
            compute_time_us: Some(500),
        };
        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[test]
    fn test_validation_error() {
        let error = ValidationError {
            message: "Unexpected token".to_string(),
            position: Some(5),
            severity: ValidationSeverity::Error,
        };
        assert_eq!(error.position, Some(5));
        assert_eq!(error.severity, ValidationSeverity::Error);
    }

    #[test]
    fn test_validation_error_warning() {
        let error = ValidationError {
            message: "Suspicious pattern".to_string(),
            position: None,
            severity: ValidationSeverity::Warning,
        };
        assert_eq!(error.severity, ValidationSeverity::Warning);
    }

    #[test]
    fn test_validation_error_info() {
        let error = ValidationError {
            message: "Info message".to_string(),
            position: Some(10),
            severity: ValidationSeverity::Info,
        };
        assert_eq!(error.severity, ValidationSeverity::Info);
    }

    #[test]
    fn test_validation_response() {
        let response = ValidateResponse {
            valid: true,
            errors: vec![],
        };
        assert!(response.valid);
        assert!(response.errors.is_empty());
    }

    #[test]
    fn test_validation_response_invalid() {
        let response = ValidateResponse {
            valid: false,
            errors: vec![
                ValidationError {
                    message: "Error 1".to_string(),
                    position: Some(1),
                    severity: ValidationSeverity::Error,
                },
                ValidationError {
                    message: "Error 2".to_string(),
                    position: Some(5),
                    severity: ValidationSeverity::Warning,
                },
            ],
        };
        assert!(!response.valid);
        assert_eq!(response.errors.len(), 2);
    }

    #[test]
    fn test_error_code_values() {
        assert_eq!(ErrorCode::UnsupportedVersion.code(), 1001);
        assert_eq!(ErrorCode::InvalidFormat.code(), 1002);
        assert_eq!(ErrorCode::ComputeError.code(), 2001);
        assert_eq!(ErrorCode::ValidationFailed.code(), 2002);
        assert_eq!(ErrorCode::Timeout.code(), 3001);
        assert_eq!(ErrorCode::InternalError.code(), 5001);
        assert_eq!(ErrorCode::NotFound.code(), 4001);
        assert_eq!(ErrorCode::PermissionDenied.code(), 4003);
    }

    #[test]
    fn test_version_negotiation_no_common() {
        let mut negotiation = VersionNegotiation::request(vec![99, 98, 97]);
        let version = negotiation.negotiate();
        assert_eq!(version, None);
    }

    #[test]
    fn test_version_negotiation_partial() {
        let mut negotiation = VersionNegotiation::request(vec![5, 4, 3]);
        let version = negotiation.negotiate();
        assert_eq!(version, Some(5));
    }

    #[test]
    fn test_client_capabilities_default() {
        let caps = ClientCapabilities::default();
        assert!(!caps.compression);
        assert!(!caps.batching);
        assert!(caps.max_message_size.is_none());
    }

    #[test]
    fn test_protocol_message_new() {
        let msg = ProtocolMessage::new(
            MsgPayload::Compute(ComputeRequest {
                expression: "x".to_string(),
                params: ComputeParams::default(),
            }),
            42,
        );
        assert_eq!(msg.header.request_id, 42);
        assert_eq!(msg.header.msg_type, MsgType::Compute);
    }

    #[test]
    fn test_protocol_error_encode_decode() {
        let msg = ProtocolMessage::new(
            MsgPayload::Error(ErrorResponse {
                code: ErrorCode::Timeout,
                message: "Request timed out".to_string(),
                request_id: Some(123),
            }),
            999,
        );

        let bytes = msg.to_msgpack().unwrap();
        let decoded = ProtocolMessage::from_msgpack(&bytes).unwrap();

        if let MsgPayload::Error(err) = decoded.payload {
            assert_eq!(err.code, ErrorCode::Timeout);
            assert_eq!(err.message, "Request timed out");
            assert_eq!(err.request_id, Some(123));
        } else {
            panic!("Expected Error payload");
        }
    }
    }

    #[test]
    fn test_performance_target() {
    fn test_performance_target() {
        let msg = ProtocolMessage::new(
            MsgPayload::Compute(ComputeRequest {
                expression: "x + y * z".to_string(),
                params: ComputeParams::default(),
            }),
            1,
        );

        let iterations = 1000;
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let bytes = msg.to_msgpack().unwrap();
            let _ = ProtocolMessage::from_msgpack(&bytes).unwrap();
        }
        let elapsed = start.elapsed();

        let per_iter = elapsed.as_nanos() as f64 / iterations as f64;
        println!(
            "Average time per round-trip: {:.0} ns ({:.3} ms)",
            per_iter,
            per_iter / 1000.0
        );

        assert!(
            per_iter < 1_000_000.0,
            "Performance target not met: {} ns",
            per_iter
        );
    }
}

#[cfg(feature = "bench")]
mod benchmarks {
    use super::*;
    use criterion::{criterion_group, criterion_main, Criterion};

    pub fn serialize_benchmark(c: &mut Criterion) {
        let mut group = c.benchmark_group("msgpack_serialize");

        let small_msg = ProtocolMessage::new(
            MsgPayload::Compute(ComputeRequest {
                expression: "x + y".to_string(),
                params: ComputeParams::default(),
            }),
            1,
        );

        group.bench_function("small_message", |b| {
            b.iter(|| small_msg.to_msgpack().unwrap())
        });

        let medium_msg = ProtocolMessage::new(
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

        group.bench_function("medium_message", |b| {
            b.iter(|| medium_msg.to_msgpack().unwrap())
        });

        let large_msg = ProtocolMessage::new(
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

        group.bench_function("large_message", |b| {
            b.iter(|| large_msg.to_msgpack().unwrap())
        });

        group.finish();
    }

    pub fn deserialize_benchmark(c: &mut Criterion) {
        let mut group = c.benchmark_group("msgpack_deserialize");

        let small_bytes = {
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
            b.iter(|| ProtocolMessage::from_msgpack(&small_bytes).unwrap())
        });

        let medium_bytes = {
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
            msg.to_msgpack().unwrap()
        };

        group.bench_function("medium_message", |b| {
            b.iter(|| ProtocolMessage::from_msgpack(&medium_bytes).unwrap())
        });

        let large_bytes = {
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
            msg.to_msgpack().unwrap()
        };

        group.bench_function("large_message", |b| {
            b.iter(|| ProtocolMessage::from_msgpack(&large_bytes).unwrap())
        });

        group.finish();
    }

    pub fn roundtrip_benchmark(c: &mut Criterion) {
        let mut group = c.benchmark_group("msgpack_roundtrip");

        let msg = ProtocolMessage::new(
            MsgPayload::Compute(ComputeRequest {
                expression: "x + y".to_string(),
                params: ComputeParams::default(),
            }),
            1,
        );

        group.bench_function("compute_request", |b| {
            b.iter(|| {
                let bytes = msg.to_msgpack().unwrap();
                ProtocolMessage::from_msgpack(&bytes).unwrap()
            })
        });

        group.finish();
    }

    criterion_group!(
        benches,
        serialize_benchmark,
        deserialize_benchmark,
        roundtrip_benchmark
    );
    criterion_main!(benches);
}

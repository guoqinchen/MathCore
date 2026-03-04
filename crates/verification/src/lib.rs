//! MathCore Verification Mesh
//!
//! Three-level verification system:
//! - Level 1: Numerical consistency (random sampling)
//! - Level 2: Inverse operation verification (integral ↔ derivative)
//! - Level 3: Symbolic equivalence verification

pub mod mesh;
pub mod levels;
pub mod certificate;
pub mod config;

pub use mesh::VerificationMesh;
pub use levels::{Level1Numerical, Level2Inverse, Level3Symbolic};
pub use certificate::VerificationCertificate;
pub use config::{Confidence, VerificationLevel, VerificationStatus, VerificationError};
pub use mesh::MeshConfig;

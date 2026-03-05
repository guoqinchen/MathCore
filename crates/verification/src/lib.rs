//! MathCore Verification Mesh
//!
//! Three-level verification system:
//! - Level 1: Numerical consistency (random sampling)
//! - Level 2: Inverse operation verification (integral ↔ derivative)
//! - Level 3: Symbolic equivalence verification

pub mod certificate;
pub mod config;
pub mod levels;
pub mod mesh;

pub use certificate::VerificationCertificate;
pub use config::{Confidence, VerificationError, VerificationLevel, VerificationStatus};
pub use levels::{Level1Numerical, Level2Inverse, Level3Symbolic};
pub use mesh::MeshConfig;
pub use mesh::VerificationMesh;

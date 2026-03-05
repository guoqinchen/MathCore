//! Three-level verification implementations
//!
//! - Level 1: Numerical consistency (random sampling)
//! - Level 2: Inverse operation verification (integral ↔ derivative)
//! - Level 3: Symbolic equivalence verification

mod level1;
mod level2;
mod level3;

pub use level1::Level1Numerical;
pub use level2::Level2Inverse;
pub use level3::Level3Symbolic;

use crate::certificate::VerificationCertificate;
use crate::config::{VerificationError, VerificationLevel, VerificationStatus};

pub trait VerificationStrategy {
    fn verify(&self, expression: &str) -> Result<VerificationCertificate, VerificationError>;
    fn level(&self) -> VerificationLevel;
}

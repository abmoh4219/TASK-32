//! Pure Rust helper logic that lives in the frontend crate but does not depend
//! on the DOM. Tested via `frontend/tests/unit_tests/`. Implementations are
//! filled in by the phase that introduces the corresponding feature.

pub mod validation;
pub mod mask;
pub mod filter;
pub mod promotion;

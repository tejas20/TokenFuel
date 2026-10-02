//! Platform-independent quota model. No credentials, network calls or UI dependencies.
pub mod model;
pub mod parsers;
pub mod scheduler;
pub use model::*;

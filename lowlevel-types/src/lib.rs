//! # Lowlevel Types
//!
//! Lowlevel Types is an implementation of types commonly used for low level access, such as ASCII
//! characters and strings.
//!
//! ## Features
//!
//! - Built in support for Serde serialization and deserialization
//! - ASCII Character and Fixed Length String support
//!
//! ## Installation
//!
//! Add this to your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! lowlevel-types = { version = "1" }
//! ```
//!
//! ## Legal
//!
//! Lowlevel Types is copyright &copy; 2025 JEleniel and released under either [The MIT License](LICENSE-MIT.md)
//! or [The Apache License](LICENSE-Apache.md), at your option.
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you
//! shall be licensed as above, without any additional terms or conditions.

/// ASCII types
pub mod ascii;

#[cfg(test)]
mod tests {}

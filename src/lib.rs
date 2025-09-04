//! # Serde Binary Advanced
//!
//! Serde Binary Advanced is a [Serde](https://crates.io/crates/serde) library enabling the
//! serialization and deserialization of Rust objects to binary representations.
//!
//! ## Features
//!
//! - Serialization and deserialization of Rust data structures to and from binary format
//! - Full support for ASCII (through `lowlevel-types`) and UTF-8 characters and strings
//! - Support for Big Endian and Little Endian (default) encoding
//! - Comprehensive error reporting
//! - Compression of `usize` markers for sequences and structures
//! - Support for `u128` and `i128` types
//! - Enums and variants stored as `u32`
//!
//! ## Limitations
//!
//! - No support foe serializing or deserializing sequences or maps of unknown length
//!
//! ## Installation
//!
//! Installation
//!
//! Add this to your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_binary_adv = { version = "1.0.0-beta.3" }
//! lowlevel_types = { version = "1.0.0-beta.3" }
//! ```
//!
//! ## Usage
//!
//! Here's a quick example on how to use Serde Binary Advanced to serialize and deserialize a struct
//! to and from binary:
//!
//! ```rust
//! use serde::{Serialize, Deserialize};
//! use serde_binary_adv::{Serializer, Deserializer, BinaryError, Result}
//!
//! # [derive(Serialize, Deserialize)]
//! struct Point {
//!     x: f64,
//!     y: f64,
//! }
//!
//! fn main() {
//!     let point = Point { x: 1.0, y: 2.0 };
//!
//!     let serialized = Serializer::to_bytes(&point, false).unwrap();
//!     let deserialized: Point = Deserializer::from_bytes(&serialized, false).unwrap();
//! 	assert_eq!(value, deserialized,);
//! }
//! ```
//!
//! ## Legal
//!
//! Serde Binary Advanced is copyright &copy; 2025 JEleniel and released under either
//! [The MIT License](LICENSE-MIT.md) or [The Apache License](LICENSE-Apache.md), at your option.
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
//! this crate by you shall be licensed as above, without any additional terms or conditions.

mod serde_binary_adv;

pub use serde_binary_adv::*;

#[cfg(test)]
mod tests {}

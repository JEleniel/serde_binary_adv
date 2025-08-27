//! # Serde Binary Advanced
//!
//! Serde Binary Advanced is a [Serde](https://crates.io/crates/serde) library enabling the
//! serialization and deserialization of Rust objects to and from binary representations.
//!
//! ## Features
//!
//! - Serialization and deserialization of Rust data structures to and from raw binary format
//! - Full support for all Rust native types
//! - Comprehensive error reporting
//!
//! ## Installation
//!
//! Add this to your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_binary_adv = { version = "1" }
//! ```
//!
//! ## Usage
//!
//! Here's a quick example on how to use Serde Binary Advanced to serialize and deserialize a struct to and from binary:
//!
//! ```rust
//! use serde::{Serialize, Deserialize};
//!
//! # [derive(Serialize, Deserialize)]
//! struct Point {
//!     x: i64,
//!     y: i64,
//! }
//!
//! fn main() -> Result<(),Error> {
//!     let point = Point { x: 1.0, y: 2.0 };
//!
//!     // Serialize to bytes
//!     let raw: Vec<u8> = serde_binary_adv::to_bytes(&point)?;
//!     assert_eq!(raw, vec![0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02]);
//!
//!     // Deserialize from bytes
//!     let deserialized_point: Point = serde_yml::from_bytes(&raw)?;
//!     assert_eq!(point, deserialized_point);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Legal
//!
//! Serde Binary Advanced is copyright &copy; 2025 JEleniel and released under either [The MIT License](LICENSE-MIT.md)
//! or [The Apache License](LICENSE-Apache.md), at your option.
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you
//! shall be licensed as above, without any additional terms or conditions.
mod serde_binary_adv;

pub use serde_binary_adv::*;

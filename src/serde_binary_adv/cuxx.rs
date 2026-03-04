//! Compressed unsigned integer (CInt) implementation
//! A compressed integer (CInt) is a u128 encoded in 1 to 17 bytes, depending on its value.
//! There are no functions for u8 because it is always encoded in a single byte already.
use serde::{Deserialize, Serialize};
use std::io::Read;
use thiserror::Error;

const BYTE_0_COUNT_MASK: u8 = 0x80;
const BYTE_0_VALUE_MASK: u8 = !BYTE_0_COUNT_MASK;
const BYTE_1_COUNT_MASK: u8 = 0xF0;
const BYTE_1_VALUE_MASK: u8 = !BYTE_1_COUNT_MASK;
const BYTE_N_VALUE_MASK: u8 = 0xFF;

/// A compressed 128-bit integer, encoded in 1 to 17 bytes.
/// Used for compact storage of integer values in EpilogLite.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Cu128 {
    bytes: Vec<u8>,
}

/// Small wrappers for narrower compressed integers. These are thin wrappers around
/// the canonical `CInt` encoding so users can explicitly express a cu64/cu32/cu16
/// semantic while reusing the same encoding/decoding logic.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Cu64(pub Cu128);

/// A compressed 32-bit unsigned integer wrapper around the canonical `CInt` encoding.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Cu32(pub Cu128);

/// A compressed 16-bit unsigned integer wrapper around the canonical `CInt` encoding.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Cu16(pub Cu128);

// Implement From for primitive types into the cu* wrappers (via CInt::from)
impl From<u64> for Cu64 {
    fn from(v: u64) -> Self {
        Cu64(Cu128::from(v as u128))
    }
}

impl From<u32> for Cu32 {
    fn from(v: u32) -> Self {
        Cu32(Cu128::from(v as u128))
    }
}

impl From<u16> for Cu16 {
    fn from(v: u16) -> Self {
        Cu16(Cu128::from(v as u128))
    }
}

// Implement TryFrom from cu* wrappers into primitive types by delegating to TryFrom<CInt>
// Provide conversions between the wrapper types and the canonical CInt so
// callers can move between representations without friction.
impl From<Cu64> for Cu128 {
    fn from(value: Cu64) -> Self {
        value.0
    }
}

/// Try to build a `Cu64` from a raw `CInt`. Returns an error if the encoded
/// value does not fit into a `u64`.
impl std::convert::TryFrom<Cu128> for Cu64 {
    type Error = CIntError;

    fn try_from(value: Cu128) -> Result<Self, Self::Error> {
        // Validate that the value fits a u64, using the existing TryFrom<Cu128> impl.
        let _v: u64 = u64::try_from(value.clone())?;
        Ok(Cu64(value))
    }
}

impl From<Cu32> for Cu128 {
    fn from(value: Cu32) -> Self {
        value.0
    }
}

/// Try to build a `Cu32` from a raw `CInt`. Returns an error if the encoded
/// value does not fit into a `u32`.
impl std::convert::TryFrom<Cu128> for Cu32 {
    type Error = CIntError;

    fn try_from(value: Cu128) -> Result<Self, Self::Error> {
        let _v: u32 = u32::try_from(value.clone())?;
        Ok(Cu32(value))
    }
}

impl From<Cu16> for Cu128 {
    fn from(value: Cu16) -> Self {
        value.0
    }
}

/// Try to build a `Cu16` from a raw `CInt`. Returns an error if the encoded
/// value does not fit into a `u16`.
impl std::convert::TryFrom<Cu128> for Cu16 {
    type Error = CIntError;

    fn try_from(value: Cu128) -> Result<Self, Self::Error> {
        let _v: u16 = u16::try_from(value.clone())?;
        Ok(Cu16(value))
    }
}

impl std::convert::TryFrom<Cu64> for u64 {
    type Error = CIntError;

    fn try_from(value: Cu64) -> Result<Self, Self::Error> {
        u64::try_from(value.0)
    }
}

impl std::convert::TryFrom<Cu32> for u32 {
    type Error = CIntError;

    fn try_from(value: Cu32) -> Result<Self, Self::Error> {
        u32::try_from(value.0)
    }
}

impl std::convert::TryFrom<Cu16> for u16 {
    type Error = CIntError;

    fn try_from(value: Cu16) -> Result<Self, Self::Error> {
        u16::try_from(value.0)
    }
}

impl Cu128 {
    /// Reads a `CInt` from a reader, reading the necessary number of bytes.
    /// Returns an error if the reader does not contain enough bytes or if the format is invalid.
    pub fn read_from(reader: &mut dyn Read) -> Result<Self, CIntError> {
        let mut bytes: Vec<u8> = Vec::new();

        let mut byte: [u8; 1] = [0];
        reader
            .read_exact(&mut byte)
            .map_err(|_| CIntError::NoData)?;
        bytes.push(byte[0]);

        if !(byte[0] & 0x80 != 0) {
            return Ok(Cu128 { bytes });
        }

        let mut len = 1 + ((byte[0] & BYTE_0_VALUE_MASK) as usize >> 7);
        if len != 2 {
            return Err(CIntError::InvalidEncodedLength(len));
        }

        byte = [0]; // Ensure byte is reset
        reader
            .read_exact(&mut byte)
            .map_err(|_| CIntError::TooFew(len, 1))?;
        bytes.push(byte[0]);
        len += ((byte[0] & BYTE_1_COUNT_MASK) >> 4) as usize;
        if len > 17 {
            return Err(CIntError::InvalidEncodedLength(len));
        }

        for i in 1..len {
            byte = [0];
            reader
                .read_exact(&mut byte)
                .map_err(|_| CIntError::TooFew(len, i))?;
            bytes.push(byte[0]);
        }
        Ok(Cu128 { bytes })
    }
}

impl std::fmt::Display for Cu128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Attempt to decode; on error show an explicit placeholder rather than panicking.
        match u128::try_from(self.clone()) {
            Ok(v) => write!(f, "{}", v),
            Err(_) => write!(f, "<invalid-cint>"),
        }
    }
}

impl std::fmt::Display for Cu64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Delegate to the canonical Cu128 Display implementation.
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::fmt::Display for Cu32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::fmt::Display for Cu16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl From<u128> for Cu128 {
    fn from(value: u128) -> Self {
        if value < BYTE_0_COUNT_MASK as u128 {
            return Cu128 {
                bytes: vec![value as u8],
            };
        }

        let mut bytes: Vec<u8> = Vec::new();
        let mut tv: u128 = value.clone();
        bytes.push(((tv & BYTE_1_VALUE_MASK as u128) as u8) | BYTE_0_COUNT_MASK);
        tv >>= 7;

        let mut byte_count: u8 = 0;
        bytes.push((tv & BYTE_1_VALUE_MASK as u128) as u8);
        tv >>= 4;

        while tv != 0 {
            bytes.push((tv & BYTE_N_VALUE_MASK as u128) as u8);
            tv >>= 8;
            byte_count += 1;
        }
        bytes[0] |= byte_count << 4;

        Cu128 { bytes }
    }
}

impl std::convert::TryFrom<Cu128> for u128 {
    type Error = CIntError;

    fn try_from(value: Cu128) -> Result<Self, Self::Error> {
        let bytes = value.bytes;
        if bytes.is_empty() {
            return Err(CIntError::NoData);
        }

        // Single-byte fast-path
        if bytes[0] & BYTE_0_COUNT_MASK == 0 {
            return Ok(bytes[0] as u128);
        }

        // Need at least two bytes for the multi-byte header
        if bytes.len() < 2 {
            return Err(CIntError::TooFew(2, bytes.len()));
        }

        let byte_count = (bytes[1] >> 4) as usize;
        let expected_len = 2usize + byte_count;
        if expected_len > 17 {
            return Err(CIntError::InvalidEncodedLength(expected_len));
        }
        if bytes.len() < expected_len {
            return Err(CIntError::TooFew(expected_len, bytes.len()));
        }

        let mut out: u128 = (bytes[0] & BYTE_1_VALUE_MASK) as u128;
        out |= ((bytes[1] & BYTE_1_VALUE_MASK) as u128) << 7;

        for i in 0..byte_count {
            let b = bytes[2 + i] as u128;
            out |= b << (11 + (i as u32 * 8));
        }

        Ok(out)
    }
}

impl From<u64> for Cu128 {
    fn from(value: u64) -> Self {
        Cu128::from(value as u128)
    }
}

impl From<u32> for Cu128 {
    fn from(value: u32) -> Self {
        Cu128::from(value as u128)
    }
}

impl From<u16> for Cu128 {
    fn from(value: u16) -> Self {
        Cu128::from(value as u128)
    }
}

impl From<usize> for Cu128 {
    fn from(value: usize) -> Self {
        Cu128::from(value as u128)
    }
}

// Try conversions FROM CInt into smaller integer types. These validate range and return a
// `CIntError::ValueOutOfRange` when the encoded value doesn't fit the target type.
impl std::convert::TryFrom<Cu128> for u16 {
    type Error = CIntError;

    fn try_from(value: Cu128) -> Result<Self, Self::Error> {
        let v: u128 = u128::try_from(value)?;
        if v > u16::MAX as u128 {
            return Err(CIntError::ValueOutOfRange(u16::MAX as u128, v));
        }
        Ok(v as u16)
    }
}

impl std::convert::TryFrom<Cu128> for u32 {
    type Error = CIntError;

    fn try_from(value: Cu128) -> Result<Self, Self::Error> {
        let v: u128 = u128::try_from(value)?;
        if v > u32::MAX as u128 {
            return Err(CIntError::ValueOutOfRange(u32::MAX as u128, v));
        }
        Ok(v as u32)
    }
}

impl std::convert::TryFrom<Cu128> for u64 {
    type Error = CIntError;

    fn try_from(value: Cu128) -> Result<Self, Self::Error> {
        let v: u128 = u128::try_from(value)?;
        if v > u64::MAX as u128 {
            return Err(CIntError::ValueOutOfRange(u64::MAX as u128, v));
        }
        Ok(v as u64)
    }
}

impl std::convert::TryFrom<Cu128> for usize {
    type Error = CIntError;

    fn try_from(value: Cu128) -> Result<Self, Self::Error> {
        let v: u128 = u128::try_from(value)?;
        if v > usize::MAX as u128 {
            return Err(CIntError::ValueOutOfRange(usize::MAX as u128, v));
        }
        Ok(v as usize)
    }
}

/// Errors that can occur during compressed integer encoding or decoding
#[derive(Clone, Debug, Error, PartialEq)]
pub enum CIntError {
    /// The encoded number of bytes is invalid
    #[error("Invalid byte count in compressed int, expected 1-17 bytes, found {0} encoded")]
    InvalidEncodedLength(usize),
    /// Not enough bytes in the input to decode the expected length
    #[error("Too few bytes, expected {0}, got {1}")]
    TooFew(usize, usize),
    /// Too many bytes in the input to decode the expected length
    #[error("Too many bytes, expected {0}, got {1}")]
    TooLong(usize, usize),
    /// The decoded value is out of range for the target type
    #[error("Value out of range, expected max {0}, got {1}")]
    ValueOutOfRange(u128, u128),
    /// The compressed int is empty (no bytes)
    #[error("No bytes to decode compressed int")]
    NoData,
    /// The value would overflow the target type during conversion
    #[error("Overflow during conversion")]
    Overflow,
    /// The math operation would underflow the target type during conversion
    #[error("Underflow during conversion")]
    Underflow,
}

//! COBS-framed postcard encode / decode helpers.
//!
//! ## Wire format
//!
//! ```text
//! ┌─────────────────────────┬──────┐
//! │  COBS-encoded payload   │ 0x00 │
//! └─────────────────────────┴──────┘
//! ```
//!
//! The COBS encoding replaces every `0x00` byte in the original postcard
//! payload with an offset marker, so the single `0x00` terminator byte
//! unambiguously signals the end of each frame.  This lets the receiver
//! accumulate bytes into a buffer and know when a complete frame has arrived
//! without needing a length prefix.
//!
//! ## Usage
//!
//! ### Embedded (no_std, fixed buffer)
//!
//! ```ignore
//! let mut buf = [0u8; COMMAND_FRAME_BUF];
//! let frame = encode_command_to_slice(&cmd, &mut buf)?;
//! uart.write_all(frame).await?;
//! ```
//!
//! ### Desktop (std, heap allocation)
//!
//! ```ignore
//! let frame = encode_command(&cmd)?;  // returns Vec<u8>
//! serial_port.write_all(&frame)?;
//! ```

use crate::{Command, Response};

// ── Error types ──────────────────────────────────────────────────────────────

/// Errors that can occur during encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    /// The caller-supplied buffer was too small.
    BufferTooSmall,
    /// Postcard serialisation failed.
    Serialise,
}

/// Errors that can occur during decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// Postcard deserialisation failed (bad COBS or corrupt data).
    Deserialise,
}

// ── Buffer size constants ────────────────────────────────────────────────────

/// Worst-case byte budget for an encoded [`Command`] COBS frame.
///
/// Computed manually from the largest variant:
/// `WriteRegisters { address: u16, values: Vec<u16, 32> }` encodes to at most
/// ≈ 70 postcard bytes; COBS adds ≤ 1 byte per 254 bytes, plus 1 terminator.
/// 128 bytes provides a generous safety margin.
pub const COMMAND_FRAME_BUF: usize = 128;

/// Worst-case byte budget for an encoded [`Response`] COBS frame.
///
/// Computed manually from the largest variant:
/// `Registers { address: u16, values: Vec<u16, 32> }` encodes to at most
/// ≈ 70 postcard bytes.  256 bytes provides a generous safety margin.
pub const RESPONSE_FRAME_BUF: usize = 256;

// ── no_std / fixed-buffer helpers ────────────────────────────────────────────

/// Encode `cmd` into `buf` using COBS-framed postcard.
///
/// Returns the encoded byte slice (including the trailing `0x00` sentinel) on
/// success, or [`EncodeError::BufferTooSmall`] when `buf` is not large enough.
///
/// A buffer of at least [`COMMAND_FRAME_BUF`] bytes is guaranteed to work.
pub fn encode_command_to_slice<'b>(
    cmd: &Command,
    buf: &'b mut [u8],
) -> Result<&'b mut [u8], EncodeError> {
    postcard::to_slice_cobs(cmd, buf).map_err(|_| EncodeError::BufferTooSmall)
}

/// Encode `resp` into `buf` using COBS-framed postcard.
///
/// Returns the encoded byte slice (including the trailing `0x00` sentinel) on
/// success, or [`EncodeError::BufferTooSmall`] when `buf` is not large enough.
///
/// A buffer of at least [`RESPONSE_FRAME_BUF`] bytes is guaranteed to work.
pub fn encode_response_to_slice<'b>(
    resp: &Response,
    buf: &'b mut [u8],
) -> Result<&'b mut [u8], EncodeError> {
    postcard::to_slice_cobs(resp, buf).map_err(|_| EncodeError::BufferTooSmall)
}

/// Decode a [`Command`] from a COBS frame.
///
/// `frame` must be the raw bytes **including** the trailing `0x00` byte, as
/// they come off the wire.  The slice is mutated in-place during COBS decoding.
pub fn decode_command_from_frame(frame: &mut [u8]) -> Result<Command, DecodeError> {
    postcard::from_bytes_cobs(frame).map_err(|_| DecodeError::Deserialise)
}

/// Decode a [`Response`] from a COBS frame.
///
/// `frame` must be the raw bytes **including** the trailing `0x00` byte, as
/// they come off the wire.  The slice is mutated in-place during COBS decoding.
pub fn decode_response_from_frame(frame: &mut [u8]) -> Result<Response, DecodeError> {
    postcard::from_bytes_cobs(frame).map_err(|_| DecodeError::Deserialise)
}

// ── std heap helpers ──────────────────────────────────────────────────────────

#[cfg(feature = "std")]
mod std_helpers {
    use std::vec::Vec;

    use super::*;

    /// Encode `cmd` into a heap-allocated `Vec<u8>` (std environments).
    ///
    /// The returned vector includes the trailing `0x00` sentinel byte.
    pub fn encode_command(cmd: &Command) -> Result<Vec<u8>, EncodeError> {
        postcard::to_stdvec_cobs(cmd).map_err(|_| EncodeError::Serialise)
    }

    /// Encode `resp` into a heap-allocated `Vec<u8>` (std environments).
    ///
    /// The returned vector includes the trailing `0x00` sentinel byte.
    pub fn encode_response(resp: &Response) -> Result<Vec<u8>, EncodeError> {
        postcard::to_stdvec_cobs(resp).map_err(|_| EncodeError::Serialise)
    }
}

#[cfg(feature = "std")]
pub use std_helpers::{encode_command, encode_response};

// ── alloc heap helpers (no_std + alloc) ───────────────────────────────────────

#[cfg(all(feature = "alloc", not(feature = "std")))]
mod alloc_helpers {
    use alloc::vec::Vec;

    use super::*;

    /// Encode `cmd` into a heap-allocated `Vec<u8>` (no_std + alloc environments).
    ///
    /// The returned vector includes the trailing `0x00` sentinel byte.
    pub fn encode_command(cmd: &Command) -> Result<Vec<u8>, EncodeError> {
        postcard::to_allocvec_cobs(cmd).map_err(|_| EncodeError::Serialise)
    }

    /// Encode `resp` into a heap-allocated `Vec<u8>` (no_std + alloc environments).
    ///
    /// The returned vector includes the trailing `0x00` sentinel byte.
    pub fn encode_response(resp: &Response) -> Result<Vec<u8>, EncodeError> {
        postcard::to_allocvec_cobs(resp).map_err(|_| EncodeError::Serialise)
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc_helpers::{encode_command, encode_response};

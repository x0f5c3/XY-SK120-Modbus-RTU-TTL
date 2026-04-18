//! # xy-sk120-protocol
//!
//! Shared protocol types for communicating with an XY-SK120 power-supply module
//! over the USB-serial (CDC) connection exposed by the ESP32-S3 firmware.
//!
//! ## Overview
//!
//! The firmware acts as a **device** (peripheral).  The Tauri desktop GUI acts
//! as a **host** (controller).
//!
//! ```text
//! ┌──────────────┐  USB-CDC serial  ┌─────────────────┐
//! │  Tauri GUI   │ ─── Command ───► │  ESP32-S3 FW    │
//! │  (desktop)   │ ◄── Response ─── │  (xy-sk120-esp) │
//! └──────────────┘                  └─────────────────┘
//! ```
//!
//! ## Wire framing
//!
//! Every message is serialised with [postcard](https://docs.rs/postcard) and
//! then wrapped in **COBS** (Consistent Overhead Byte Stuffing) framing so that
//! a single `0x00` byte unambiguously marks the end of each packet.  The sender
//! appends one `0x00` sentinel byte after the COBS-encoded payload; the receiver
//! accumulates bytes until it sees a `0x00` and then hands the frame to the
//! codec.
//!
//! ## Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `std` (default) | Enables heap-based codec helpers via `postcard::to_stdvec_cobs` |
//! | `alloc` | Enables heap-based codec helpers in `no_std + alloc` environments |
//!
//! Without either flag the crate is pure `no_std` with no heap allocator
//! required.  Use the [`codec`] module's `encode_to_slice` / `decode_from_slice`
//! functions together with caller-supplied fixed-size buffers.

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod codec;
pub mod commands;
pub mod responses;

pub use commands::Command;
pub use responses::{DeviceInfo, ErrorCode, LiveMeasurements, OutputStatus, Response};

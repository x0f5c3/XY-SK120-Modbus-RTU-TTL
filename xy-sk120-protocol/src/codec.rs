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

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "std"))]
mod tests {
    use heapless::Vec as HVec;

    use super::*;
    use crate::{
        commands::TemperatureUnit,
        responses::{DeviceInfo, ErrorCode, LiveMeasurements, OutputStatus, ProtectionStatus},
        Command, Response,
    };

    // ── helpers ──────────────────────────────────────────────────────────────

    /// Round-trip encode → decode for a Command using fixed-size buffers.
    fn rt_command_slice(cmd: &Command) -> Command {
        let mut buf = [0u8; COMMAND_FRAME_BUF];
        let frame = encode_command_to_slice(cmd, &mut buf).expect("encode failed");
        decode_command_from_frame(frame).expect("decode failed")
    }

    /// Round-trip encode → decode for a Command using heap allocation.
    fn rt_command_heap(cmd: &Command) -> Command {
        let mut frame = encode_command(cmd).expect("encode failed");
        decode_command_from_frame(&mut frame).expect("decode failed")
    }

    /// Round-trip encode → decode for a Response using fixed-size buffers.
    fn rt_response_slice(resp: &Response) -> Response {
        let mut buf = [0u8; RESPONSE_FRAME_BUF];
        let frame = encode_response_to_slice(resp, &mut buf).expect("encode failed");
        decode_response_from_frame(frame).expect("decode failed")
    }

    /// Round-trip encode → decode for a Response using heap allocation.
    fn rt_response_heap(resp: &Response) -> Response {
        let mut frame = encode_response(resp).expect("encode failed");
        decode_response_from_frame(&mut frame).expect("decode failed")
    }

    // ── COBS framing ─────────────────────────────────────────────────────────

    /// The last byte of every encoded command frame must be the COBS sentinel.
    #[test]
    fn command_frame_ends_with_zero() {
        let frame = encode_command(&Command::GetInfo).unwrap();
        assert_eq!(*frame.last().unwrap(), 0x00, "frame should end with 0x00");
    }

    /// The last byte of every encoded response frame must be the COBS sentinel.
    #[test]
    fn response_frame_ends_with_zero() {
        let frame = encode_response(&Response::Ok).unwrap();
        assert_eq!(*frame.last().unwrap(), 0x00, "frame should end with 0x00");
    }

    /// Every intermediate byte of a COBS frame must be non-zero.
    #[test]
    fn command_frame_no_interior_zeros() {
        let frame = encode_command(&Command::SetVoltageCurrent {
            voltage_v: 12.0,
            current_a: 2.5,
        })
        .unwrap();
        let len = frame.len();
        for b in &frame[..len - 1] {
            assert_ne!(*b, 0x00, "unexpected 0x00 before end of frame");
        }
    }

    // ── Query commands ────────────────────────────────────────────────────────

    #[test]
    fn get_info_roundtrip() {
        let cmd = Command::GetInfo;
        assert!(matches!(rt_command_slice(&cmd), Command::GetInfo));
        assert!(matches!(rt_command_heap(&cmd), Command::GetInfo));
    }

    #[test]
    fn get_output_status_roundtrip() {
        let cmd = Command::GetOutputStatus;
        assert!(matches!(rt_command_slice(&cmd), Command::GetOutputStatus));
    }

    #[test]
    fn get_temperature_roundtrip() {
        let cmd = Command::GetTemperature;
        assert!(matches!(rt_command_slice(&cmd), Command::GetTemperature));
    }

    #[test]
    fn get_live_measurements_roundtrip() {
        let cmd = Command::GetLiveMeasurements;
        assert!(matches!(
            rt_command_slice(&cmd),
            Command::GetLiveMeasurements
        ));
    }

    #[test]
    fn get_protection_status_roundtrip() {
        let cmd = Command::GetProtectionStatus;
        assert!(matches!(
            rt_command_slice(&cmd),
            Command::GetProtectionStatus
        ));
    }

    #[test]
    fn get_memory_group_roundtrip() {
        for g in 0u8..=9 {
            let cmd = Command::GetMemoryGroup(g);
            if let Command::GetMemoryGroup(got) = rt_command_slice(&cmd) {
                assert_eq!(got, g);
            } else {
                panic!("wrong variant");
            }
        }
    }

    // ── Output control commands ───────────────────────────────────────────────

    #[test]
    fn set_output_roundtrip() {
        for on in [true, false] {
            let cmd = Command::SetOutput(on);
            if let Command::SetOutput(got) = rt_command_slice(&cmd) {
                assert_eq!(got, on);
            } else {
                panic!("wrong variant");
            }
        }
    }

    #[test]
    fn set_voltage_roundtrip() {
        let cmd = Command::SetVoltage(24.5);
        if let Command::SetVoltage(v) = rt_command_slice(&cmd) {
            assert!((v - 24.5f32).abs() < 1e-4);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn set_current_roundtrip() {
        let cmd = Command::SetCurrent(3.14);
        if let Command::SetCurrent(i) = rt_command_heap(&cmd) {
            assert!((i - 3.14f32).abs() < 1e-4);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn set_voltage_current_roundtrip() {
        let cmd = Command::SetVoltageCurrent {
            voltage_v: 12.0,
            current_a: 1.5,
        };
        if let Command::SetVoltageCurrent {
            voltage_v: v,
            current_a: i,
        } = rt_command_slice(&cmd)
        {
            assert!((v - 12.0f32).abs() < 1e-4);
            assert!((i - 1.5f32).abs() < 1e-4);
        } else {
            panic!("wrong variant");
        }
    }

    // ── Protection threshold commands ─────────────────────────────────────────

    #[test]
    fn set_ovp_roundtrip() {
        let cmd = Command::SetOverVoltageProtection(15.0);
        if let Command::SetOverVoltageProtection(v) = rt_command_slice(&cmd) {
            assert!((v - 15.0f32).abs() < 1e-4);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn set_ocp_roundtrip() {
        let cmd = Command::SetOverCurrentProtection(5.0);
        if let Command::SetOverCurrentProtection(v) = rt_command_heap(&cmd) {
            assert!((v - 5.0f32).abs() < 1e-4);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn set_mppt_threshold_roundtrip() {
        let cmd = Command::SetMpptThreshold(75);
        if let Command::SetMpptThreshold(t) = rt_command_slice(&cmd) {
            assert_eq!(t, 75);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn set_temperature_unit_roundtrip() {
        for unit in [TemperatureUnit::Celsius, TemperatureUnit::Fahrenheit] {
            let cmd = Command::SetTemperatureUnit(unit);
            if let Command::SetTemperatureUnit(u) = rt_command_slice(&cmd) {
                assert_eq!(u, unit);
            } else {
                panic!("wrong variant");
            }
        }
    }

    // ── Bulk-register commands ────────────────────────────────────────────────

    #[test]
    fn write_registers_roundtrip() {
        let mut values: HVec<u16, 32> = HVec::new();
        for v in [0x0001u16, 0x00C8, 0x07D0] {
            values.push(v).unwrap();
        }
        let cmd = Command::WriteRegisters {
            address: 0x0050,
            values: values.clone(),
        };
        if let Command::WriteRegisters {
            address: addr,
            values: got,
        } = rt_command_slice(&cmd)
        {
            assert_eq!(addr, 0x0050);
            assert_eq!(got.as_slice(), values.as_slice());
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn read_registers_roundtrip() {
        let cmd = Command::ReadRegisters {
            address: 0x0000,
            count: 10,
        };
        if let Command::ReadRegisters { address, count } = rt_command_heap(&cmd) {
            assert_eq!(address, 0x0000);
            assert_eq!(count, 10);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn write_register_roundtrip() {
        let cmd = Command::WriteRegister {
            address: 0x0012,
            value: 0x0001,
        };
        if let Command::WriteRegister { address, value } = rt_command_slice(&cmd) {
            assert_eq!(address, 0x0012);
            assert_eq!(value, 0x0001);
        } else {
            panic!("wrong variant");
        }
    }

    // ── Response variants ─────────────────────────────────────────────────────

    #[test]
    fn response_ok_roundtrip() {
        let resp = Response::Ok;
        assert!(matches!(rt_response_slice(&resp), Response::Ok));
        assert!(matches!(rt_response_heap(&resp), Response::Ok));
    }

    #[test]
    fn response_err_roundtrip() {
        for code in [
            ErrorCode::ModbusCrcError,
            ErrorCode::ModbusIoError,
            ErrorCode::InvalidParameter,
            ErrorCode::Unknown,
        ] {
            let resp = Response::Err(code);
            if let Response::Err(got) = rt_response_slice(&resp) {
                assert_eq!(got, code);
            } else {
                panic!("wrong variant");
            }
        }
    }

    #[test]
    fn response_device_info_roundtrip() {
        let resp = Response::DeviceInfo(DeviceInfo {
            model: 120,
            version: 3,
        });
        if let Response::DeviceInfo(info) = rt_response_heap(&resp) {
            assert_eq!(info.model, 120);
            assert_eq!(info.version, 3);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn response_output_status_roundtrip() {
        let resp = Response::OutputStatus(OutputStatus {
            output_voltage_v: 12.34,
            output_current_a: 1.234,
            output_power_w: 15.22,
            input_voltage_v: 24.0,
            output_enabled: true,
        });
        if let Response::OutputStatus(s) = rt_response_slice(&resp) {
            assert!((s.output_voltage_v - 12.34f32).abs() < 1e-4);
            assert!((s.output_current_a - 1.234f32).abs() < 1e-4);
            assert!((s.output_power_w - 15.22f32).abs() < 1e-4);
            assert!((s.input_voltage_v - 24.0f32).abs() < 1e-4);
            assert!(s.output_enabled);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn response_temperature_roundtrip() {
        let resp = Response::Temperature(45.6);
        if let Response::Temperature(t) = rt_response_heap(&resp) {
            assert!((t - 45.6f32).abs() < 1e-4);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn response_live_measurements_roundtrip() {
        let resp = Response::LiveMeasurements(LiveMeasurements {
            output_voltage_v: 5.0,
            output_current_a: 2.0,
            output_power_w: 10.0,
            input_voltage_v: 12.0,
            temperature_c: 35.0,
        });
        if let Response::LiveMeasurements(m) = rt_response_slice(&resp) {
            assert!((m.output_voltage_v - 5.0f32).abs() < 1e-4);
            assert!((m.temperature_c - 35.0f32).abs() < 1e-4);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn response_protection_status_roundtrip() {
        // OVP (bit 0) + OTP (bit 3)
        let resp = Response::ProtectionStatus(ProtectionStatus { flags: 0b0000_1001 });
        if let Response::ProtectionStatus(ps) = rt_response_heap(&resp) {
            assert_eq!(ps.flags, 0b0000_1001);
            assert!(ps.ovp());
            assert!(!ps.ocp());
            assert!(!ps.opp());
            assert!(ps.otp());
            assert!(!ps.lvp());
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn response_memory_group_roundtrip() {
        let mut regs: HVec<u16, 14> = HVec::new();
        for v in [100u16, 200, 300, 400] {
            regs.push(v).unwrap();
        }
        let resp = Response::MemoryGroup {
            group: 3,
            registers: regs.clone(),
        };
        if let Response::MemoryGroup { group, registers } = rt_response_slice(&resp) {
            assert_eq!(group, 3);
            assert_eq!(registers.as_slice(), regs.as_slice());
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn response_registers_roundtrip() {
        let mut vals: HVec<u16, 32> = HVec::new();
        for v in 0u16..8 {
            vals.push(v * 10).unwrap();
        }
        let resp = Response::Registers {
            address: 0x0002,
            values: vals.clone(),
        };
        if let Response::Registers { address, values } = rt_response_heap(&resp) {
            assert_eq!(address, 0x0002);
            assert_eq!(values.as_slice(), vals.as_slice());
        } else {
            panic!("wrong variant");
        }
    }

    // ── Buffer sizing ─────────────────────────────────────────────────────────

    /// Ensure the compile-time constant is actually big enough for the largest command.
    #[test]
    fn command_buf_constant_is_sufficient() {
        let mut values: HVec<u16, 32> = HVec::new();
        for v in 0u16..32 {
            values.push(v).unwrap();
        }
        let cmd = Command::WriteRegisters {
            address: 0xFFFF,
            values,
        };
        let mut buf = [0u8; COMMAND_FRAME_BUF];
        encode_command_to_slice(&cmd, &mut buf).expect("COMMAND_FRAME_BUF is too small");
    }

    /// Ensure the compile-time constant is actually big enough for the largest response.
    #[test]
    fn response_buf_constant_is_sufficient() {
        let mut values: HVec<u16, 32> = HVec::new();
        for v in 0u16..32 {
            values.push(v).unwrap();
        }
        let resp = Response::Registers {
            address: 0xFFFF,
            values,
        };
        let mut buf = [0u8; RESPONSE_FRAME_BUF];
        encode_response_to_slice(&resp, &mut buf).expect("RESPONSE_FRAME_BUF is too small");
    }

    /// A frame that has been corrupted should fail to decode.
    #[test]
    fn corrupted_frame_returns_error() {
        let cmd = Command::GetInfo;
        let mut frame = encode_command(&cmd).unwrap();
        // Flip some bits in the middle
        let mid = frame.len() / 2;
        frame[mid] ^= 0xFF;
        let result = decode_command_from_frame(&mut frame);
        assert!(result.is_err());
    }
}

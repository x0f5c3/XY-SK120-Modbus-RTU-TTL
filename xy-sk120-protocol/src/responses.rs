//! [`Response`] messages sent from the firmware **to** the desktop GUI.

use heapless::Vec;
use serde::{Deserialize, Serialize};

use crate::commands::MAX_REGISTER_VALUES;

/// Maximum number of registers that a single memory-group read can return.
pub const MAX_GROUP_REGISTERS: usize = 14;

// ── Supporting data types ────────────────────────────────────────────────────

/// Basic device identity returned by [`Command::GetInfo`](crate::Command::GetInfo).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Model identifier code.
    pub model: u16,
    /// Firmware version code.
    pub version: u16,
}

/// Snapshot of the current output state, returned by
/// [`Command::GetOutputStatus`](crate::Command::GetOutputStatus).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OutputStatus {
    /// Measured output voltage in Volts.
    pub output_voltage_v: f32,
    /// Measured output current in Amps.
    pub output_current_a: f32,
    /// Calculated output power in Watts.
    pub output_power_w: f32,
    /// Measured input (bus) voltage in Volts.
    pub input_voltage_v: f32,
    /// `true` if the power output is currently enabled.
    pub output_enabled: bool,
}

/// All live measurements in one response, returned by
/// [`Command::GetLiveMeasurements`](crate::Command::GetLiveMeasurements).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LiveMeasurements {
    /// Measured output voltage in Volts.
    pub output_voltage_v: f32,
    /// Measured output current in Amps.
    pub output_current_a: f32,
    /// Calculated output power in Watts.
    pub output_power_w: f32,
    /// Measured input (bus) voltage in Volts.
    pub input_voltage_v: f32,
    /// Internal temperature in °C.
    pub temperature_c: f32,
}

/// Error codes returned by the firmware when a command fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// The Modbus response CRC did not match.
    ModbusCrcError,
    /// The Modbus slave address in the response did not match.
    ModbusInvalidSlave,
    /// The Modbus function code in the response was unexpected.
    ModbusInvalidFunction,
    /// The response frame was too short.
    ModbusFrameTooShort,
    /// A Modbus I/O error occurred (read/write failed).
    ModbusIoError,
    /// The Modbus response contained inconsistent data.
    ModbusProtocolError,
    /// The provided parameter value is out of range.
    InvalidParameter,
    /// An unknown or unexpected error occurred.
    Unknown,
}

/// Protection-status bit flags.
///
/// The raw 16-bit register value is returned verbatim.  Individual bits
/// correspond to protection events defined in the XY-SK120 documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtectionStatus {
    /// Raw value of the protection-status register.
    pub flags: u16,
}

impl ProtectionStatus {
    /// Returns `true` if the over-voltage protection event is active.
    pub fn ovp(&self) -> bool {
        self.flags & (1 << 0) != 0
    }

    /// Returns `true` if the over-current protection event is active.
    pub fn ocp(&self) -> bool {
        self.flags & (1 << 1) != 0
    }

    /// Returns `true` if the over-power protection event is active.
    pub fn opp(&self) -> bool {
        self.flags & (1 << 2) != 0
    }

    /// Returns `true` if the over-temperature protection event is active.
    pub fn otp(&self) -> bool {
        self.flags & (1 << 3) != 0
    }

    /// Returns `true` if the low-voltage (input under-voltage) protection
    /// event is active.
    pub fn lvp(&self) -> bool {
        self.flags & (1 << 4) != 0
    }
}

// ── Response enum ────────────────────────────────────────────────────────────

/// All responses the firmware can send back to the desktop GUI.
///
/// For **set** commands the firmware replies with either [`Response::Ok`] or
/// [`Response::Err`].  For **query** commands the firmware uses one of the
/// data-bearing variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// The command executed successfully and there is no additional data.
    Ok,

    /// The command failed; the enclosed [`ErrorCode`] indicates the reason.
    Err(ErrorCode),

    // ── Query responses ──────────────────────────────────────────────────
    /// Response to [`Command::GetInfo`](crate::Command::GetInfo).
    DeviceInfo(DeviceInfo),

    /// Response to [`Command::GetOutputStatus`](crate::Command::GetOutputStatus).
    OutputStatus(OutputStatus),

    /// Response to [`Command::GetTemperature`](crate::Command::GetTemperature).
    ///
    /// Temperature is in **°C**.
    Temperature(f32),

    /// Response to
    /// [`Command::GetLiveMeasurements`](crate::Command::GetLiveMeasurements).
    LiveMeasurements(LiveMeasurements),

    /// Response to
    /// [`Command::GetProtectionStatus`](crate::Command::GetProtectionStatus).
    ProtectionStatus(ProtectionStatus),

    /// Response to [`Command::GetMemoryGroup`](crate::Command::GetMemoryGroup).
    MemoryGroup {
        /// The memory group index that was read (0–9).
        group: u8,
        /// Raw register values from the group.
        registers: Vec<u16, MAX_GROUP_REGISTERS>,
    },

    /// Response to [`Command::ReadRegisters`](crate::Command::ReadRegisters).
    Registers {
        /// The starting address of the read.
        address: u16,
        /// The register values returned.
        values: Vec<u16, MAX_REGISTER_VALUES>,
    },
}

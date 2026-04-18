//! [`Command`] messages sent from the desktop GUI **to** the firmware.

use heapless::Vec;
use serde::{Deserialize, Serialize};

/// Maximum number of register values that can be written or read in a single
/// bulk-register operation.
pub const MAX_REGISTER_VALUES: usize = 32;

/// Temperature display unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemperatureUnit {
    /// Degrees Celsius.
    Celsius,
    /// Degrees Fahrenheit.
    Fahrenheit,
}

/// All commands that the desktop GUI can send to the XY-SK120 firmware over
/// the USB serial connection.
///
/// After sending a `Command` the host **must** wait for the matching
/// [`Response`](crate::Response) before issuing the next command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    // ── Read-only queries ───────────────────────────────────────────────────
    /// Query the device model number and firmware version.
    GetInfo,

    /// Read the current output voltage, current, power, input voltage, and
    /// output-enable state.
    GetOutputStatus,

    /// Read the internal temperature (°C).
    GetTemperature,

    /// Read all live measurements in a single round-trip:
    /// voltage, current, power, input voltage, and temperature.
    GetLiveMeasurements,

    /// Read the raw protection-status register flags.
    GetProtectionStatus,

    /// Read the register contents of a preset memory group (0–9).
    GetMemoryGroup(
        /// Memory group index (0–9).
        u8,
    ),

    // ── Output control ──────────────────────────────────────────────────────
    /// Enable (`true`) or disable (`false`) the power output.
    SetOutput(bool),

    /// Set the target output voltage in **Volts** (e.g. `12.0`).
    SetVoltage(
        /// Voltage in V (resolution 0.01 V).
        f32,
    ),

    /// Set the target output current in **Amps** (e.g. `2.5`).
    SetCurrent(
        /// Current in A (resolution 0.001 A).
        f32,
    ),

    /// Set both output voltage and current in a single operation.
    SetVoltageCurrent {
        /// Target voltage in V (resolution 0.01 V).
        voltage_v: f32,
        /// Target current in A (resolution 0.001 A).
        current_a: f32,
    },

    // ── UI / UX settings ────────────────────────────────────────────────────
    /// Lock (`true`) or unlock (`false`) the front-panel keys on the device.
    SetKeyLock(bool),

    /// Select whether the device display shows temperatures in °C or °F.
    SetTemperatureUnit(TemperatureUnit),

    // ── Protection thresholds ───────────────────────────────────────────────
    /// Set the over-voltage protection limit in **Volts** (resolution 0.01 V).
    SetOverVoltageProtection(f32),

    /// Set the over-current protection limit in **Amps** (resolution 0.001 A).
    SetOverCurrentProtection(f32),

    /// Set the over-power protection limit in **Watts** (resolution 0.1 W).
    SetOverPowerProtection(f32),

    /// Set the low-voltage protection (input under-voltage) limit in **Volts**
    /// (resolution 0.01 V).
    SetLowVoltageProtection(f32),

    /// Set the over-temperature protection limit in **°C** (resolution 0.1 °C).
    SetOverTempProtection(f32),

    /// Set the battery charge cutoff current in **Amps** (resolution 0.001 A).
    SetBatteryCutoffCurrent(f32),

    // ── MPPT (Maximum Power Point Tracking) ─────────────────────────────────
    /// Enable (`true`) or disable (`false`) MPPT mode.
    SetMpptEnable(bool),

    /// Set the MPPT threshold as a percentage (0–100 %).
    SetMpptThreshold(
        /// Threshold percentage (clamped to 0–100).
        u8,
    ),

    // ── Constant-Power (CP) mode ─────────────────────────────────────────────
    /// Enable (`true`) or disable (`false`) constant-power mode.
    SetConstantPowerMode(bool),

    /// Set the constant-power setpoint in **Watts** (resolution 0.1 W).
    SetConstantPowerValue(f32),

    // ── Preset memory groups ─────────────────────────────────────────────────
    /// Recall (apply) a preset memory group (0–9) to the device.
    RecallMemoryGroup(
        /// Memory group index (0–9).
        u8,
    ),

    // ── Low-level / debug access ─────────────────────────────────────────────
    /// Read up to [`MAX_REGISTER_VALUES`] holding registers starting at `address`.
    ReadRegisters {
        /// Starting register address (0-based, big-endian on wire).
        address: u16,
        /// Number of registers to read (1–[`MAX_REGISTER_VALUES`]).
        count: u16,
    },

    /// Write a single holding register.
    WriteRegister {
        /// Register address.
        address: u16,
        /// Value to write.
        value: u16,
    },

    /// Write up to [`MAX_REGISTER_VALUES`] consecutive holding registers.
    WriteRegisters {
        /// Starting register address.
        address: u16,
        /// Values to write (up to [`MAX_REGISTER_VALUES`] entries).
        values: Vec<u16, MAX_REGISTER_VALUES>,
    },

    /// Restore all device settings to factory defaults.
    ///
    /// ⚠️ **Irreversible** – use with caution.
    RestoreFactoryDefaults,
}

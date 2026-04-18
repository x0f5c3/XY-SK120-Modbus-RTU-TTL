# xy-sk120-protocol

Shared Rust library crate that defines the **USB serial protocol** used between:

- the **ESP32-S3 firmware** (`xy-sk120-esp-rs`) running on the power-supply module, and
- the **Tauri desktop GUI** (desktop side) that controls it.

The protocol abstracts away all Modbus RTU internals.  A desktop developer only needs to construct one of the typed [`Command`] variants and call the codec to send it.  No knowledge of Modbus register addresses, CRC polynomials, or raw byte manipulation is required.

## Quick-start

### Desktop (std)

```toml
[dependencies]
xy-sk120-protocol = { path = "../xy-sk120-protocol" }  # default-features = ["std"]
```

```rust
use xy_sk120_protocol::{
    Command,
    codec::{encode_command, decode_response_from_frame, RESPONSE_FRAME_BUF},
};

// Encode a command and write it to the serial port
let frame = encode_command(&Command::SetOutput(true))?;
serial_port.write_all(&frame)?;

// Read the response (accumulate until 0x00)
let mut buf = vec![0u8; RESPONSE_FRAME_BUF];
let n = serial_port.read_until(0x00, &mut buf)?;
let response = decode_response_from_frame(&mut buf[..n])?;
```

### Embedded firmware (no_std + alloc)

```toml
[dependencies]
xy-sk120-protocol = { path = "../xy-sk120-protocol", default-features = false, features = ["alloc"] }
```

```rust
use xy_sk120_protocol::{
    codec::{
        decode_command_from_frame, encode_response, COMMAND_FRAME_BUF,
    },
    Command, Response,
};

// Accumulate USB-serial bytes into `frame_buf` until you see a 0x00, then:
let cmd = decode_command_from_frame(&mut frame_buf[..n])?;

// Execute it, then encode and send the response
let resp = match cmd {
    Command::GetInfo => Response::DeviceInfo(device.read_info().await?),
    Command::SetOutput(on) => {
        device.set_output(on).await?;
        Response::Ok
    }
    // …
};
let bytes = encode_response(&resp)?;
usb_uart.write_all(&bytes).await?;
```

---

## Commands

| Command | Description |
|---------|-------------|
| `GetInfo` | Read model number and firmware version |
| `GetOutputStatus` | Read voltage, current, power, input voltage, enable state |
| `GetTemperature` | Read internal temperature (°C) |
| `GetLiveMeasurements` | All live readings in one round-trip |
| `GetProtectionStatus` | Raw protection-event flags |
| `GetMemoryGroup(n)` | Read preset memory group 0–9 |
| `SetOutput(bool)` | Enable / disable power output |
| `SetVoltage(V)` | Set output voltage in Volts |
| `SetCurrent(A)` | Set output current in Amps |
| `SetVoltageCurrent { V, A }` | Set voltage **and** current together |
| `SetKeyLock(bool)` | Lock / unlock front-panel keys |
| `SetTemperatureUnit(unit)` | Switch display between °C and °F |
| `SetOverVoltageProtection(V)` | OVP threshold |
| `SetOverCurrentProtection(A)` | OCP threshold |
| `SetOverPowerProtection(W)` | OPP threshold |
| `SetLowVoltageProtection(V)` | LVP threshold |
| `SetOverTempProtection(°C)` | OTP threshold |
| `SetBatteryCutoffCurrent(A)` | Battery charge cutoff |
| `SetMpptEnable(bool)` | Enable / disable MPPT mode |
| `SetMpptThreshold(%)` | MPPT threshold 0–100 % |
| `SetConstantPowerMode(bool)` | Enable / disable constant-power mode |
| `SetConstantPowerValue(W)` | CP setpoint in Watts |
| `RecallMemoryGroup(n)` | Apply preset memory group 0–9 |
| `ReadRegisters { addr, count }` | Raw register read (debug) |
| `WriteRegister { addr, value }` | Raw single-register write (debug) |
| `WriteRegisters { addr, values }` | Raw multi-register write (debug) |
| `RestoreFactoryDefaults` | ⚠️ Factory reset |

## Responses

All **set** commands return `Response::Ok` on success or `Response::Err(ErrorCode)` on failure.

Query commands return their matching data variant (e.g. `Response::DeviceInfo`, `Response::OutputStatus`, etc.).

## Wire framing

Every message is serialised with [`postcard`](https://docs.rs/postcard) and then COBS-encoded.  A trailing `0x00` byte marks the end of each frame.  Maximum frame sizes are exposed as `codec::COMMAND_FRAME_BUF` and `codec::RESPONSE_FRAME_BUF`.

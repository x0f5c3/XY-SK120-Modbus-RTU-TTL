use alloc::{format, string::String, vec::Vec};

use embedded_io::ErrorType;
use embedded_io_async::{Read, Write};

use crate::device::DeviceController;

pub async fn run_tui<CON, MB>(console: &mut CON, device: &mut DeviceController<MB>) -> !
where
    CON: Read + Write + ErrorType,
    MB: Read + Write + ErrorType,
{
    write_line(console, "\r\nXY-SK120 Rust Modbus TUI").await;
    write_line(console, "Type 'help' for commands.").await;

    loop {
        write_raw(console, b"\r\n> ").await;

        let line = read_line(console).await;
        if line.is_empty() {
            continue;
        }

        handle_command(console, device, &line).await;
    }
}

async fn handle_command<CON, MB>(
    console: &mut CON,
    device: &mut DeviceController<MB>,
    line: &str,
) where
    CON: Read + Write + ErrorType,
    MB: Read + Write + ErrorType,
{
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() {
        return;
    }

    match tokens[0] {
        "help" | "menu" => show_help(console).await,
        "info" => match device.read_info().await {
            Ok(info) => {
                write_line(
                    console,
                    &format!("Model: {} | Firmware: {}", info.model, info.version),
                )
                .await;
            }
            Err(_) => write_line(console, "Failed to read device info").await,
        },
        "on" => write_result(console, device.set_output(true).await, "Output ON").await,
        "off" => write_result(console, device.set_output(false).await, "Output OFF").await,
        "v" if tokens.len() >= 2 => {
            if let Ok(v) = tokens[1].parse::<f32>() {
                write_result(console, device.set_voltage(v).await, "Voltage updated").await;
            } else {
                write_line(console, "Invalid voltage").await;
            }
        }
        "i" if tokens.len() >= 2 => {
            if let Ok(i) = tokens[1].parse::<f32>() {
                write_result(console, device.set_current(i).await, "Current updated").await;
            } else {
                write_line(console, "Invalid current").await;
            }
        }
        "vi" if tokens.len() >= 3 => {
            if let (Ok(v), Ok(i)) = (tokens[1].parse::<f32>(), tokens[2].parse::<f32>()) {
                write_result(
                    console,
                    device.set_voltage_current(v, i).await,
                    "Voltage/current updated",
                )
                .await;
            } else {
                write_line(console, "Invalid vi values").await;
            }
        }
        "lock" => write_result(console, device.set_key_lock(true).await, "Key lock enabled").await,
        "unlock" => write_result(console, device.set_key_lock(false).await, "Key lock disabled").await,
        "status" | "read" => match device.read_output_status().await {
            Ok(s) => {
                write_line(
                    console,
                    &format!(
                        "OUT={} V={:.2}V I={:.3}A P={:.2}W VIN={:.2}V",
                        if s.output_enabled { "ON" } else { "OFF" },
                        s.output_voltage,
                        s.output_current,
                        s.output_power,
                        s.input_voltage
                    ),
                )
                .await;
            }
            Err(_) => write_line(console, "Failed to read output status").await,
        },
        "temp" => match device.read_temperature_c().await {
            Ok(t) => write_line(console, &format!("Internal temp: {:.1} C", t)).await,
            Err(_) => write_line(console, "Failed to read temperature").await,
        },
        "all" => match device.read_live_measurements().await {
            Ok((v, i, p, vin, t)) => {
                write_line(
                    console,
                    &format!(
                        "V={:.2}V I={:.3}A P={:.2}W VIN={:.2}V T={:.1}C",
                        v, i, p, vin, t
                    ),
                )
                .await;
            }
            Err(_) => write_line(console, "Failed to read measurements").await,
        },
        "ovp" if tokens.len() >= 2 => parse_f32_and_apply(console, tokens[1], |v| device.set_ovp(v), "OVP updated").await,
        "ocp" if tokens.len() >= 2 => parse_f32_and_apply(console, tokens[1], |v| device.set_ocp(v), "OCP updated").await,
        "opp" if tokens.len() >= 2 => parse_f32_and_apply(console, tokens[1], |v| device.set_opp(v), "OPP updated").await,
        "lvp" if tokens.len() >= 2 => parse_f32_and_apply(console, tokens[1], |v| device.set_lvp(v), "LVP updated").await,
        "otp" if tokens.len() >= 2 => parse_f32_and_apply(console, tokens[1], |v| device.set_otp(v), "OTP updated").await,
        "btf" if tokens.len() >= 2 => {
            parse_f32_and_apply(console, tokens[1], |v| device.set_battery_cutoff(v), "Battery cutoff updated").await
        }
        "mppt" if tokens.len() >= 2 => {
            let on = tokens[1] == "on";
            write_result(console, device.set_mppt_enable(on).await, "MPPT updated").await;
        }
        "mpptthr" if tokens.len() >= 2 => {
            if let Ok(thr) = tokens[1].parse::<u8>() {
                write_result(
                    console,
                    device.set_mppt_threshold_percent(thr.min(100)).await,
                    "MPPT threshold updated",
                )
                .await;
            } else {
                write_line(console, "Invalid mpptthr").await;
            }
        }
        "cpmode" if tokens.len() >= 2 => {
            let on = tokens[1] == "on";
            write_result(console, device.set_cp_mode(on).await, "CP mode updated").await;
        }
        "cp" if tokens.len() >= 2 => {
            parse_f32_and_apply(console, tokens[1], |v| device.set_cp_value(v), "CP value updated").await
        }
        "tempunit" if tokens.len() >= 2 => {
            let f = matches!(tokens[1], "f" | "F");
            write_result(
                console,
                device.set_temperature_unit_fahrenheit(f).await,
                "Temperature unit updated",
            )
            .await;
        }
        "group" if tokens.len() >= 2 => {
            if let Ok(g) = tokens[1].parse::<u8>() {
                write_result(console, device.call_memory_group(g).await, "Memory group applied").await;
            } else {
                write_line(console, "Invalid group").await;
            }
        }
        "readgroup" if tokens.len() >= 2 => {
            if let Ok(g) = tokens[1].parse::<u8>() {
                match device.read_memory_group(g).await {
                    Ok(values) => {
                        write_line(console, &format!("Group {}: {:?}", g, values.as_slice())).await;
                    }
                    Err(_) => write_line(console, "Failed to read memory group").await,
                }
            } else {
                write_line(console, "Invalid group").await;
            }
        }
        "readreg" if tokens.len() >= 3 => {
            if let (Ok(addr), Ok(count)) = (parse_u16(tokens[1]), parse_u16(tokens[2])) {
                match device.read_registers(addr, count).await {
                    Ok(values) => write_line(console, &format!("0x{addr:04X}: {:?}", values.as_slice())).await,
                    Err(_) => write_line(console, "Failed to read registers").await,
                }
            } else {
                write_line(console, "Invalid readreg args").await;
            }
        }
        "writereg" if tokens.len() >= 3 => {
            if let (Ok(addr), Ok(value)) = (parse_u16(tokens[1]), parse_u16(tokens[2])) {
                write_result(console, device.write_register(addr, value).await, "Register written").await;
            } else {
                write_line(console, "Invalid writereg args").await;
            }
        }
        "mwrite" if tokens.len() >= 3 => {
            if let Ok(addr) = parse_u16(tokens[1]) {
                let mut values: Vec<u16> = Vec::new();
                let mut ok = true;
                for tok in &tokens[2..] {
                    if let Ok(v) = parse_u16(tok) {
                        values.push(v);
                    } else {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    write_result(console, device.write_registers(addr, &values).await, "Registers written").await;
                } else {
                    write_line(console, "Invalid mwrite values").await;
                }
            } else {
                write_line(console, "Invalid mwrite address").await;
            }
        }
        "prot" => match device.read_protection_status_raw().await {
            Ok(raw) => write_line(console, &format!("Protection flags: 0x{raw:04X}")).await,
            Err(_) => write_line(console, "Failed to read protection status").await,
        },
        "reset" => write_result(
            console,
            device.restore_factory_defaults().await,
            "Factory reset command sent",
        )
        .await,
        _ => write_line(console, "Unknown command. Type 'help'.").await,
    }
}

async fn parse_f32_and_apply<CON, F, Fut>(console: &mut CON, raw: &str, f: F, ok_msg: &str)
where
    CON: Read + Write + ErrorType,
    F: FnOnce(f32) -> Fut,
    Fut: core::future::Future<Output = Result<(), crate::modbus::ModbusError>>,
{
    if let Ok(v) = raw.parse::<f32>() {
        write_result(console, f(v).await, ok_msg).await;
    } else {
        write_line(console, "Invalid number").await;
    }
}

fn parse_u16(s: &str) -> Result<u16, ()> {
    if let Some(stripped) = s.strip_prefix("0x") {
        u16::from_str_radix(stripped, 16).map_err(|_| ())
    } else {
        s.parse::<u16>().map_err(|_| ())
    }
}

async fn show_help<CON>(console: &mut CON)
where
    CON: Read + Write + ErrorType,
{
    write_line(console, "Commands:").await;
    write_line(console, "  info | status | read | temp | all").await;
    write_line(console, "  on | off | lock | unlock").await;
    write_line(console, "  v <V> | i <A> | vi <V> <A>").await;
    write_line(console, "  ovp/ocp/opp/lvp/otp <value> | btf <A>").await;
    write_line(console, "  mppt on|off | mpptthr <0..100>").await;
    write_line(console, "  cpmode on|off | cp <W>").await;
    write_line(console, "  tempunit c|f").await;
    write_line(console, "  group <0..9> | readgroup <0..9>").await;
    write_line(console, "  readreg <addr> <count>").await;
    write_line(console, "  writereg <addr> <value>").await;
    write_line(console, "  mwrite <addr> <v1> <v2> ...").await;
    write_line(console, "  prot | reset").await;
}

async fn read_line<CON>(console: &mut CON) -> String
where
    CON: Read + Write + ErrorType,
{
    let mut out = String::new();
    let mut byte = [0u8; 1];

    loop {
        if console.read(&mut byte).await.is_err() {
            continue;
        }

        let b = byte[0];
        if b == b'\r' || b == b'\n' {
            break;
        }

        if out.len() < 128 {
            out.push(b as char);
        }
    }

    out
}

async fn write_result<CON>(
    console: &mut CON,
    result: Result<(), crate::modbus::ModbusError>,
    ok: &str,
) where
    CON: Read + Write + ErrorType,
{
    if result.is_ok() {
        write_line(console, ok).await;
    } else {
        write_line(console, "Command failed").await;
    }
}

async fn write_line<CON>(console: &mut CON, line: &str)
where
    CON: Read + Write + ErrorType,
{
    write_raw(console, line.as_bytes()).await;
    write_raw(console, b"\r\n").await;
}

async fn write_raw<CON>(console: &mut CON, data: &[u8])
where
    CON: Read + Write + ErrorType,
{
    let _ = console.write_all(data).await;
    let _ = console.flush().await;
}

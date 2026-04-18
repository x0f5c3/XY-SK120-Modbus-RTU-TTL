# xy-sk120-esp-rs

`no_std` ESP32-S3 firmware crate generated with `esp-generate`, focused only on Modbus RTU control of XY-SK120 class power supplies.

## Scope

- ✅ Modbus RTU device control logic (register reads/writes, scaling, memory groups)
- ✅ Async serial TUI command menu (Embassy-based runtime)
- ✅ Optional compile-time MQTT configuration via `toml_cfg`
- ❌ No web UI code

## Board Choice

This crate targets **ESP32-S3** (same family used by the current hardware wiring), which keeps UART pin mapping and bring-up simpler for this migration.

## Wiring (default)

- Modbus TX: `GPIO6`
- Modbus RX: `GPIO7`
- Modbus slave: `1`
- Modbus baud: `9600`

## Build Prerequisites

- esp-rs toolchain (`espup`)
- `espflash`

## Run

```bash
cd xy-sk120-esp-rs
cargo run
```

## TUI Commands

- `help`, `info`, `status`, `all`, `temp`
- `on`, `off`, `lock`, `unlock`
- `v <V>`, `i <A>`, `vi <V> <A>`
- `ovp <V>`, `ocp <A>`, `opp <W>`, `lvp <V>`, `otp <C>`, `btf <A>`
- `mppt on|off`, `mpptthr <0..100>`
- `cpmode on|off`, `cp <W>`
- `group <0..9>`, `readgroup <0..9>`
- `readreg <addr> <count>`, `writereg <addr> <value>`, `mwrite <addr> <v1> ...`
- `prot`, `reset`

## Compile-time MQTT config (`toml_cfg`)

Defined in `Cargo.toml` under `[package.metadata.toml_cfg]`:

```toml
[package.metadata.toml_cfg]
mqtt_enabled = {}
mqtt_broker_url = "mqtt://broker.local:1883"
mqtt_topic_prefix = "xy-sk120"
```

`build.rs` emits these as cfg flags with `toml_cfg::emit_cfgs()`.

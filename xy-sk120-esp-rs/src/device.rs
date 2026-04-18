use heapless::Vec;

use crate::{
    modbus::{ModbusError, ModbusRtuClient},
    registers::{
        DATA_GROUP_BASE_ADDR, DATA_GROUP_REGISTERS, DATA_GROUP_SIZE, REG_BTF, REG_CP_ENABLE,
        REG_CP_SET, REG_EXTRACT_M, REG_F_C, REG_FACTORY_RESET, REG_I_SET, REG_LOCK, REG_MODEL,
        REG_MPPT_ENABLE, REG_MPPT_THRESHOLD, REG_ONOFF, REG_PROTECT, REG_S_LVP, REG_S_OCP,
        REG_S_OPP, REG_S_OTP, REG_S_OVP, REG_T_IN, REG_V_SET, REG_VERSION, REG_VOUT,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct DeviceInfo {
    pub model: u16,
    pub version: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct OutputStatus {
    pub output_voltage: f32,
    pub output_current: f32,
    pub output_power: f32,
    pub input_voltage: f32,
    pub output_enabled: bool,
}

pub struct DeviceController<IO> {
    modbus: ModbusRtuClient<IO>,
}

impl<IO> DeviceController<IO>
where
    IO: embedded_io_async::Read + embedded_io_async::Write + embedded_io::ErrorType,
{
    pub fn new(modbus: ModbusRtuClient<IO>) -> Self {
        Self { modbus }
    }

    pub async fn read_info(&mut self) -> Result<DeviceInfo, ModbusError> {
        let mut buf = [0u16; 1];
        self.modbus
            .read_holding_registers(REG_MODEL, 1, &mut buf)
            .await?;
        let model = buf[0];
        self.modbus
            .read_holding_registers(REG_VERSION, 1, &mut buf)
            .await?;
        Ok(DeviceInfo {
            model,
            version: buf[0],
        })
    }

    pub async fn set_voltage(&mut self, voltage_v: f32) -> Result<(), ModbusError> {
        let scaled = voltage_v * 100.0;
        let value = if scaled.is_finite() {
            scaled.clamp(0.0, u16::MAX as f32) as u16
        } else {
            0
        };
        self.modbus.write_single_register(REG_V_SET, value).await
    }

    pub async fn set_current(&mut self, current_a: f32) -> Result<(), ModbusError> {
        let scaled = current_a * 1000.0;
        let value = if scaled.is_finite() {
            scaled.clamp(0.0, u16::MAX as f32) as u16
        } else {
            0
        };
        self.modbus.write_single_register(REG_I_SET, value).await
    }

    pub async fn set_voltage_current(
        &mut self,
        voltage_v: f32,
        current_a: f32,
    ) -> Result<(), ModbusError> {
        self.set_voltage(voltage_v).await?;
        self.set_current(current_a).await
    }

    pub async fn set_output(&mut self, enabled: bool) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_ONOFF, if enabled { 1 } else { 0 })
            .await
    }

    pub async fn set_key_lock(&mut self, locked: bool) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_LOCK, if locked { 1 } else { 0 })
            .await
    }

    pub async fn read_output_status(&mut self) -> Result<OutputStatus, ModbusError> {
        let mut reg = [0u16; 4];
        self.modbus
            .read_holding_registers(REG_VOUT, 4, &mut reg)
            .await?;

        let mut onoff = [0u16; 1];
        self.modbus
            .read_holding_registers(REG_ONOFF, 1, &mut onoff)
            .await?;

        Ok(OutputStatus {
            output_voltage: reg[0] as f32 / 100.0,
            output_current: reg[1] as f32 / 1000.0,
            output_power: reg[2] as f32 / 100.0,
            input_voltage: reg[3] as f32 / 100.0,
            output_enabled: onoff[0] != 0,
        })
    }

    pub async fn read_temperature_c(&mut self) -> Result<f32, ModbusError> {
        let mut reg = [0u16; 1];
        self.modbus
            .read_holding_registers(REG_T_IN, 1, &mut reg)
            .await?;
        Ok(reg[0] as f32 / 10.0)
    }

    pub async fn set_ovp(&mut self, volts: f32) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_S_OVP, (volts * 100.0) as u16)
            .await
    }

    pub async fn set_ocp(&mut self, amps: f32) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_S_OCP, (amps * 1000.0) as u16)
            .await
    }

    pub async fn set_opp(&mut self, watts: f32) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_S_OPP, (watts * 10.0) as u16)
            .await
    }

    pub async fn set_lvp(&mut self, volts: f32) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_S_LVP, (volts * 100.0) as u16)
            .await
    }

    pub async fn set_otp(&mut self, temp: f32) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_S_OTP, (temp * 10.0) as u16)
            .await
    }

    pub async fn set_battery_cutoff(&mut self, amps: f32) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_BTF, (amps * 1000.0) as u16)
            .await
    }

    pub async fn set_mppt_enable(&mut self, enabled: bool) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_MPPT_ENABLE, if enabled { 1 } else { 0 })
            .await
    }

    pub async fn set_mppt_threshold_percent(
        &mut self,
        threshold_percent: u8,
    ) -> Result<(), ModbusError> {
        let threshold = (threshold_percent as u16).min(100);
        self.modbus
            .write_single_register(REG_MPPT_THRESHOLD, threshold)
            .await
    }

    pub async fn set_cp_mode(&mut self, enabled: bool) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_CP_ENABLE, if enabled { 1 } else { 0 })
            .await
    }

    pub async fn set_cp_value(&mut self, watts: f32) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_CP_SET, (watts * 10.0) as u16)
            .await
    }

    pub async fn set_temperature_unit_fahrenheit(
        &mut self,
        fahrenheit: bool,
    ) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_F_C, if fahrenheit { 1 } else { 0 })
            .await
    }

    pub async fn read_protection_status_raw(&mut self) -> Result<u16, ModbusError> {
        let mut reg = [0u16; 1];
        self.modbus
            .read_holding_registers(REG_PROTECT, 1, &mut reg)
            .await?;
        Ok(reg[0])
    }

    pub async fn read_registers(
        &mut self,
        address: u16,
        count: u16,
    ) -> Result<Vec<u16, 32>, ModbusError> {
        let count = count.min(32);
        let mut tmp = [0u16; 32];
        let got = self
            .modbus
            .read_holding_registers(address, count, &mut tmp)
            .await?;
        let mut out = Vec::<u16, 32>::new();
        for item in tmp.iter().take(got) {
            out.push(*item).map_err(|_| ModbusError::Protocol)?;
        }
        Ok(out)
    }

    pub async fn write_register(&mut self, address: u16, value: u16) -> Result<(), ModbusError> {
        self.modbus.write_single_register(address, value).await
    }

    pub async fn write_registers(
        &mut self,
        address: u16,
        values: &[u16],
    ) -> Result<(), ModbusError> {
        self.modbus.write_multiple_registers(address, values).await
    }

    pub async fn call_memory_group(&mut self, group: u8) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_EXTRACT_M, (group as u16).min(9))
            .await
    }

    pub async fn read_memory_group(&mut self, group: u8) -> Result<Vec<u16, 14>, ModbusError> {
        let start = DATA_GROUP_BASE_ADDR + (group as u16).min(9) * DATA_GROUP_SIZE;
        let mut tmp = [0u16; 14];
        self.modbus
            .read_holding_registers(start, DATA_GROUP_REGISTERS, &mut tmp)
            .await?;
        let mut out = Vec::<u16, 14>::new();
        for v in tmp {
            out.push(v).map_err(|_| ModbusError::Protocol)?;
        }
        Ok(out)
    }

    pub async fn restore_factory_defaults(&mut self) -> Result<(), ModbusError> {
        self.modbus
            .write_single_register(REG_FACTORY_RESET, 0x0001)
            .await
    }

    pub async fn read_live_measurements(
        &mut self,
    ) -> Result<(f32, f32, f32, f32, f32), ModbusError> {
        let status = self.read_output_status().await?;
        let temp = self.read_temperature_c().await?;
        Ok((
            status.output_voltage,
            status.output_current,
            status.output_power,
            status.input_voltage,
            temp,
        ))
    }
}

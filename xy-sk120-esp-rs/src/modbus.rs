use alloc::vec::Vec;

use embassy_time::{Duration, Timer};
use embedded_io::ErrorType;
use embedded_io_async::{Read, Write};

use crate::registers::{MB_FUNC_READ_HOLDING, MB_FUNC_WRITE_MULTIPLE, MB_FUNC_WRITE_SINGLE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModbusError {
    Io,
    FrameTooShort,
    Crc,
    InvalidSlave,
    InvalidFunction,
    Protocol,
    BufferTooSmall,
}

pub struct ModbusRtuClient<IO> {
    io: IO,
    slave: u8,
    silent_interval: Duration,
}

impl<IO> ModbusRtuClient<IO>
where
    IO: Read + Write + ErrorType,
{
    pub fn new(io: IO, slave: u8, silent_interval: Duration) -> Self {
        Self {
            io,
            slave,
            silent_interval,
        }
    }

    pub fn set_slave(&mut self, slave: u8) {
        self.slave = slave;
    }

    pub async fn read_holding_registers(
        &mut self,
        addr: u16,
        count: u16,
        out: &mut [u16],
    ) -> Result<usize, ModbusError> {
        if out.len() < count as usize {
            return Err(ModbusError::BufferTooSmall);
        }

        let mut req = [0u8; 8];
        req[0] = self.slave;
        req[1] = MB_FUNC_READ_HOLDING;
        req[2..4].copy_from_slice(&addr.to_be_bytes());
        req[4..6].copy_from_slice(&count.to_be_bytes());
        let crc = crc16(&req[..6]);
        req[6..8].copy_from_slice(&crc.to_le_bytes());

        let expected_len = 5 + (count as usize * 2);
        let mut resp = Vec::new();
        resp.resize(expected_len, 0);

        let received = self.transceive(&req, &mut resp).await?;
        if received < 5 {
            return Err(ModbusError::FrameTooShort);
        }

        if resp[0] != self.slave {
            return Err(ModbusError::InvalidSlave);
        }
        if resp[1] != MB_FUNC_READ_HOLDING {
            return Err(ModbusError::InvalidFunction);
        }

        let byte_count = resp[2] as usize;
        if byte_count != count as usize * 2 {
            return Err(ModbusError::Protocol);
        }

        for i in 0..count as usize {
            let base = 3 + i * 2;
            out[i] = u16::from_be_bytes([resp[base], resp[base + 1]]);
        }

        Ok(count as usize)
    }

    pub async fn write_single_register(
        &mut self,
        addr: u16,
        value: u16,
    ) -> Result<(), ModbusError> {
        let mut req = [0u8; 8];
        req[0] = self.slave;
        req[1] = MB_FUNC_WRITE_SINGLE;
        req[2..4].copy_from_slice(&addr.to_be_bytes());
        req[4..6].copy_from_slice(&value.to_be_bytes());
        let crc = crc16(&req[..6]);
        req[6..8].copy_from_slice(&crc.to_le_bytes());

        let mut resp = [0u8; 8];
        let received = self.transceive(&req, &mut resp).await?;
        if received < 8 {
            return Err(ModbusError::FrameTooShort);
        }

        if resp[0] != self.slave {
            return Err(ModbusError::InvalidSlave);
        }
        if resp[1] != MB_FUNC_WRITE_SINGLE {
            return Err(ModbusError::InvalidFunction);
        }

        Ok(())
    }

    pub async fn write_multiple_registers(
        &mut self,
        addr: u16,
        values: &[u16],
    ) -> Result<(), ModbusError> {
        let count = values.len() as u16;
        let byte_count = (count * 2) as u8;

        let mut req = Vec::new();
        req.push(self.slave).map_err(|_| ModbusError::Protocol)?;
        req.push(MB_FUNC_WRITE_MULTIPLE)
            .map_err(|_| ModbusError::Protocol)?;
        req.extend_from_slice(&addr.to_be_bytes())
            .map_err(|_| ModbusError::Protocol)?;
        req.extend_from_slice(&count.to_be_bytes())
            .map_err(|_| ModbusError::Protocol)?;
        req.push(byte_count).map_err(|_| ModbusError::Protocol)?;
        for v in values {
            req.extend_from_slice(&v.to_be_bytes())
                .map_err(|_| ModbusError::Protocol)?;
        }
        let crc = crc16(&req);
        req.extend_from_slice(&crc.to_le_bytes())
            .map_err(|_| ModbusError::Protocol)?;

        let mut resp = [0u8; 8];
        let received = self.transceive(req.as_slice(), &mut resp).await?;
        if received < 8 {
            return Err(ModbusError::FrameTooShort);
        }
        if resp[0] != self.slave {
            return Err(ModbusError::InvalidSlave);
        }
        if resp[1] != MB_FUNC_WRITE_MULTIPLE {
            return Err(ModbusError::InvalidFunction);
        }

        Ok(())
    }

    async fn transceive(&mut self, req: &[u8], resp: &mut [u8]) -> Result<usize, ModbusError> {
        Timer::after(self.silent_interval).await;

        self.io.write_all(req).await.map_err(|_| ModbusError::Io)?;
        self.io.flush().await.map_err(|_| ModbusError::Io)?;

        Timer::after(self.silent_interval).await;

        self.io
            .read_exact(resp)
            .await
            .map_err(|_| ModbusError::Io)?;

        let len = resp.len();
        if len < 4 {
            return Err(ModbusError::FrameTooShort);
        }
        let frame_crc = u16::from_le_bytes([resp[len - 2], resp[len - 1]]);
        let calc = crc16(&resp[..len - 2]);
        if frame_crc != calc {
            return Err(ModbusError::Crc);
        }

        Ok(len)
    }
}

pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for b in data {
        crc ^= *b as u16;
        for _ in 0..8 {
            if (crc & 0x0001) != 0 {
                crc >>= 1;
                crc ^= 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

use std::io::Error;
use std::net::SocketAddr;
use tokio_modbus::client::{Context, Reader, tcp, Writer};
use tokio_modbus::slave::{Slave, SlaveContext};
use crate::victron::VictronError;

pub(crate) struct VictronClient {
    client: Context,
}

impl From<std::io::Error> for VictronError {
    fn from(e: Error) -> Self {
        Self(e.to_string())
    }
}

impl VictronClient {
    pub async fn new(addr: SocketAddr) -> Result<Self, VictronError> {
        let ctx = tcp::connect(addr).await?;
        Ok(Self { client: ctx })
    }

    pub fn set_unit(&mut self, unit: u8) {
        self.client.set_slave(Slave(unit))
    }

    pub async fn write_i16(&mut self, addr: u16, value: i16) -> Result<(), VictronError> {
        Ok(self.write_u16(addr, value as u16).await?)
    }

    pub async fn write_u16(&mut self, addr: u16, value: u16) -> Result<(), VictronError> {
        Ok(self.client.write_single_register(addr, value).await?)
    }

    pub async fn read_bool(&mut self, addr: u16) -> Result<bool, VictronError> {
        Ok(match dbg!(self.read_u16(addr).await?) {
            0 => false,
            1 => true,
            _ => return Err(VictronError("Unknown bool state!".to_owned()))
        })
    }

    pub async fn read_i16(&mut self, addr: u16) -> Result<i16, VictronError> {
        Ok(self.read_u16(addr).await? as i16)
    }

    pub async fn read_u16(&mut self, addr: u16) -> Result<u16, VictronError> {
        let v = self.client.read_input_registers(addr, 1).await
            .map_err(|e| VictronError(e.to_string()))?;
        Ok(v[0] as u16)
    }
}
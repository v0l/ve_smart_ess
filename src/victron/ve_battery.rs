use std::net::SocketAddr;
use crate::victron::client::VictronClient;
use crate::VictronError;

pub struct VictronBattery {
    client: VictronClient,
}

impl VictronBattery {
    pub async fn new(addr: SocketAddr, unit: u8) -> Result<Self, VictronError> {
        let mut cli = VictronClient::new(addr).await?;
        cli.set_unit(unit);
        Ok(Self { client: cli })
    }

    pub async fn capacity(&mut self) -> Result<f32, VictronError> {
        let v = self.client.read_u16(309).await?;
        Ok(v as f32 / 10.0)
    }
}
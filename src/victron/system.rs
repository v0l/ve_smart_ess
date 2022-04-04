use std::net::SocketAddr;
use crate::victron::client::VictronClient;
use crate::VictronError;

pub struct VictronSystem {
    client: VictronClient,
}

impl VictronSystem {
    pub async fn new(addr: SocketAddr, unit: u8) -> Result<Self, VictronError> {
        let mut cli = VictronClient::new(addr).await?;
        cli.set_unit(unit);
        Ok(Self { client: cli })
    }


}
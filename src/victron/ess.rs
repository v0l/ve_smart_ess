use std::net::SocketAddr;
use crate::victron::client::{VictronClient};
use crate::victron::Line;
use crate::VictronError;

pub struct VictronESS {
    client: VictronClient,
}

pub enum ESSRegister {
    /// Value in watts.
    /// Positive values take power from grid.
    /// Negative values feed into the grid.
    PowerSetPoint(Line, i16),

    /// Disable charger 0/1
    DisableCharge(bool),
    /// Disable feed-in to grid 0/1
    DisableFeedIn(bool),
}

impl VictronESS {
    pub async fn new(addr: SocketAddr, unit: u8) -> Result<Self, VictronError> {
        let mut cli = VictronClient::new(addr).await?;
        cli.set_unit(unit);
        Ok(Self { client: cli })
    }

    pub async fn get_param(&mut self, reg: ESSRegister) -> Result<ESSRegister, VictronError> {
        let addr = self.map_register(&reg);
        Ok(match reg {
            ESSRegister::PowerSetPoint(l, _) => {
                ESSRegister::PowerSetPoint(l, self.client.read_i16(addr).await?)
            }
            ESSRegister::DisableCharge(_) => {
                ESSRegister::DisableCharge(self.client.read_bool(addr).await?)
            }
            ESSRegister::DisableFeedIn(_) => {
                ESSRegister::DisableFeedIn(self.client.read_bool(addr).await?)
            }
        })
    }

    pub async fn set_param(&mut self, reg: ESSRegister) -> Result<(), VictronError> {
        let addr = self.map_register(&reg);
        Ok(match reg {
            ESSRegister::PowerSetPoint(_, power) => {
                self.client.write_i16(addr, power).await?
            }
            ESSRegister::DisableCharge(v) => {
                self.client.write_u16(addr, if v { 1 } else { 0 }).await?
            }
            ESSRegister::DisableFeedIn(v) => {
                self.client.write_u16(addr, if v { 1 } else { 0 }).await?
            }
        })
    }

    fn map_register(&self, reg: &ESSRegister) -> u16 {
        match reg {
            ESSRegister::PowerSetPoint(l, _) => match l {
                Line::L1 => 37,
                Line::L2 => 40,
                Line::L3 => 41,
            },
            ESSRegister::DisableCharge(_) => 38,
            ESSRegister::DisableFeedIn(_) => 39,
        }
    }
}
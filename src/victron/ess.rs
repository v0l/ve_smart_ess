use std::net::SocketAddr;
use crate::victron::client::{VictronClient};
use crate::victron::Line;
use crate::VictronError;

pub struct VictronESS {
    client: VictronClient,
}

pub enum ESS {
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

    pub async fn get_param(&mut self, reg: ESS) -> Result<ESS, VictronError> {
        let addr = self.map_register(&reg);
        Ok(match reg {
            ESS::PowerSetPoint(l, _) => {
                ESS::PowerSetPoint(l, self.client.read_i16(addr).await?)
            }
            ESS::DisableCharge(_) => {
                ESS::DisableCharge(self.client.read_bool(addr).await?)
            }
            ESS::DisableFeedIn(_) => {
                ESS::DisableFeedIn(self.client.read_bool(addr).await?)
            }
        })
    }

    pub async fn set_param(&mut self, reg: ESS) -> Result<(), VictronError> {
        let addr = self.map_register(&reg);
        Ok(match reg {
            ESS::PowerSetPoint(_, power) => {
                self.client.write_i16(addr, power).await?
            }
            ESS::DisableCharge(v) => {
                self.client.write_u16(addr, if v { 1 } else { 0 }).await?
            }
            ESS::DisableFeedIn(v) => {
                self.client.write_u16(addr, if v { 1 } else { 0 }).await?
            }
        })
    }

    fn map_register(&self, reg: &ESS) -> u16 {
        match reg {
            ESS::PowerSetPoint(l, _) => match l {
                Line::L1 => 37,
                Line::L2 => 40,
                Line::L3 => 41,
            },
            ESS::DisableCharge(_) => 38,
            ESS::DisableFeedIn(_) => 39,
        }
    }
}
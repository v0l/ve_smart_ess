use std::net::SocketAddr;
use crate::victron::client::{VictronClient};
use crate::victron::Line;
use crate::VictronError;

pub struct VictronESS {
    client: VictronClient,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ESS {
    /// Value in watts.
    /// Positive values take power from grid.
    /// Negative values feed into the grid.
    PowerSetPoint(Line, i16),

    /// Control charge load, 0 = Disabled charge, 100 = Full charge power
    ChargePower(u8),

    /// Control feed-in from battery, 0 = Disabled, 100 = Full feed-in power
    FeedInPower(u8),
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
            ESS::ChargePower(_) => {
                ESS::ChargePower(self.client.read_u16(addr).await? as u8)
            }
            ESS::FeedInPower(_) => {
                ESS::FeedInPower(self.client.read_u16(addr).await? as u8)
            }
        })
    }

    pub async fn set_param(&mut self, reg: ESS) -> Result<(), VictronError> {
        let addr = self.map_register(&reg);
        Ok(match reg {
            ESS::PowerSetPoint(_, power) => {
                self.client.write_i16(addr, power).await?
            }
            ESS::ChargePower(v) => {
                self.client.write_u16(addr, v as u16).await?
            }
            ESS::FeedInPower(v) => {
                self.client.write_u16(addr, v as u16).await?
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
            ESS::ChargePower(_) => 38,
            ESS::FeedInPower(_) => 39,
        }
    }
}
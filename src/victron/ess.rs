use std::net::SocketAddr;
use crate::victron::client::{VictronClient};
use crate::victron::Line;
use crate::VictronError;

pub struct VictronESS {
    client: VictronClient,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Hub4Mode {
    WithPhaseCompensation = 1,
    WithoutPhaseCompensation = 2,
    External = 3,
}

impl TryFrom<u16> for Hub4Mode {
    type Error = VictronError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Hub4Mode::WithPhaseCompensation,
            2 => Hub4Mode::WithoutPhaseCompensation,
            3 => Hub4Mode::External,
            e => return Err(VictronError(format!("Invalid Hub4 mode {}!", e)))
        })
    }
}

impl ToString for Hub4Mode {
    fn to_string(&self) -> String {
        match self {
            Hub4Mode::WithPhaseCompensation => "ESS with phase compensation",
            Hub4Mode::WithoutPhaseCompensation => "ESS without phase compensation",
            Hub4Mode::External => "Disabled / External Control",
        }.to_owned()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Register {
    /// Value in watts.
    /// Positive values take power from grid.
    /// Negative values feed into the grid.
    PowerSetPoint(Line, i16),

    /// Control charger
    DisableCharge(bool),

    /// Control feed-in from battery
    DisableFeedIn(bool),

    /// ESS Mode
    Mode(Hub4Mode),
}

impl VictronESS {
    pub async fn new(addr: SocketAddr, unit: u8) -> Result<Self, VictronError> {
        let mut cli = VictronClient::new(addr).await?;
        cli.set_unit(unit);
        Ok(Self { client: cli })
    }

    pub async fn get_param(&mut self, reg: Register) -> Result<Register, VictronError> {
        let addr = self.map_register(&reg);
        Ok(match reg {
            Register::PowerSetPoint(l, _) => {
                Register::PowerSetPoint(l, self.client.read_i16(addr).await?)
            }
            Register::DisableCharge(_) => {
                Register::DisableCharge(self.client.read_u16(addr).await? as u8 == 1)
            }
            Register::DisableFeedIn(_) => {
                Register::DisableFeedIn(self.client.read_u16(addr).await? as u8 == 1)
            },
            Register::Mode(_) => {
                Register::Mode(Hub4Mode::try_from(self.client.read_u16(addr).await?)?)
            }
        })
    }

    pub async fn set_param(&mut self, reg: Register) -> Result<(), VictronError> {
        let addr = self.map_register(&reg);
        Ok(match reg {
            Register::PowerSetPoint(_, power) => {
                self.client.write_i16(addr, power).await?
            }
            Register::DisableCharge(v) => {
                self.client.write_u16(addr, if v { 100 } else { 0 }).await?
            }
            Register::DisableFeedIn(v) => {
                self.client.write_u16(addr, if v { 100 } else { 0 }).await?
            }
            Register::Mode(v) => {
                self.client.write_u16(addr, v as u16).await?
            }
        })
    }

    fn map_register(&self, reg: &Register) -> u16 {
        match reg {
            Register::PowerSetPoint(l, _) => match l {
                Line::L1 => 37,
                Line::L2 => 40,
                Line::L3 => 41,
            },
            Register::DisableCharge(_) => 38,
            Register::DisableFeedIn(_) => 39,
            Register::Mode(_) => 2902
        }
    }
}
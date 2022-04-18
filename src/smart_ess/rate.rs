use std::cmp::Ordering;
use std::ops::Index;
use chrono::{Datelike, DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::smart_ess::window::RateWindow;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rate {
    /// Name of this rate
    pub name: String,

    /// Fiat cost per 1 kW/h
    pub unit_cost: f32,

    /// Rate start and end times
    pub windows: Vec<RateWindow>,

    /// Controls stored energy usage during this rate
    pub discharge: RateDischarge,

    /// Controls charging during this rate
    pub charge: RateCharge,

    /// Number of units to be reserved for this rate until next charge
    pub reserve: i16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RateDischarge {
    /// Discharge disabled
    None,

    /// Percentage of current inverter load
    Capacity(f32),

    /// Drain capacity dynamically until to the end of the rate window
    Spread,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RateCharge {
    /// Charger mode
    mode: ChargeMode,

    /// Limit number of units that can be consumed by the charger in this rate.
    unit_limit: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ChargeMode {
    /// Charger is disabled
    Disabled,

    /// target minimum battery capacity
    Capacity(f32),
}
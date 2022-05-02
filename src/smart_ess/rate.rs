use crate::smart_ess::window::{RateWindow, RateWindowAbsolute};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub reserve: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum RateDischarge {
    /// Discharge disabled
    None,

    /// Percentage of current inverter load
    Capacity(f32),

    /// Drain capacity dynamically until to the end of the rate window
    Spread,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct RateCharge {
    /// Charger mode
    mode: ChargeMode,

    /// Limit number of units that can be consumed by the charger in this rate.
    unit_limit: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ChargeMode {
    /// Charger is disabled
    Disabled,

    /// target minimum battery capacity
    Capacity(f32),
}

impl Rate {
    pub fn schedule(&self, from: DateTime<Utc>) -> Vec<RateWindowAbsolute> {
        let mut ret: Vec<RateWindowAbsolute> = self
            .windows
            .iter()
            .map(|w| w.schedule(from))
            .flatten()
            .collect();
        ret.sort_by(|a, b| a.start.cmp(&b.start));
        ret
    }
}

impl RateCharge {
    pub fn charge_enabled(&self) -> bool {
        self.mode != ChargeMode::Disabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_ess::window::{RateTime, ALL_WEEKDAYS};
    use chrono::TimeZone;
    use std::str::FromStr;

    #[test]
    fn next_from() {
        let rate = Rate {
            name: "test".to_owned(),
            unit_cost: 0.2,
            windows: vec![
                RateWindow {
                    start: RateTime::from_str("09:00").unwrap(),
                    end: RateTime::from_str("9:59").unwrap(),
                    days: ALL_WEEKDAYS.to_vec(),
                },
                RateWindow {
                    start: RateTime::from_str("11:00").unwrap(),
                    end: RateTime::from_str("11:59").unwrap(),
                    days: ALL_WEEKDAYS.to_vec(),
                },
            ],
            charge: RateCharge {
                mode: ChargeMode::Capacity(1.0),
                unit_limit: 0,
            },
            discharge: RateDischarge::None,
            reserve: 0.0,
        };

        let next = rate.schedule(Utc.ymd(2022, 04, 18).and_hms(8, 0, 0));
        assert_eq!(Utc.ymd(2022, 04, 18).and_hms(9, 0, 0), next[0].start);
        assert_eq!(Utc.ymd(2022, 04, 18).and_hms(11, 0, 0), next[1].start);

        let next = rate.schedule(Utc.ymd(2022, 04, 18).and_hms(9, 0, 0));
        assert_eq!(Utc.ymd(2022, 04, 18).and_hms(9, 0, 0), next[0].start);
        assert_eq!(Utc.ymd(2022, 04, 18).and_hms(11, 0, 0), next[1].start);

        let next = rate.schedule(Utc.ymd(2022, 04, 18).and_hms(10, 0, 0));
        assert_eq!(Utc.ymd(2022, 04, 18).and_hms(11, 0, 0), next[0].start);
        assert_eq!(Utc.ymd(2022, 04, 19).and_hms(9, 0, 0), next[1].start);
    }
}

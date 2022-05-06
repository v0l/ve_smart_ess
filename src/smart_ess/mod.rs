use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::smart_ess::rate::{Rate, RateDischarge};
use crate::smart_ess::window::RateWindowAbsolute;

pub mod rate;
pub mod window;

#[derive(Debug)]
pub struct ControllerError(pub String);

impl<TStr: ToString> From<TStr> for ControllerError {
    fn from(t: TStr) -> Self {
        ControllerError(t.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Controller {
    /// Rate tariffs
    rates: Vec<Rate>,

    /// Depth of Discharge
    dod: f32,
}

#[derive(Debug, Clone)]
pub struct Schedule {
    pub rate: Rate,
    pub window: RateWindowAbsolute,
}

#[derive(Debug, Clone)]
pub struct ControllerInputState {
    /// Power usage of the system in watts
    pub system_load: f32,

    /// Battery state of charge percent
    pub soc: f32,

    /// Battery capacity in kWh
    pub capacity: f32,

    /// Battery voltage
    pub voltage: f32,
}

#[derive(Debug, Clone)]
pub struct ControllerOutputState {
    pub disable_charge: bool,
    pub disable_feed_in: bool,
    pub soc: f32,

    /// Grid load in watts
    pub grid_load: f32,

    /// Battery load in watts
    pub battery_load: f32,

    /// Target battery usage in kWh
    pub using_capacity: f32,

    /// Reserve capacity for upcoming rates in kWh
    pub reserve_capacity: f32,

    pub current_rate: Schedule,

    pub next_rate: Schedule,

    pub next_charge: Schedule,
}

impl Display for ControllerOutputState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Grid Load: {} W\nBattery Load: {} W\nSoC: {}\nUsing: {} kWh\nReserve: {} kWh\nCurrent Rate: {} @ {}\nNext Rate: {} @ {}\nNext Charge: {} @ {}\nTime To Charge: {} min",
               self.grid_load,
               self.battery_load,
               self.soc,
               self.using_capacity,
               self.reserve_capacity,
               self.current_rate.rate.name,
               self.current_rate.window.start,
               self.next_rate.rate.name,
               self.next_rate.window.start,
               self.next_charge.rate.name,
               self.next_charge.window.start,
               (self.next_charge.window.start - Utc::now()).num_minutes())
    }
}

impl Controller {
    pub fn load() -> Result<Controller, ControllerError> {
        let path = "smart_ess.json";
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(_) => File::create(path)?,
        };
        let mut json = String::new();
        file.read_to_string(&mut json)?;
        let v: Controller = serde_json::from_str(&json)?;
        Ok(v)
    }

    pub fn next_charge(&self, from: DateTime<Utc>) -> Result<Schedule, ControllerError> {
        if let Some(v) = self
            .get_schedule(from)
            .iter()
            .filter(|s| s.rate.charge.charge_enabled())
            .next()
        {
            Ok(v.clone())
        } else {
            Err(ControllerError("No rate found!".to_owned()))
        }
    }

    pub fn get_schedule(&self, from: DateTime<Utc>) -> Vec<Schedule> {
        let mut sch: Vec<Schedule> = self
            .rates
            .iter()
            .map(|e| (e, e.schedule(from)))
            .map(|e| {
                e.1.iter()
                    .map(|f| Schedule {
                        rate: e.0.clone(),
                        window: f.clone(),
                    })
                    .collect::<Vec<Schedule>>()
            })
            .flatten()
            .collect();

        sch.sort_by(|a, b| a.window.start.cmp(&b.window.start));
        sch
    }

    pub fn desired_state(
        &self,
        from: DateTime<Utc>,
        current_state: ControllerInputState,
    ) -> Result<ControllerOutputState, ControllerError> {
        let sch = self.get_schedule(from);

        let current_sch = sch
            .first()
            .ok_or_else(|| ControllerError("No current rate Found".to_owned()))?;
        let next_charge = sch
            .iter()
            .filter(|s| s.rate.charge.charge_enabled())
            .next()
            .ok_or_else(|| ControllerError("No next charge rate Found".to_owned()))?;

        if current_sch.rate.charge.charge_enabled() {
            // current rate is charger, just charge
            return Ok(ControllerOutputState {
                disable_charge: false,
                disable_feed_in: true,
                soc: current_state.soc,
                grid_load: 32_000.0,
                battery_load: 0.0,
                using_capacity: 0.0,
                reserve_capacity: 0.0,
                current_rate: current_sch.clone(),
                next_rate: sch
                    .get(1)
                    .ok_or_else(|| ControllerError("No next rate found".to_owned()))?
                    .clone(),
                next_charge: next_charge.clone(),
            });
        } else {
            // we are discharging, use remaining capacity
            let rates_before_charge: Vec<&Schedule> = sch
                .iter()
                .filter(|s| s.window.start < next_charge.window.start && !s.window.is_inside(from))
                .collect();
            let reserve = rates_before_charge
                .iter()
                .fold(0f32, |acc, &s| acc + s.rate.reserve);
            let time_until_charge = next_charge.window.start - from;
            let soc = (current_state.soc - (1.0 - self.dod)).max(0.0);
            let kwh_capacity = current_state.capacity * soc;
            let remaining_capacity = (kwh_capacity - reserve).max(0.0);

            let battery_load = match current_sch.rate.discharge {
                RateDischarge::Spread => {
                    let hours = time_until_charge.num_minutes() as f32 / 60.0;
                    (remaining_capacity / hours) * 1000.0
                }
                RateDischarge::Capacity(v) => current_state.system_load * v,
                _ => 0.0,
            }.max(0.0);

            let disable_feed_in = remaining_capacity == 0.0 || battery_load == 0.0;
            return Ok(ControllerOutputState {
                disable_charge: true,
                disable_feed_in,
                soc,
                grid_load: (current_state.system_load - battery_load).max(0.0),
                battery_load,
                using_capacity: remaining_capacity,
                reserve_capacity: reserve,
                current_rate: current_sch.clone(),
                next_rate: sch
                    .get(1)
                    .ok_or_else(|| ControllerError("No next rate found".to_owned()))?
                    .clone(),
                next_charge: next_charge.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_ess::rate::{ChargeMode, Rate, RateCharge};
    use crate::smart_ess::window::{RateTime, RateWindow, ALL_WEEKDAYS};
    use chrono::{Local, TimeZone};
    use std::str::FromStr;

    fn get_controller() -> Controller {
        Controller {
            dod: 0.9,
            rates: vec![
                Rate {
                    name: "Day".to_owned(),
                    unit_cost: 0.0,
                    windows: vec![RateWindow {
                        start: RateTime::from_str("09:00").unwrap(),
                        end: RateTime::from_str("16:59").unwrap(),
                        days: ALL_WEEKDAYS.into(),
                    }],
                    discharge: RateDischarge::Spread,
                    charge: RateCharge {
                        mode: ChargeMode::Disabled,
                        unit_limit: 0,
                    },
                    reserve: 0.0,
                },
                Rate {
                    name: "Peak".to_owned(),
                    unit_cost: 0.0,
                    windows: vec![RateWindow {
                        start: RateTime::from_str("17:00").unwrap(),
                        end: RateTime::from_str("18:59").unwrap(),
                        days: ALL_WEEKDAYS.into(),
                    }],
                    discharge: RateDischarge::Capacity(1.0),
                    charge: RateCharge {
                        mode: ChargeMode::Disabled,
                        unit_limit: 0,
                    },
                    reserve: 0.0,
                },
                Rate {
                    name: "Night".to_owned(),
                    unit_cost: 0.0,
                    windows: vec![RateWindow {
                        start: RateTime::from_str("23:00").unwrap(),
                        end: RateTime::from_str("08:59").unwrap(),
                        days: ALL_WEEKDAYS.into(),
                    }],
                    discharge: RateDischarge::None,
                    charge: RateCharge {
                        mode: ChargeMode::Capacity(1.0),
                        unit_limit: 0,
                    },
                    reserve: 0.0,
                },
            ],
        }
    }

    #[test]
    fn schedule() {
        let controller = get_controller();
        let from = Local.ymd(2022, 05, 03).and_hms(2, 0, 0).with_timezone(&Utc);
        let sch = controller.get_schedule(from);
        let next = sch.get(0).unwrap();

        assert_eq!(next.window.start, Local.ymd(2022, 05, 02).and_hms(23, 0, 0));
    }

    #[test]
    fn min_soc() {
        let controller = get_controller();
        let from = Local
            .ymd(2022, 05, 03)
            .and_hms(17, 30, 0)
            .with_timezone(&Utc);

        let state_at_dod = controller
            .desired_state(
                from,
                ControllerInputState {
                    system_load: 1000.0,
                    soc: 1.0 - controller.dod,
                    capacity: 4.0,
                    voltage: 0.0,
                },
            )
            .unwrap();

        assert_eq!(
            state_at_dod.current_rate.rate.discharge,
            RateDischarge::Capacity(1.0),
            "Is peak discharge"
        );
        assert_eq!(
            state_at_dod.disable_feed_in, true,
            "Disable feed-in when at min state of charge {:?}", state_at_dod
        );

        let state_above_dod = controller
            .desired_state(
                from,
                ControllerInputState {
                    system_load: 1000.0,
                    soc: 1.1 - controller.dod,
                    capacity: 4.0,
                    voltage: 0.0,
                },
            )
            .unwrap();

        assert_eq!(
            state_above_dod.disable_feed_in, false,
            "Above DoD {:?}", state_above_dod
        );
    }
}

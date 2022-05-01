use crate::smart_ess::rate::{Rate, RateDischarge};
use crate::victron::{ess, ve_bus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::ops::Index;

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
    rates: Vec<Rate>,
}

#[derive(Debug, Clone)]
pub struct Schedule {
    pub rate: Rate,
    pub start: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ControllerState {
    pub disable_charge: bool,
    pub disable_feed_in: bool,

    /// Grid load in watts
    pub grid_load: f32,

    /// Battery load in watts
    pub battery_load: f32,

    /// Battery state of charge percent
    pub soc: f32,

    /// Battery capacity in kWh
    pub capacity: f32,
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
                        start: f.start.clone(),
                    })
                    .collect::<Vec<Schedule>>()
            })
            .flatten()
            .collect();

        sch.sort_by(|a, b| a.start.cmp(&b.start));
        sch
    }

    pub fn desired_state(
        &self,
        from: DateTime<Utc>,
        current_state: ControllerState,
    ) -> Result<ControllerState, ControllerError> {
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
            return Ok(ControllerState {
                disable_charge: false,
                disable_feed_in: true,
                grid_load: 32_000.0,
                battery_load: 0.0,
                soc: current_state.soc,
                capacity: current_state.capacity,
            });
        } else {
            // we are discharging, use remaining capacity
            let rates_before_charge: Vec<&Schedule> = sch
                .iter()
                .filter(|s| s.start < next_charge.start)
                .collect();
            let reserve = rates_before_charge
                .iter()
                .fold(0f32, |acc, &s| acc + s.rate.reserve);
            let time_until_charge = dbg!(next_charge.start - from);
            let kwh_capacity = dbg!(current_state.capacity * current_state.soc);
            let remaining_capacity = dbg!((kwh_capacity - reserve).max(0.0));

            let discharge = current_sch.rate.discharge;
            let battery_load = match discharge {
                RateDischarge::Spread => {
                    let hours = dbg!(time_until_charge.num_minutes() as f32 / 60.0);
                    (remaining_capacity / hours) * 1000.0
                }
                _ => 0.0
            };

            return Ok(ControllerState {
                disable_charge: true,
                disable_feed_in: false,
                grid_load: (current_state.grid_load - battery_load).max(0.0),
                battery_load,
                soc: current_state.soc,
                capacity: current_state.capacity,
            });
        }
    }
}

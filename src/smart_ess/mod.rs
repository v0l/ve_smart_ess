use std::borrow::Borrow;
use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Read};
use chrono::{DateTime, Utc};
use crate::smart_ess::rate::{Rate, RateCharge};
use serde::{Serialize, Deserialize};
use crate::VictronError;

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

impl Controller {
    pub fn load() -> Result<Controller, ControllerError> {
        let path = "smart_ess.json";
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => File::create(path)?
        };
        let mut json = String::new();
        file.read_to_string(&mut json);
        let v: Controller = serde_json::from_str(&json)?;
        Ok(v)
    }

    pub fn next_charge_from(&self, from: DateTime<Utc>) -> Result<Schedule, ControllerError> {
        if let Some(v) = self.get_schedule_from(from)
            .iter().filter(|s| s.rate.charge.charge_enabled())
            .take(1).next() {
            Ok(v.clone())
        } else {
            Err(ControllerError("No rate found!".to_owned()))
        }
    }

    pub fn get_schedule_from(&self, from: DateTime<Utc>) -> Vec<Schedule> {
        let mut sch: Vec<Schedule> = self.rates.iter()
            .map(|e| (e, e.next_from(from)))
            .filter(|f| f.1.is_some())
            .map(|e| Schedule { rate: e.0.clone(), start: e.1.unwrap() })
            .collect();
        sch.sort_by(|a,b| a.start.cmp(&b.start));
        sch
    }
}
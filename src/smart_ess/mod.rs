use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Read};
use crate::smart_ess::rate::Rate;
use serde::{Serialize, Deserialize};
use crate::VictronError;

pub mod rate;
pub mod window;

#[derive(Debug)]
pub struct ControllerError(pub String);

impl<TStr : ToString> From<TStr> for ControllerError {
    fn from(t: TStr) -> Self {
        ControllerError(t.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Controller {
    rates: Vec<Rate>,
}

impl Controller {
    pub fn load() -> Result<Controller, ControllerError> {
        let path = "smart_ess.json";
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => File::create(path)?
        };
        let mut json  = String::new();
        file.read_to_string(&mut json);
        let v : Controller = serde_json::from_str(&json)?;
        Ok(v)
    }

    pub fn next_charge(&self) -> Result<Rate, ControllerError> {
        let mut rates_by_window : Vec<Rate> = vec![];
        for rate in &self.rates {
            for window in &rate.windows {
                let mut rate_window = rate.clone();
                rate_window.windows.clear();
                rate_window.windows.push(window.clone());
                rates_by_window.push(rate_window);
            }
        }

        rates_by_window.sort_by(|a, b| a.windows.first().unwrap().start.cmp(&b.windows.first().unwrap().start));

        for rate in dbg!(rates_by_window) {

        }
        Err(ControllerError("No rate found!".to_owned()))
    }
}
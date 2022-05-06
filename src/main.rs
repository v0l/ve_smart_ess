extern crate core;

use crate::smart_ess::{Controller, ControllerInputState};
use crate::victron::ess::VictronESS;
use crate::victron::ve_bus::VictronBus;
use crate::victron::{ess, Line, Side, VictronError};
use chrono::{Local, Utc};
use std::thread::sleep;
use std::time::Duration;

mod smart_ess;
mod victron;

const INVERTER: u8 = 227;
//const BATTERY: u8 = 225;
//const SYSTEM: u8 = 100;

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), VictronError> {
    let mut vs = VictronBus::new("10.100.1.58:502".parse().unwrap(), INVERTER).await?;
    let mut ess = VictronESS::new("10.100.1.58:502".parse().unwrap(), INVERTER).await?;

    let ctr = Controller::load().map_err(|e| VictronError(e.0))?;

    loop {
        let soc = vs.soc().await?;
        let in1 = vs.get_line_info(Side::Input, Line::L1).await?;
        let out1 = vs.get_line_info(Side::Output, Line::L1).await?;

        // ess
        let set_point = ess
            .get_param(ess::Register::PowerSetPoint(Line::L1, 0))
            .await?;
        let disable_charger = ess.get_param(ess::Register::DisableCharge(false)).await?;
        let disable_feed_in = ess.get_param(ess::Register::DisableFeedIn(false)).await?;

        println!("====================");
        println!("Time: {}", Local::now());

        let desired_state = ctr
            .desired_state(
                Utc::now(),
                ControllerInputState {
                    system_load: out1.power,
                    soc: soc / 100.0,
                    capacity: 4.0,
                    voltage: 0.0,
                },
            )
            .unwrap();
        println!("{}", desired_state);

        let target_set_point = (desired_state.grid_load as i16).max(50);
        ess.set_param(ess::Register::PowerSetPoint(Line::L1, target_set_point))
            .await?;

        if disable_feed_in != ess::Register::DisableFeedIn(desired_state.disable_feed_in) {
            ess.set_param(ess::Register::DisableFeedIn(desired_state.disable_feed_in))
                .await?;
        }
        if disable_charger != ess::Register::DisableCharge(desired_state.disable_charge) {
            ess.set_param(ess::Register::DisableCharge(desired_state.disable_charge))
                .await?;
        }

        sleep(Duration::from_secs(10))
    }
}

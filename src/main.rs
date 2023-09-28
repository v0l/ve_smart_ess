extern crate core;

use crate::smart_ess::{Controller, ControllerInputState};
use crate::victron::ess::VictronESS;
use crate::victron::ve_bus::VictronBus;
use crate::victron::{ess, Line, Side, VictronError};
use chrono::{Local, Utc};
use std::net::SocketAddr;
use std::thread::sleep;
use std::time::Duration;
use crate::victron::ve_battery::VictronBattery;

mod smart_ess;
mod victron;

const INVERTER: u8 = 227;
//const BATTERY: u8 = 225;
//const SYSTEM: u8 = 100;

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), VictronError> {
    let addr: SocketAddr = "10.100.2.248:502".parse().unwrap();
    let mut vs = VictronBus::new(addr, INVERTER).await?;
    let mut ess = VictronESS::new(addr, INVERTER).await?;

    let ctr = Controller::load().map_err(|e| VictronError(e.0))?;

    loop {
        let soc = vs.soc().await?;
        let out1 = vs.get_line_info(Side::Output, Line::L1).await?;

        println!("====================");
        println!("Time: {}", Local::now());

        let desired_state = ctr
            .desired_state(
                Utc::now(),
                ControllerInputState {
                    system_load: out1.power,
                    soc: soc / 100.0,
                    capacity: 7.2,
                    voltage: 0.0,
                },
            )
            .unwrap();
        println!("{}", desired_state);

        let target_set_point = (desired_state.grid_load as i16).max(50);
        ess.set_param(ess::Register::PowerSetPoint(Line::L1, target_set_point))
            .await?;

        ess.set_param(ess::Register::DisableFeedIn(desired_state.disable_feed_in))
            .await?;

        ess.set_param(ess::Register::DisableCharge(desired_state.disable_charge))
            .await?;

        sleep(Duration::from_secs(10))
    }
}

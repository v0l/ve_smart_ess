extern crate core;

use std::thread::sleep;
use std::time::Duration;
use crate::smart_ess::Controller;
use crate::victron::ess::{ESS, VictronESS};
use crate::victron::ve_bus::{Register, VictronBus};
use crate::victron::{Line, Side, VictronError};

mod victron;
mod smart_ess;

const INVERTER: u8 = 227;
const BATTERY: u8 = 225;
const SYSTEM: u8 = 100;

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), VictronError> {
    let mut vs = VictronBus::new("10.100.1.58:502".parse().unwrap(), INVERTER).await?;

    let ctr = Controller::load().map_err(|e| VictronError(e.0))?;

    println!("{:?}", ctr);

    loop {
        let phases = vs.get(VEBus::PhaseCount).await?;
        let active_input = vs.get_active_input().await?;
        let mode = vs.get_mode().await?;

        let in1 = vs.get_line_info(Side::Input, Line::L1).await?;
        let out1 = vs.get_line_info(Side::Output, Line::L1).await?;

        println!("=== SYSTEM ===\nPhases: \t{}\nActive Phase: \t{}\nMode: \t\t{}", phases, active_input.to_string(), mode.to_string());

        println!("IN_L1 = {:?}", in1);
        println!("OUT_L1 = {:?}", out1);

        sleep(Duration::from_secs(5))
    }
    Ok(())
}
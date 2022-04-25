extern crate core;

use std::thread::sleep;
use std::time::Duration;
use chrono::Utc;
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
    let mut ess = VictronESS::new("10.100.1.58:502".parse().unwrap(), INVERTER).await?;

    let ctr = Controller::load().map_err(|e| VictronError(e.0))?;

    println!("{:?}", ctr);

    let phases = vs.get(Register::PhaseCount).await?;
    let active_input = vs.get_active_input().await?;
    let mode = vs.get_mode().await?;
    let state = vs.get_state().await?;

    let in1 = vs.get_line_info(Side::Input, Line::L1).await?;
    let out1 = vs.get_line_info(Side::Output, Line::L1).await?;

    // ess
    let set_point = ess.get_param(ESS::PowerSetPoint(Line::L1, 0)).await?;
    let disable_charger = ess.get_param(ESS::DisableCharge(false)).await?;
    let disable_feed_in = ess.get_param(ESS::DisableFeedIn(false)).await?;

    let target_set_point = (out1.power / 2f32) as i16;
    if set_point != ESS::PowerSetPoint(Line::L1, target_set_point) {
        //ess.set_param(ESS::PowerSetPoint(Line::L1, target_set_point)).await?;
    }
    if disable_feed_in != ESS::DisableFeedIn(false) {
        ess.set_param(ESS::DisableFeedIn(false)).await?;
    }
    if disable_charger != ESS::DisableCharge(false) {
        ess.set_param(ESS::DisableCharge(false)).await?;
    }

    println!("=== SCHEDULE ===");
    let sch = ctr.get_schedule(Utc::now());
    for s in sch.iter() {
        println!("{} @ {}", s.rate.name, s.start);
    }

    let next_charge = ctr.next_charge(Utc::now());

    println!("=== SYSTEM ===\nPhases: \t{}\nActive Phase: \t{}\nMode: \t\t{}\nState: \t\t{}", phases, active_input.to_string(), mode.to_string(), state.to_string());

    println!("IN_L1 = {:?}", in1);
    println!("OUT_L1 = {:?}", out1);
    println!("ESS = {:?} {:?} {:?}", set_point, disable_charger, disable_feed_in);
    println!("Next Charge: {:?}", next_charge);

    Ok(())
}
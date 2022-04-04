use crate::victron::ess::{ESSRegister, VictronESS};
use crate::victron::VictronError;

mod victron;

const INVERTER: u8 = 227;
const BATTERY: u8 = 225;
const SYSTEM: u8 = 100;

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), VictronError> {
    let mut cli = VictronESS::new("10.100.1.58:502".parse().unwrap(), INVERTER).await?;

    cli.set_param(ESSRegister::DisableFeedIn(true)).await?;

    Ok(())
}
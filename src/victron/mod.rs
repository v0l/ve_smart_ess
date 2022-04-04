pub mod client;
pub mod ess;
pub mod system;

#[derive(Debug)]
pub struct VictronError(String);

#[derive(Debug, Copy, Clone)]
pub enum Line {
    L1 = 1,
    L2 = 2,
    L3 = 3,
}

#[derive(Debug, Copy, Clone)]
pub struct LineDetail {
    voltage: f32,
    current: f32,
    frequency: f32,
    power: f32,
}
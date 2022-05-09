pub mod client;
pub mod ess;
pub mod ve_bus;
pub mod ve_battery;

#[derive(Debug)]
pub struct VictronError(pub String);

#[derive(Debug, Copy, Clone)]
pub enum Side {
    Input,
    Output,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Line {
    L1 = 1,
    L2 = 2,
    L3 = 3,
}

#[derive(Debug, Copy, Clone)]
pub struct LineDetail {
    pub voltage: f32,
    pub current: f32,
    pub frequency: f32,
    pub power: f32,
}

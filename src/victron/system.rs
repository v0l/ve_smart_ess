use crate::victron::client::VictronClient;
use crate::victron::{Line, LineDetail, Side, VictronError};
use std::net::SocketAddr;
use std::ops::Div;

pub struct VictronSystem {
    client: VictronClient,
}

pub enum VEBusMode {
    ChargerOnly = 1,
    InverterOnly = 2,
    On = 3,
    Off = 4,
}

impl ToString for VEBusMode {
    fn to_string(&self) -> String {
        match self {
            VEBusMode::ChargerOnly => "Charger Only",
            VEBusMode::InverterOnly => "Inverter Only",
            VEBusMode::On => "On",
            VEBusMode::Off => "Off",
        }.to_owned()
    }
}

pub enum VEBus {
    InputVoltage(Line),
    InputCurrent(Line),
    InputFrequency(Line),
    InputPower(Line),

    OutputVoltage(Line),
    OutputCurrent(Line),
    OutputFrequency,
    OutputPower(Line),

    ActiveInputCurrentLimit,

    BatteryVoltage,
    BatteryCurrent,

    PhaseCount,
    ActiveInput,

    Mode,
    Alarm(VEBusAlarm),
}

pub enum ActiveInput {
    Line1,
    Line2,
    Disconnected,
}

impl ToString for ActiveInput {
    fn to_string(&self) -> String {
        match self {
            ActiveInput::Line1 => "Line 1",
            ActiveInput::Line2 => "Line 2",
            ActiveInput::Disconnected => "Disconnected"
        }.to_owned()
    }
}

impl TryInto<Line> for ActiveInput {
    type Error = VictronError;

    fn try_into(self) -> Result<Line, Self::Error> {
        Ok(match self {
            ActiveInput::Line1 => Line::L1,
            ActiveInput::Line2 => Line::L2,
            _ => return Err(VictronError("No active input!".to_owned()))
        })
    }
}

#[derive(Copy, Clone)]
pub enum VEBusAlarmState {
    Ok = 0,
    Warning = 1,
    Alarm = 2,
}

#[derive(Copy, Clone)]
pub enum VEBusAlarm {
    HighTemperature(VEBusAlarmState),
    LowBattery(VEBusAlarmState),
    Overload(VEBusAlarmState),
    TemperatureSensor(VEBusAlarmState),
    VoltageSensor(VEBusAlarmState),

    LineTemperature(Line, VEBusAlarmState),
    LineLowBattery(Line, VEBusAlarmState),
    LineOverload(Line, VEBusAlarmState),
    LineRipple(Line, VEBusAlarmState),

    PhaseRotation(VEBusAlarmState),
    GridLost(VEBusAlarmState),
}

impl VictronSystem {
    pub async fn new(addr: SocketAddr, unit: u8) -> Result<Self, VictronError> {
        let mut cli = VictronClient::new(addr).await?;
        cli.set_unit(unit);
        Ok(Self { client: cli })
    }

    pub async fn get_line_info(&mut self, side: Side, line: Line) -> Result<LineDetail, VictronError> {
        match side {
            Side::Input => self.input_info(line).await,
            Side::Output => self.output_info(line).await
        }
    }

    async fn input_info(&mut self, line: Line) -> Result<LineDetail, VictronError> {
        let v = self.get(VEBus::InputVoltage(line)).await?;
        let i = self.get(VEBus::InputCurrent(line)).await?;
        let f = self.get(VEBus::InputFrequency(line)).await?;
        let p = self.get(VEBus::InputPower(line)).await?;

        Ok(LineDetail {
            voltage: v as f32 / 10f32,
            current: i as f32 / 10f32,
            frequency: f as f32 / 100f32,
            power: p as f32 / 0.1f32,
        })
    }

    async fn output_info(&mut self, line: Line) -> Result<LineDetail, VictronError> {
        let v = self.get(VEBus::OutputVoltage(line)).await?;
        let i = self.get(VEBus::OutputCurrent(line)).await?;
        let f = self.get(VEBus::OutputFrequency).await?;
        let p = self.get(VEBus::OutputPower(line)).await?;

        Ok(LineDetail {
            voltage: v as f32 / 10f32,
            current: i as f32 / 10f32,
            frequency: f as f32 / 100f32,
            power: p as f32 / 0.1f32,
        })
    }

    pub async fn get_mode(&mut self) -> Result<VEBusMode, VictronError> {
        let m = self.get(VEBus::Mode).await?;
        Ok(match m {
            1 => VEBusMode::ChargerOnly,
            2 => VEBusMode::InverterOnly,
            3 => VEBusMode::On,
            4 => VEBusMode::Off,
            e => return Err(VictronError(format!("Invalid mode {}!", e)))
        })
    }

    pub async fn set_mode(&mut self, mode: VEBusMode) -> Result<(), VictronError> {
        self.client.write_u16(self.get_register(VEBus::Mode), mode as u16).await
    }

    pub async fn get_active_input(&mut self) -> Result<ActiveInput, VictronError> {
        let a = self.get(VEBus::ActiveInput).await?;
        Ok(match a {
            0 => ActiveInput::Line1,
            1 => ActiveInput::Line2,
            240 => ActiveInput::Disconnected,
            e => return Err(VictronError(format!("Unknown active input {}!", e)))
        })
    }

    pub async fn get_alarms(&mut self) -> Result<Vec<VEBusAlarm>, VictronError> {
        use VEBusAlarm::*;
        use VEBusAlarmState::*;
        use Line::*;
        let mut all_alarms = vec![
            HighTemperature(Ok),
            LowBattery(Ok),
            Overload(Ok),
            TemperatureSensor(Ok),
            VoltageSensor(Ok),
            LineTemperature(L1, Ok),
            LineLowBattery(L1, Ok),
            LineOverload(L1, Ok),
            LineRipple(L1, Ok),
            LineTemperature(L2, Ok),
            LineLowBattery(L2, Ok),
            LineOverload(L2, Ok),
            LineRipple(L2, Ok),
            LineTemperature(L3, Ok),
            LineLowBattery(L3, Ok),
            LineOverload(L3, Ok),
            LineRipple(L3, Ok),
            PhaseRotation(Ok),
            GridLost(Ok),
        ];


        for alarm in all_alarms.iter_mut() {
            let sv = self.client.read_u16(self.get_register(VEBus::Alarm(alarm.clone()))).await?;
            let state = match sv {
                0 => VEBusAlarmState::Ok,
                1 => VEBusAlarmState::Warning,
                2 => VEBusAlarmState::Alarm,
                e => return Err(VictronError(format!("Invalid alarm state {}!", e)))
            };

            match alarm {
                HighTemperature(mut v) => v = state,
                LowBattery(mut v) => v = state,
                Overload(mut v) => v = state,
                TemperatureSensor(mut v) => v = state,
                VoltageSensor(mut v) => v = state,
                LineTemperature(_, mut v) => v = state,
                LineLowBattery(_, mut v) => v = state,
                LineOverload(_, mut v) => v = state,
                LineRipple(_, mut v) => v = state,
                PhaseRotation(mut v) => v = state,
                GridLost(mut v) => v = state,
            }
        }

        Result::Ok(all_alarms)
    }

    pub async fn get(&mut self, reg: VEBus) -> Result<u16, VictronError> {
        self.client.read_u16(self.get_register(reg)).await
    }

    fn get_register(&self, reg: VEBus) -> u16 {
        use VEBus::*;
        match reg {
            InputVoltage(l) => 2 + l as u16,
            InputCurrent(l) => 5 + l as u16,
            InputFrequency(l) => 8 + l as u16,
            InputPower(l) => 11 + l as u16,

            OutputVoltage(l) => 14 + l as u16,
            OutputCurrent(l) => 16 + l as u16,
            OutputFrequency => 21,
            OutputPower(l) => 22 + l as u16,

            ActiveInputCurrentLimit => 22,

            BatteryVoltage => 26,
            BatteryCurrent => 27,

            PhaseCount => 28,
            ActiveInput => 29,

            Mode => 33,
            Alarm(a) => match a {
                VEBusAlarm::HighTemperature(_) => 34,
                VEBusAlarm::LowBattery(_) => 35,
                VEBusAlarm::Overload(_) => 36,
                VEBusAlarm::TemperatureSensor(_) => 42,
                VEBusAlarm::VoltageSensor(_) => 43,
                VEBusAlarm::LineTemperature(l, _) => 44 + (4 * (l as u16 - 1)),
                VEBusAlarm::LineLowBattery(l, _) => 45 + (4 * (l as u16 - 1)),
                VEBusAlarm::LineOverload(l, _) => 46 + (4 * (l as u16 - 1)),
                VEBusAlarm::LineRipple(l, _) => 47 + (4 * (l as u16 - 1)),
                VEBusAlarm::PhaseRotation(_) => 63,
                VEBusAlarm::GridLost(_) => 64,
            }
        }
    }
}

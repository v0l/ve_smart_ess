use crate::victron::client::VictronClient;
use crate::victron::{Line, LineDetail, Side, VictronError};
use std::net::SocketAddr;

pub struct VictronBus {
    client: VictronClient,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    ChargerOnly = 1,
    InverterOnly = 2,
    On = 3,
    Off = 4,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::ChargerOnly => "Charger Only",
            Mode::InverterOnly => "Inverter Only",
            Mode::On => "On",
            Mode::Off => "Off",
        }.to_owned()
    }
}

impl TryFrom<u8> for Mode {
    type Error = VictronError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Mode::ChargerOnly,
            2 => Mode::InverterOnly,
            3 => Mode::On,
            4 => Mode::Off,
            e => return Err(VictronError(format!("Invalid mode {}!", e)))
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Off = 0,
    LowPower = 1,
    Fault = 2,
    Bulk = 3,
    Absorption = 4,
    Float = 5,
    Storage = 6,
    Equalize = 7,
    Passthrough = 8,
    Inverting = 9,
    PowerAssist = 10,
    PowerSupply = 11,
    BulkProtection = 252,
}

impl ToString for State {
    fn to_string(&self) -> String {
        match self {
            State::Off => "Off",
            State::LowPower => "Low Power",
            State::Fault => "Fault",
            State::Bulk => "Bulk",
            State::Absorption => "Absorption",
            State::Float => "Float",
            State::Storage => "Storage",
            State::Equalize => "Equalize",
            State::Passthrough => "Passthrough",
            State::Inverting => "Inverting",
            State::PowerAssist => "Power Assist",
            State::PowerSupply => "Power Supply",
            State::BulkProtection => "Bulk Protection"
        }.to_owned()
    }
}

impl TryFrom<u8> for State {
    type Error = VictronError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => State::Off,
            1 => State::LowPower,
            2 => State::Fault,
            3 => State::Bulk,
            4 => State::Absorption,
            5 => State::Float,
            6 => State::Storage,
            7 => State::Equalize,
            8 => State::Passthrough,
            9 => State::Inverting,
            10 => State::PowerAssist,
            11 => State::PowerSupply,
            252 => State::BulkProtection,
            e => return Err(VictronError(format!("Invalid mode {}!", e)))
        })
    }
}

pub enum Register {
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

    State,
    Mode,
    Alarm(Alarm),

    ACInputIgnore(Line, bool),
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
pub enum AlarmState {
    Ok = 0,
    Warning = 1,
    Alarm = 2,
}

impl TryFrom<u8> for AlarmState {
    type Error = VictronError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => AlarmState::Ok,
            1 => AlarmState::Warning,
            2 => AlarmState::Alarm,
            e => return Err(VictronError(format!("Invalid alarm state {}!", e)))
        })
    }
}

#[derive(Copy, Clone)]
pub enum Alarm {
    HighTemperature(AlarmState),
    LowBattery(AlarmState),
    Overload(AlarmState),
    TemperatureSensor(AlarmState),
    VoltageSensor(AlarmState),

    LineTemperature(Line, AlarmState),
    LineLowBattery(Line, AlarmState),
    LineOverload(Line, AlarmState),
    LineRipple(Line, AlarmState),

    PhaseRotation(AlarmState),
    GridLost(AlarmState),
}

impl VictronBus {
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
        let v = self.get(Register::InputVoltage(line)).await?;
        let i = self.get(Register::InputCurrent(line)).await?;
        let f = self.get(Register::InputFrequency(line)).await?;
        let p = self.get(Register::InputPower(line)).await?;

        Ok(LineDetail {
            voltage: v as f32 / 10f32,
            current: i as f32 / 10f32,
            frequency: f as f32 / 100f32,
            power: p as f32 / 0.1f32,
        })
    }

    async fn output_info(&mut self, line: Line) -> Result<LineDetail, VictronError> {
        let v = self.get(Register::OutputVoltage(line)).await?;
        let i = self.get(Register::OutputCurrent(line)).await?;
        let f = self.get(Register::OutputFrequency).await?;
        let p = self.get(Register::OutputPower(line)).await?;

        Ok(LineDetail {
            voltage: v as f32 / 10f32,
            current: i as f32 / 10f32,
            frequency: f as f32 / 100f32,
            power: p as f32 / 0.1f32,
        })
    }

    pub async fn get_state(&mut self) -> Result<State, VictronError> {
        let s = self.get(Register::State).await?;
        State::try_from(s as u8)
    }
    pub async fn get_mode(&mut self) -> Result<Mode, VictronError> {
        let m = self.get(Register::Mode).await?;
        Mode::try_from(m as u8)
    }

    pub async fn set_mode(&mut self, mode: Mode) -> Result<(), VictronError> {
        self.client.write_u16(self.get_register(Register::Mode)?, mode as u16).await
    }

    pub async fn get_active_input(&mut self) -> Result<ActiveInput, VictronError> {
        let a = self.get(Register::ActiveInput).await?;
        Ok(match a {
            0 => ActiveInput::Line1,
            1 => ActiveInput::Line2,
            240 => ActiveInput::Disconnected,
            e => return Err(VictronError(format!("Unknown active input {}!", e)))
        })
    }

    pub async fn get_alarms(&mut self) -> Result<Vec<Alarm>, VictronError> {
        use crate::victron::ve_bus::Alarm::*;
        use AlarmState::*;
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
            let sv = self.client.read_u16(self.get_register(Register::Alarm(alarm.clone()))?).await?;
            let state = AlarmState::try_from(sv as u8)?;

            match alarm {
                HighTemperature(mut _v) => _v = state,
                LowBattery(mut _v) => _v = state,
                Overload(mut _v) => _v = state,
                TemperatureSensor(mut _v) => _v = state,
                VoltageSensor(mut _v) => _v = state,
                LineTemperature(_, mut _v) => _v = state,
                LineLowBattery(_, mut _v) => _v = state,
                LineOverload(_, mut _v) => _v = state,
                LineRipple(_, mut _v) => _v = state,
                PhaseRotation(mut _v) => _v = state,
                GridLost(mut _v) => _v = state,
            }
        }

        Result::Ok(all_alarms)
    }

    pub async fn get(&mut self, reg: Register) -> Result<u16, VictronError> {
        self.client.read_u16(self.get_register(reg)?).await
    }

    fn get_register(&self, reg: Register) -> Result<u16, VictronError> {
        use crate::victron::ve_bus::Alarm as BusAlarm;
        use Register::*;
        Ok(match reg {
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

            State => 31,
            Mode => 33,
            Alarm(a) => match a {
                BusAlarm::HighTemperature(_) => 34,
                BusAlarm::LowBattery(_) => 35,
                BusAlarm::Overload(_) => 36,
                BusAlarm::TemperatureSensor(_) => 42,
                BusAlarm::VoltageSensor(_) => 43,
                BusAlarm::LineTemperature(l, _) => 44 + (4 * (l as u16 - 1)),
                BusAlarm::LineLowBattery(l, _) => 45 + (4 * (l as u16 - 1)),
                BusAlarm::LineOverload(l, _) => 46 + (4 * (l as u16 - 1)),
                BusAlarm::LineRipple(l, _) => 47 + (4 * (l as u16 - 1)),
                BusAlarm::PhaseRotation(_) => 63,
                BusAlarm::GridLost(_) => 64,
            },
            ACInputIgnore(l, _) => match l {
                Line::L1 => 69,
                Line::L2 => 70,
                Line::L3 => return Err(VictronError("No AC Input Ignore for Line 3!".to_owned()))
            }
        })
    }
}

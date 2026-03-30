use crate::{DateTime, PersistenceId, PersistenceItem};
use heapless::spsc::{Consumer, Producer, Queue};

#[derive(Debug, Copy, Clone)]
pub enum IdleEvent {
    SetEepromItem(PersistenceItem),
    ClearEepromItems(&'static [PersistenceId]),
    RestoreEepromItems,
    RestoreToSandardProfile,
    SdCardItem(SdCardCmd),
    DebugLog(DebugLogRecord),
    EventLog(EventLogRecord),
    FeedTheDog,
    SetGain(u8),
    DateTime(DateTime),
    ResetDevice(ResetReason),
    Output1(PinState),
    Output2(PinState),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PinState {
    Low,
    High,
}

impl core::ops::Not for PinState {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            PinState::High => PinState::Low,
            PinState::Low => PinState::High,
        }
    }
}

impl core::convert::From<bool> for PinState {
    fn from(is_high: bool) -> Self {
        if is_high {
            PinState::High
        } else {
            PinState::Low
        }
    }
}

impl PinState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PinState::Low => "Low",
            PinState::High => "High",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ResetReason {
    NoReason,
    ConfigChanged,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SdCardCmd {
    SwUpdateAccepted,
    SwUpdateCanceled,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum DebugLogCode {
    Stats = 1,
    NmeaQueueDrop = 2,
    NmeaParseError = 3,
    SchedulerOverflow = 4,
    NmeaTxOverflow = 5,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DebugLogRecord {
    pub code: DebugLogCode,
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum EventLogCode {
    ThermalReset = 1,
    ThermalCircleComplete = 2,
    ThermalEstimateValid = 3,
    ThermalEstimateInvalid = 4,
    FlyModeChanged = 10,
    GpsStateChanged = 20,
    UpdateFound = 30,
    UpdateInstallStart = 31,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EventLogRecord {
    pub code: EventLogCode,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
}

// This queue transports the configuration PersItems from controller to the idle loop.
const MAX_IDLE_EVENTS: usize = 20;
pub type QIdleEvents = Queue<IdleEvent, MAX_IDLE_EVENTS>;
pub type PIdleEvents = Producer<'static, IdleEvent, MAX_IDLE_EVENTS>;
pub type CIdleEvents = Consumer<'static, IdleEvent, MAX_IDLE_EVENTS>;

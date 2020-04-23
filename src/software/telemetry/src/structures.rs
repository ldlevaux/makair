#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Production,
    Qualification,
    IntegrationTest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Inhalation,
    Exhalation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubPhase {
    Inspiration,
    HoldInspiration,
    Exhale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlarmPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSnapshot {
    pub version: String,
    pub device_id: String,
    pub systick: u64,
    pub centile: u16,
    pub pressure: u16,
    pub phase: Phase,
    pub subphase: SubPhase,
    pub blower_valve_position: u8,
    pub patient_valve_position: u8,
    pub blower_rpm: u8,
    pub battery_level: u8,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MachineSnapshot {
    pub version: String,
    pub device_id: String,
    pub cycle: u32,
    pub peak_command: u8,
    pub plateau_command: u8,
    pub peep_command: u8,
    pub cpm_command: u8,
    pub previous_peak_pressure: u8,
    pub previous_plateau_pressure: u8,
    pub previous_peep_pressure: u8,
    pub current_alarm_codes: Vec<u8>,
    pub previous_alarm_codes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelemetryMessage {
    BootMessage {
        version: String,
        device_id: String,
        systick: u64,
        mode: Mode,
        min8: u8,
        max8: u8,
        min32: u32,
        max32: u32,
    },
    DataSnapshot(DataSnapshot),
    MachineStateSnapshot(MachineSnapshot),
    AlarmTrap {
        version: String,
        device_id: String,
        systick: u64,
        centile: u16,
        pressure: u16,
        phase: Phase,
        subphase: SubPhase,
        cycle: u32,
        alarm_code: u8,
        alarm_priority: AlarmPriority,
        triggered: bool,
        expected: u32,
        measured: u32,
        cycles_since_trigger: u32,
    },
}

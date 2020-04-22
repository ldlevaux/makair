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
    MachineStateSnapshot {
        version: String,
        device_id: String,
        cycle: u32,
        peak_command: u8,
        plateau_command: u8,
        peep_command: u8,
        cpm_command: u8,
        previous_peak_pressure: u8,
        previous_plateau_pressure: u8,
        previous_peep_pressure: u8,
        current_alarm_codes: Vec<u8>,
        previous_alarm_codes: Vec<u8>,
    },
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

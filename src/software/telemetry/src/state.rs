use super::structures::{TelemetryMessage};

pub struct CurrentPrevious<C> {
  current: C,
  previous: C
}

impl<C: Copy> CurrentPrevious<C> {
  pub fn new(init: C) -> CurrentPrevious<C> {
    CurrentPrevious {
      current: init,
      previous: init
    }
  }

  pub fn push(&mut self, current: C, previous: C) {
    self.current = current;
    self.previous = previous;
  }
}

pub struct TelemetryState {
  pub active_errors: Vec<u8>,
  pub pressure_history: Vec<u16>,
  pub peak: CurrentPrevious<u8>,
  pub plateau: CurrentPrevious<u8>,
  pub peep: CurrentPrevious<u8>,
  pub cycles_per_minute: u8,
}

impl TelemetryState {
  pub fn new() -> TelemetryState {
    TelemetryState {
      active_errors: Vec::new(),
      pressure_history: Vec::new(),
      peak: CurrentPrevious::new(0),
      plateau: CurrentPrevious::new(0),
      peep: CurrentPrevious::new(0),
      cycles_per_minute: 0,
    }
  }

  pub fn push(&mut self, message: TelemetryMessage) {
    match message {
      TelemetryMessage::AlarmTrap { alarm_code, triggered, .. } => {
        debug!("We got an error: {}: {}", alarm_code, triggered);
        match triggered {
          true => {
            if self.active_errors.iter().find(|error| **error == alarm_code).is_none() {
              self.active_errors.push(alarm_code);
            }
          },
          false => {
            self.active_errors.retain(|error| *error != alarm_code);
          }
        }
      },
      TelemetryMessage::BootMessage { .. } => {
        unimplemented!("Boot message");
      },
      TelemetryMessage::DataSnapshot(snapshot) => {
        self.pressure_history.push(snapshot.pressure);
      },
      TelemetryMessage::MachineStateSnapshot {
        peak_command, previous_peak_pressure,
        plateau_command, previous_plateau_pressure,
        peep_command, previous_peep_pressure,
        .. } => {
        self.peak.push(peak_command, previous_peak_pressure);
        self.plateau.push(plateau_command, previous_plateau_pressure);
        self.peep.push(peep_command, previous_peep_pressure);
      }
    }
  }

  pub fn display(&self) {
    self.clear();
    info!("Active errors: {:?}", self.active_errors);

    let last_pressure = self.pressure_history.last().map(|d| d.to_string()).unwrap_or("None yet".to_string());
    info!("Last pressure: {}", last_pressure);

    info!("P(peak): {} <- ({})", self.peak.current, self.peak.previous);
    info!("P(plateau): {} <- ({})", self.plateau.current, self.plateau.previous);
    info!("P(expiratory): {} <- ({})", self.peep.current, self.peep.previous);
    info!("Cycles per minute: {}", self.cycles_per_minute);
  }

  fn clear(&self) {
    print!("{}[2J", 27 as char);
  }
}
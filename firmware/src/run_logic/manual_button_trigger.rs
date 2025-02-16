use super::Trigger;
use crate::{
    buttons::{Button, ButtonEvent, ButtonPushDuration},
    fan::{FanCommand, FanSpeed},
};
use defmt::Format;
use embassy_time::Duration;

const FAN_BUTTON_TIMEOUT: Duration = Duration::from_secs(60 * 20);

#[derive(Clone, Format)]
pub(super) struct ManualButtonTrigger {
    time_remaining: Option<Duration>,
    requested_speed: FanSpeed,
}

impl Default for ManualButtonTrigger {
    fn default() -> Self {
        Self {
            time_remaining: None,
            requested_speed: FanSpeed::Low,
        }
    }
}

impl Trigger for ManualButtonTrigger {
    fn fan_command(&self) -> FanCommand {
        match self.time_remaining {
            Some(_) => FanCommand::Run(self.requested_speed.clone()),
            None => FanCommand::Stop,
        }
    }
}

impl ManualButtonTrigger {
    pub(super) fn handle_tick(&mut self) -> bool {
        if let Some(time_remaining) = self.time_remaining {
            match time_remaining.checked_sub(Duration::from_secs(1)) {
                Some(t) => {
                    if t < Duration::from_secs(1) {
                        *self = Self::default();
                    } else {
                        self.time_remaining = Some(t);
                    }
                }
                None => {
                    *self = Self::default();
                }
            }
            true
        } else {
            false
        }
    }

    pub(super) fn handle_button(&mut self, event: ButtonEvent) -> bool {
        match event {
            // Start/renew time
            ButtonEvent {
                button: Button::Demand,
                push_duration: ButtonPushDuration::Short,
            } => {
                self.time_remaining = Some(FAN_BUTTON_TIMEOUT);
                true
            }
            // Stop
            ButtonEvent {
                button: Button::Demand,
                push_duration: ButtonPushDuration::Long,
            } => {
                if self.time_remaining.is_some() {
                    *self = Self::default();
                    true
                } else {
                    false
                }
            }
            // Cycle fan speed
            ButtonEvent {
                button: Button::Speed,
                push_duration: ButtonPushDuration::Short,
            } => {
                if self.time_remaining.is_some() {
                    self.requested_speed.cycle();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

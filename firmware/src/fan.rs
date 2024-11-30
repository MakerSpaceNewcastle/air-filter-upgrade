use defmt::{debug, Format};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;

pub(crate) static FAN_SPEED: Signal<CriticalSectionRawMutex, Option<FanSpeed>> = Signal::new();

#[derive(Clone, Format, Eq, PartialEq)]
pub(crate) enum FanSpeed {
    Low,
    Medium,
    High,
}

impl FanSpeed {
    pub(crate) fn cycle(&mut self) {
        match self {
            FanSpeed::Low => *self = FanSpeed::Medium,
            FanSpeed::Medium => *self = FanSpeed::High,
            FanSpeed::High => *self = FanSpeed::Low,
        }
    }
}

#[embassy_executor::task]
pub(super) async fn task(r: crate::FanRelayResources) {
    let mut fan_high = Output::new(r.high, Level::Low);
    let mut fan_medium = Output::new(r.medium, Level::Low);
    let mut fan_low = Output::new(r.low, Level::Low);
    let mut contactor_voltage = Output::new(r.contactor_voltage, Level::Low);

    let mut last = None;

    loop {
        let speed_cmd = FAN_SPEED.wait().await;

        if speed_cmd != last {
            debug!("Open all speed selection contactors");
            fan_low.set_low();
            fan_medium.set_low();
            fan_high.set_low();

            Timer::after_millis(10).await;

            if let Some(speed) = speed_cmd.clone() {
                debug!("Set contactor voltage to 24V");
                contactor_voltage.set_high();

                Timer::after_millis(10).await;

                debug!("Close speed selection contactor for {}", speed);
                match speed {
                    FanSpeed::Low => &mut fan_low,
                    FanSpeed::Medium => &mut fan_medium,
                    FanSpeed::High => &mut fan_high,
                }
                .set_high();

                Timer::after_millis(500).await;

                debug!("Set contactor voltage to 5V");
                contactor_voltage.set_low();
            }

            // Enforce the new speed for a very minimal sensible amount of time
            Timer::after_secs(1).await;

            last = speed_cmd;
        }
    }
}

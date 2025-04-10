use defmt::{debug, info, warn, Format};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, WaitResult},
};
use embassy_time::Timer;

pub(crate) static FAN_COMMAND: PubSubChannel<CriticalSectionRawMutex, FanCommand, 1, 2, 1> =
    PubSubChannel::new();

#[derive(Clone, Format, Eq, PartialEq)]
pub(crate) enum FanCommand {
    Stop,
    Run(FanSpeed),
}

#[derive(Clone, Format, Eq, PartialEq)]
pub(crate) enum FanSpeed {
    Low,
    Medium,
    High,
}

impl FanSpeed {
    pub(crate) fn cycle(&mut self) {
        *self = match self {
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Low,
        };
    }
}

#[embassy_executor::task]
pub(super) async fn task(r: crate::FanRelayResources) {
    let mut fan_high = Output::new(r.high, Level::Low);
    let mut fan_medium = Output::new(r.medium, Level::Low);
    let mut fan_low = Output::new(r.low, Level::Low);
    let mut contactor_voltage = Output::new(r.contactor_voltage, Level::Low);

    let mut last = FanCommand::Stop;

    let mut rx = FAN_COMMAND.subscriber().unwrap();

    loop {
        match rx.next_message().await {
            WaitResult::Lagged(count) => {
                warn!("Subscriber lagged, lost {} messages", count);
            }
            WaitResult::Message(cmd) => {
                if cmd != last {
                    info!("Set fan to {:?}", cmd);
                    debug!("Open all speed selection contactors");
                    fan_low.set_low();
                    fan_medium.set_low();
                    fan_high.set_low();

                    Timer::after_millis(10).await;

                    if let FanCommand::Run(speed) = cmd.clone() {
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

                    last = cmd;
                }
            }
        }
    }
}

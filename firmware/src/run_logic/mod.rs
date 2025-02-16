mod manual_button_trigger;

use crate::{
    buttons::BUTTON_EVENTS,
    fan::{FanCommand, FAN_COMMAND},
};
use defmt::{info, warn, Format};
use embassy_futures::select::{select, Either};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, WaitResult},
};
use embassy_time::{Duration, Ticker};
use manual_button_trigger::ManualButtonTrigger;

pub(crate) static STATE_CHANGED: PubSubChannel<CriticalSectionRawMutex, State, 1, 2, 1> =
    PubSubChannel::new();

trait Trigger {
    fn fan_command(&self) -> FanCommand;
}

#[derive(Clone, Default, Format)]
pub(crate) struct State {
    button_trigger: ManualButtonTrigger,
}

impl Trigger for State {
    fn fan_command(&self) -> FanCommand {
        self.button_trigger.fan_command()
    }
}

#[embassy_executor::task]
pub(super) async fn task() {
    let mut state = State::default();

    let mut tick_1hz = Ticker::every(Duration::from_hz(1));
    let mut button_sub = BUTTON_EVENTS.subscriber().unwrap();
    let state_pub = STATE_CHANGED.publisher().unwrap();
    let fan_pub = FAN_COMMAND.publisher().unwrap();

    loop {
        let changed = match select(tick_1hz.next(), button_sub.next_message()).await {
            Either::First(_) => state.button_trigger.handle_tick(),
            Either::Second(event) => match event {
                WaitResult::Lagged(count) => {
                    warn!("Subscriber lagged, lost {} messages", count);
                    false
                }
                WaitResult::Message(event) => state.button_trigger.handle_button(event),
            },
        };

        if changed {
            info!("New state: {:?}", state);
            fan_pub.publish(state.fan_command()).await;
            state_pub.publish(state.clone()).await;
        }
    }
}

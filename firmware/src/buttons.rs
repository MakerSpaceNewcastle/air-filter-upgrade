use defmt::{info, Format};
use embassy_futures::select::{select, Either};
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use embassy_time::{Duration, Instant};

pub(crate) static BUTTON_EVENTS: PubSubChannel<CriticalSectionRawMutex, ButtonEvent, 8, 2, 1> =
    PubSubChannel::new();

#[derive(Clone, PartialEq, Eq, Format)]
pub(crate) struct ButtonEvent {
    pub button: Button,
    pub push_duration: ButtonPushDuration,
}

#[derive(Clone, PartialEq, Eq, Format)]
pub(crate) enum Button {
    Demand,
    Speed,
}

#[derive(Clone, PartialEq, Eq, Format)]
pub(crate) enum ButtonPushDuration {
    Short,
    Long,
}

const PUSH_THRESHOLD: Duration = Duration::from_millis(75);
const LONG_PUSH_THRESHOLD: Duration = Duration::from_secs(3);

#[derive(Format)]
enum ButtonState {
    Pressed { at: Instant },
    Released { at: Instant },
}

impl Default for ButtonState {
    fn default() -> Self {
        Self::Released { at: Instant::now() }
    }
}

impl ButtonState {
    fn update(&mut self, level: Level) -> Option<ButtonPushDuration> {
        let now = Instant::now();
        match self {
            ButtonState::Pressed { at } => {
                let since = now - *at;
                if level == Level::High {
                    if since >= LONG_PUSH_THRESHOLD {
                        *self = Self::Released { at: now };
                        return Some(ButtonPushDuration::Long);
                    } else if since >= PUSH_THRESHOLD {
                        *self = Self::Released { at: now };
                        return Some(ButtonPushDuration::Short);
                    }
                }
            }
            ButtonState::Released { at } => {
                let since = now - *at;
                if level == Level::Low && since >= Duration::from_millis(250) {
                    *self = Self::Pressed { at: now };
                }
            }
        }

        None
    }
}

#[embassy_executor::task]
pub(super) async fn task(r: crate::ButtonResources) {
    let mut demand_button = Input::new(r.demand, Pull::Down);
    let mut speed_button = Input::new(r.speed_select, Pull::Down);

    let tx = BUTTON_EVENTS.publisher().unwrap();

    let mut demand_button_state = ButtonState::default();
    let mut speed_button_state = ButtonState::default();

    loop {
        let event = match select(
            demand_button.wait_for_any_edge(),
            speed_button.wait_for_any_edge(),
        )
        .await
        {
            Either::First(_) => {
                demand_button_state
                    .update(demand_button.get_level())
                    .map(|push_duration| ButtonEvent {
                        button: Button::Demand,
                        push_duration,
                    })
            }
            Either::Second(_) => {
                speed_button_state
                    .update(speed_button.get_level())
                    .map(|push_duration| ButtonEvent {
                        button: Button::Speed,
                        push_duration,
                    })
            }
        };

        if let Some(event) = event {
            info!("Button event: {:?}", event);
            tx.publish(event).await;
        }
    }
}

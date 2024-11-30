use defmt::{info, Format};
use embassy_futures::select::{select, Either};
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use embassy_time::Timer;

#[derive(Clone, Format)]
pub(crate) enum PresenceSensor {
    PirA,
    PirB,
}

#[derive(Clone, Eq, PartialEq, Format)]
pub(crate) enum Presence {
    Clear,
    Occupied,
}

impl From<Level> for Presence {
    fn from(value: Level) -> Self {
        match value {
            Level::Low => Self::Clear,
            Level::High => Self::Occupied,
        }
    }
}

#[derive(Clone, Format)]
pub(crate) struct PresenceDetectionEvent {
    pub(crate) sensor: PresenceSensor,
    pub(crate) state: Presence,
}

pub(crate) static PRESENCE_EVENTS: PubSubChannel<
    CriticalSectionRawMutex,
    PresenceDetectionEvent,
    8,
    1,
    1,
> = PubSubChannel::new();

#[embassy_executor::task]
pub(crate) async fn task(r: crate::PresenceSensorResources) {
    let mut pir_a = Input::new(r.pir_a, Pull::Down);
    let mut pir_b = Input::new(r.pir_b, Pull::Down);

    let tx = PRESENCE_EVENTS.publisher().unwrap();

    loop {
        let event = match select(
            wait_for_edge_and_a_bit(&mut pir_a),
            wait_for_edge_and_a_bit(&mut pir_b),
        )
        .await
        {
            Either::First(_) => PresenceDetectionEvent {
                sensor: PresenceSensor::PirA,
                state: pir_a.get_level().into(),
            },
            Either::Second(_) => PresenceDetectionEvent {
                sensor: PresenceSensor::PirB,
                state: pir_b.get_level().into(),
            },
        };

        info!("Presence event: {}", event);
        tx.publish(event).await;
    }
}

async fn wait_for_edge_and_a_bit(input: &mut Input<'static>) {
    input.wait_for_any_edge().await;
    Timer::after_millis(50).await;
}

use defmt::{info, Format};
use embassy_rp::gpio::{Input, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use embassy_time::{Duration, Instant};

#[derive(Clone, Format)]
pub(crate) enum UiEvent {
    SpeedButtonPushed,
}

pub(crate) static UI_EVENTS: PubSubChannel<CriticalSectionRawMutex, UiEvent, 8, 1, 1> =
    PubSubChannel::new();

#[embassy_executor::task]
pub(crate) async fn task(r: crate::UiButtonResources) {
    let mut speed_button = Input::new(r.speed, Pull::Down);

    let tx = UI_EVENTS.publisher().unwrap();

    loop {
        speed_button.wait_for_high().await;
        let time_high = Instant::now();

        speed_button.wait_for_low().await;
        let time_low = Instant::now();

        if (time_low - time_high) > Duration::from_millis(100) {
            let event = UiEvent::SpeedButtonPushed;

            info!("UI event: {}", event);
            tx.publish(event).await;
        }
    }
}

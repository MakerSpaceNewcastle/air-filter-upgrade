mod drawables;
mod no_cs;

use crate::{
    fan::FanCommand,
    run_logic::{Trigger, STATE_CHANGED},
};
use core::cell::RefCell;
use defmt::{debug, warn};
use drawables::{boot_screen::BootScreen, main_screen::MainScreen};
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::{
    gpio::{Level, Output},
    pwm::{Pwm, SetDutyCycle},
};
use embassy_sync::{
    blocking_mutex::{raw::NoopRawMutex, Mutex},
    pubsub::WaitResult,
};
use embassy_time::{Delay, Timer};
use embedded_graphics::{pixelcolor::Rgb565, Drawable};
use mipidsi::{interface::SpiInterface, models::ST7789, options::ColorInversion};
use no_cs::NoCs;

type Color = Rgb565;

#[embassy_executor::task]
pub(super) async fn task(r: crate::DisplayResources) {
    let mut config = embassy_rp::spi::Config::default();
    // Speeds much higher than this reduce the reliability of communication with the display
    // (likely an electrical noise issue since it is fairly close to the contactors).
    config.frequency = 4_000_000;
    config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    config.polarity = embassy_rp::spi::Polarity::IdleHigh;

    let spi = embassy_rp::spi::Spi::new_blocking(r.spi, r.clk, r.mosi, r.miso, config.clone());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let display_spi = SpiDeviceWithConfig::new(&spi_bus, NoCs, config);

    let mut backlight = Pwm::new_output_b(
        r.backlight_pwm,
        r.backlight,
        embassy_rp::pwm::Config::default(),
    );
    let _ = backlight.set_duty_cycle_fully_on();

    let dc = Output::new(r.dc, Level::Low);
    let rst = Output::new(r.rst, Level::Low);

    let mut buffer = [0_u8; 512];
    let interface = SpiInterface::new(display_spi, dc, &mut buffer);

    let mut display = mipidsi::Builder::new(ST7789, interface)
        .display_size(240, 240)
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(rst)
        .init(&mut Delay)
        .unwrap();

    let mut state_sub = STATE_CHANGED.subscriber().unwrap();

    // Show the boot splash screen
    BootScreen {}.draw(&mut display).unwrap();
    Timer::after_secs(3).await;

    let mut main_screen = MainScreen::default();

    loop {
        match state_sub.next_message().await {
            WaitResult::Lagged(count) => {
                warn!("Subscriber lagged, lost {} messages", count);
            }
            WaitResult::Message(state) => {
                debug!("Got new state to draw");

                // Set backlight intensity
                match state.fan_command() {
                    FanCommand::Stop => {
                        let _ = backlight.set_duty_cycle_percent(20);
                    }
                    FanCommand::Run(_) => {
                        let _ = backlight.set_duty_cycle_fully_on();
                    }
                }

                // Update display contents
                main_screen.update_state(state);
                main_screen.draw(&mut display).unwrap();
            }
        }
    }
}

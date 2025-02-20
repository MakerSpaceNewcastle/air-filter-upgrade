#![no_std]
#![no_main]

mod buttons;
mod display;
mod fan;
mod run_logic;
mod temperature_sensors;

use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::{Executor, Spawner};
use embassy_rp::{
    gpio::{Level, Output},
    multicore::{spawn_core1, Stack},
    peripherals,
    watchdog::Watchdog,
};
use embassy_time::{Duration, Ticker};
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use portable_atomic as _;
use static_cell::StaticCell;

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    use embassy_rp::gpio::{Level, Output};

    let p = unsafe { embassy_rp::Peripherals::steal() };
    let r = split_resources!(p);

    // Turn off all fan output contactors
    let relays = r.fan_relays;
    let _ = Output::new(relays.high, Level::Low);
    let _ = Output::new(relays.medium, Level::Low);
    let _ = Output::new(relays.low, Level::Low);
    let _ = Output::new(relays.contactor_voltage, Level::Low);

    let mut led = Output::new(r.status.led, Level::Low);
    loop {
        embassy_time::block_for(Duration::from_hz(20));
        led.toggle();
    }
}

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

assign_resources::assign_resources! {
    fan_relays: FanRelayResources {
        low: PIN_16,
        medium: PIN_6,
        high: PIN_7,
        contactor_voltage: PIN_17,
    },
    buttons: ButtonResources {
        demand: PIN_8, // Isolated input 7
        speed_select: PIN_9, // Isolated input 6
    },
    display: DisplayResources {
        spi: SPI0,
        clk: PIN_2,
        mosi: PIN_3,
        miso: PIN_4,
        dc: PIN_0,
        rst: PIN_1,
        backlight: PIN_5,
        backlight_pwm: PWM_SLICE2,
    },
    onewire: OnewireResources {
        data: PIN_22,
    },
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    },
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    info!("Version: {}", env!("VERSION"));

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                unwrap!(spawner.spawn(watchdog_feed(r.status)));
                unwrap!(spawner.spawn(crate::fan::task(r.fan_relays)));
                unwrap!(spawner.spawn(crate::run_logic::task()));
                unwrap!(spawner.spawn(crate::buttons::task(r.buttons)));
            });
        },
    );

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        // TODO unwrap!(spawner.spawn(crate::temperature_sensors::task(r.onewire)));
        unwrap!(spawner.spawn(crate::display::task(r.display)));
    });
}

#[embassy_executor::task]
async fn watchdog_feed(r: StatusResources) {
    let mut led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_secs(2));

    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        ticker.next().await;

        watchdog.feed();
        led.toggle();
    }
}

use defmt::{info, warn};
use ds18b20::Ds18b20;
use embassy_rp::gpio::{Level, OutputOpenDrain};
use embassy_time::{Delay, Duration, Ticker, Timer};
use one_wire_bus::{Address, OneWire};

#[embassy_executor::task]
pub(super) async fn task(r: crate::OnewireResources) {
    let mut bus = {
        let pin = OutputOpenDrain::new(r.data, Level::Low);
        OneWire::new(pin).unwrap()
    };

    for device_address in bus.devices(false, &mut Delay) {
        let device_address = device_address.unwrap();
        info!("Found one wire device at address: {}", device_address.0);
    }

    let onboard_temp_sensor =
        Ds18b20::new::<()>(Address(env!("BOARD_TEMP_SENSOR_ADDRESS").parse().unwrap())).unwrap();

    let mut ticker = Ticker::every(Duration::from_secs(5));

    loop {
        ticker.next().await;

        ds18b20::start_simultaneous_temp_measurement(&mut bus, &mut Delay).unwrap();

        Timer::after_millis(ds18b20::Resolution::Bits12.max_measurement_time_millis() as u64).await;

        // TODO: do something sensible with the temperature readings
        match onboard_temp_sensor.read_data(&mut bus, &mut Delay) {
            Ok(reading) => info!("Board temperature: {}C", reading.temperature),
            Err(_) => warn!("Failed to read board temperature"),
        }
    }
}

use defmt::{debug, info, warn};
use ds18b20::Resolution;
use embassy_time::{Delay, Duration, Ticker, Timer};

#[embassy_executor::task]
pub(super) async fn task(r: crate::OnewireResources) {
    let mut bus = pico_plc_bsp::onewire::new(r.data).unwrap();

    let mut ticker = Ticker::every(Duration::from_secs(10));

    loop {
        ds18b20::start_simultaneous_temp_measurement(&mut bus, &mut Delay).unwrap();

        Timer::after_millis(Resolution::Bits12.max_measurement_time_millis() as u64).await;

        let mut search_state = None;
        while let Some((device_address, state)) = bus
            .device_search(search_state.as_ref(), false, &mut Delay)
            .unwrap()
        {
            search_state = Some(state);

            if device_address.family_code() == ds18b20::FAMILY_CODE {
                debug!("Found DS18B20 at address: {}", device_address.0);

                let sensor = ds18b20::Ds18b20::new::<()>(device_address).unwrap();
                match sensor.read_data(&mut bus, &mut Delay) {
                    Ok(sensor_data) => {
                        info!(
                            "DS18B20 {} is {}°C",
                            device_address.0, sensor_data.temperature
                        );
                    }
                    Err(_) => {
                        warn!("Failed to read DS18B20 at {}", device_address.0);
                    }
                }
            } else {
                info!(
                    "Found unknown one wire device at address: {}",
                    device_address.0
                );
            }
        }

        ticker.next().await;
    }
}

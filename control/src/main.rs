mod air_filter;
mod app_config;
mod sensor;
mod zone;

use clap::Parser;
use log::{debug, info, trace, warn};
use rumqttc::{AsyncClient, Event, Packet, Publish};
use sensor::SensorUpdate;
use std::time::Duration;
use zone::Zone;

#[derive(Debug, Parser)]
#[command(version = env!("VERSION"), about)]
struct Cli {
    /// Configuration file
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    env_logger::init();

    let config = config::Config::builder()
        .add_source(config::File::with_name(&args.config))
        .build()?;
    let config = config.try_deserialize::<app_config::Config>()?;
    debug!("Zone config: {:#?}", config.zones);

    let (mqtt_client, mut mqtt_connection) = AsyncClient::new(config.mqtt.into(), 16);

    let mut zones: Vec<Zone> = config
        .zones
        .into_iter()
        .map(|config| Zone::new(mqtt_client.clone(), config))
        .collect();

    let subscriptions: Vec<_> = zones.iter().flat_map(|z| z.mqtt_subscriptions()).collect();
    info!("Subscribing to MQTT topics: {:#?}", subscriptions);
    mqtt_client.subscribe_many(subscriptions).await?;

    let mut sensor_update_interval = tokio::time::interval(Duration::from_secs(15));
    let mut new_data = false;

    loop {
        let control_send_wait = tokio::time::sleep(if new_data {
            Duration::from_secs(1)
        } else {
            Duration::MAX
        });

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Exiting");
                break;
            }
            event = mqtt_connection.poll() => {
                trace!("MQTT event: {:?}", event);
                match event {
                    Ok(Event::Incoming(Packet::Publish(msg))) => {
                        if update_zones_via_mqtt_message(&mut zones, &msg).await == Update::Updated {
                            new_data = true;
                        }
                    },
                    Err(e) => warn!("MQTT error: {:?}", e),
                    _ => {}
                }
            }
            _ = sensor_update_interval.tick() => {
                if update_zones_via_time(&mut zones).await == Update::Updated {
                    new_data = true;
                }
            }
            _ = control_send_wait => {
                evaluate_and_send_command(&mut zones).await;
                new_data = false;
            }
        };
    }

    Ok(())
}

async fn update_zones_via_mqtt_message(zones: &mut [Zone], msg: &Publish) -> Update {
    let mut result = Update::NotUpdated;

    info!("Updating sensors via MQTT topic: {}", msg.topic);
    for zone in zones.iter_mut() {
        match zone.update_via_mqtt_message(msg) {
            Ok(Update::Updated) => {
                result = Update::Updated;
            }
            Ok(Update::NotUpdated) => {}
            Err(e) => warn!("Error when updateing zone {}: {:?}", zone.name(), e),
        }
    }

    result
}

async fn update_zones_via_time(zones: &mut [Zone]) -> Update {
    let mut result = Update::NotUpdated;

    info!("Updating sensors on interval");
    for zone in zones.iter_mut() {
        match zone.update_via_time() {
            Ok(Update::Updated) => {
                result = Update::Updated;
            }
            Ok(Update::NotUpdated) => {}
            Err(e) => warn!("Error when updateing zone {}: {:?}", zone.name(), e),
        }
    }

    result
}

async fn evaluate_and_send_command(zones: &mut [Zone]) {
    info!("Evaluating zones and sending updated commands");
    for zone in zones {
        if let Err(e) = zone.evaluate_and_send_command().await {
            warn!(
                "Failed to evalueate zone {} and send air filter command: {:?}",
                zone.name(),
                e
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Update {
    Updated,
    NotUpdated,
}

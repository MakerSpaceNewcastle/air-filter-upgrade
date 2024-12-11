use super::{SensorRead, SensorReading, SensorUpdate};
use crate::Update;
use log::info;
use rumqttc::{QoS, SubscribeFilter};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct PresenceSensorConfig {
    pub(crate) name: String,
    pub(crate) topic: String,

    pub(crate) timeout: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Presence {
    Clear,
    Occupied,
}

pub(crate) struct PresenceSensor {
    config: PresenceSensorConfig,
    reading: Option<SensorReading<Presence, ()>>,
}

impl From<PresenceSensorConfig> for PresenceSensor {
    fn from(config: PresenceSensorConfig) -> Self {
        Self {
            config,
            reading: None,
        }
    }
}

impl SensorRead<Presence, ()> for PresenceSensor {
    fn name(&self) -> &str {
        self.config.name.as_str()
    }

    fn reading(&self) -> Option<SensorReading<Presence, ()>> {
        self.reading.clone()
    }
}

impl SensorUpdate for PresenceSensor {
    fn mqtt_subscriptions(&self) -> Vec<SubscribeFilter> {
        vec![SubscribeFilter::new(
            self.config.topic.clone(),
            QoS::ExactlyOnce,
        )]
    }

    fn update_via_mqtt_message(&mut self, msg: &rumqttc::Publish) -> anyhow::Result<Update> {
        if self.config.topic == msg.topic {
            let s = std::str::from_utf8(&msg.payload)?;

            let value = match s {
                "off" => Presence::Clear,
                "on" => Presence::Occupied,
                _ => anyhow::bail!("unexpected payload for esphome binary sensor"),
            };

            if value == Presence::Clear {
                return Ok(Update::NotUpdated);
            }

            info!(
                "Setting presence sensor {} to {:?} via MQTT",
                self.name(),
                value
            );
            self.reading = Some(SensorReading::new(value, None));

            Ok(Update::Updated)
        } else {
            Ok(Update::NotUpdated)
        }
    }

    fn update_via_time(&mut self) -> anyhow::Result<Update> {
        Ok(if let Some(last) = &self.reading {
            if last.value == Presence::Occupied && last.age().as_secs() > self.config.timeout {
                info!(
                    "Setting presence sensor {} to unoccupied after timeout",
                    self.name()
                );
                self.reading = Some(SensorReading::new(Presence::Clear, None));
                Update::Updated
            } else {
                Update::NotUpdated
            }
        } else {
            Update::NotUpdated
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rumqttc::{Publish, QoS};

    #[test]
    fn basic() {
        let config = PresenceSensorConfig {
            name: "test sensor".into(),
            topic: "test/value".into(),
            timeout: 30,
        };

        let mut sensor: PresenceSensor = config.clone().into();
        assert_eq!(sensor.config, config);

        assert_eq!(sensor.reading(), None);

        assert_eq!(
            sensor
                .update_via_mqtt_message(&Publish::new("test/value", QoS::ExactlyOnce, b"on"))
                .ok(),
            Some(Update::Updated)
        );

        let reading = sensor.reading().unwrap();
        assert_eq!(reading.value, Presence::Occupied);

        assert_eq!(
            sensor
                .update_via_mqtt_message(&Publish::new("test/value", QoS::ExactlyOnce, b"on"))
                .ok(),
            Some(Update::Updated)
        );

        assert_eq!(
            sensor
                .update_via_mqtt_message(&Publish::new("test/value2", QoS::ExactlyOnce, b"off"))
                .ok(),
            Some(Update::NotUpdated)
        );

        let reading = sensor.reading().unwrap();
        assert_eq!(reading.value, Presence::Occupied);
    }
}

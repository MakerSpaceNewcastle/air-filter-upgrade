use log::debug;
use ms_air_filter_protocol::ExternalCommand;
use rumqttc::AsyncClient;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct AirFilterConfig {
    command_topic: String,
}

pub(crate) struct AirFilter {
    mqtt_client: AsyncClient,
    config: AirFilterConfig,
    last_command: Option<ExternalCommand>,
}

impl AirFilter {
    pub(crate) fn new(mqtt_client: AsyncClient, config: AirFilterConfig) -> Self {
        Self {
            mqtt_client,
            config,
            last_command: None,
        }
    }

    pub(crate) async fn command(&mut self, command: ExternalCommand) -> anyhow::Result<()> {
        if self.last_command == Some(command.clone()) {
            debug!("Ignoring duplicate fan command");
        } else {
            debug!("Sending fan command {command:?}");

            let s = serde_json::to_string(&command)?;

            self.mqtt_client
                .publish(
                    &self.config.command_topic,
                    rumqttc::QoS::AtLeastOnce,
                    false,
                    s,
                )
                .await?;

            self.last_command = Some(command);
        }

        Ok(())
    }
}

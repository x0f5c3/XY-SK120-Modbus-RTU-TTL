#[cfg(toml_cfg_mqtt_enabled)]
pub struct MqttConfig {
    pub broker_url: &'static str,
    pub topic_prefix: &'static str,
}

#[cfg(toml_cfg_mqtt_enabled)]
pub const MQTT_CONFIG: MqttConfig = MqttConfig {
    broker_url: "mqtt://broker.local:1883",
    topic_prefix: "xy-sk120",
};

#[cfg(toml_cfg_mqtt_enabled)]
pub async fn publish_status_stub(_payload: &str) {
    let _ = MQTT_CONFIG;
}

#[cfg(not(toml_cfg_mqtt_enabled))]
pub async fn publish_status_stub(_payload: &str) {}

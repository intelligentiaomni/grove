```rust
use serde::Deserialize;

use super::environment::Environment;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub environment: Environment,

    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub storage: StorageSettings,
    pub logging: LoggingSettings,
    pub telemetry: TelemetrySettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageSettings {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingSettings {
    pub level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetrySettings {
    pub otlp_endpoint: Option<String>,
}
```
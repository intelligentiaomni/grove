```rust
use anyhow::Result;
use config::{Config, Environment, File};
use dotenvy::dotenv;

use super::{validate, Settings};

pub fn load_settings() -> Result<Settings> {
    // Load .env if present (ignored in production)
    dotenv().ok();

    let settings: Settings = Config::builder()
        // Optional configuration file
        .add_source(File::with_name("config/default").required(false))
        // Environment-specific overrides
        .add_source(
            File::with_name(&format!(
                "config/{}",
                std::env::var("GROVE_ENV").unwrap_or_else(|_| "development".into())
            ))
            .required(false),
        )
        // Environment variables
        .add_source(
            Environment::with_prefix("GROVE")
                .separator("__")
        )
        .build()?
        .try_deserialize()?;

    validate(&settings)?;

    Ok(settings)
}
```
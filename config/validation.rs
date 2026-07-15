```rust
use anyhow::{bail, Result};

use super::Settings;

pub fn validate(settings: &Settings) -> Result<()> {
    validate_server(settings)?;
    validate_database(settings)?;
    validate_storage(settings)?;

    Ok(())
}

fn validate_server(settings: &Settings) -> Result<()> {
    if settings.server.host.trim().is_empty() {
        bail!("server.host must not be empty");
    }

    if settings.server.port == 0 {
        bail!("server.port must be greater than 0");
    }

    Ok(())
}

fn validate_database(settings: &Settings) -> Result<()> {
    if settings.database.url.trim().is_empty() {
        bail!("database.url must not be empty");
    }

    if settings.database.max_connections == 0 {
        bail!("database.max_connections must be greater than 0");
    }

    Ok(())
}

fn validate_storage(settings: &Settings) -> Result<()> {
    if settings.storage.endpoint.trim().is_empty() {
        bail!("storage.endpoint must not be empty");
    }

    if settings.storage.bucket.trim().is_empty() {
        bail!("storage.bucket must not be empty");
    }

    Ok(())
}
```
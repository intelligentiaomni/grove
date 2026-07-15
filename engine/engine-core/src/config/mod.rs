```rust
//! Configuration subsystem.
//!
//! Responsible for loading, validating, and exposing typed runtime
//! configuration for the engine.

pub mod environment;
pub mod loader;
pub mod settings;
pub mod validation;

pub use environment::Environment;
pub use loader::load_settings;
pub use settings::{
    DatabaseSettings,
    LoggingSettings,
    ServerSettings,
    Settings,
    StorageSettings,
    TelemetrySettings,
};
pub use validation::validate;
```
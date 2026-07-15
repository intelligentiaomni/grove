use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Test,
    Staging,
    Production,
}

impl Environment {
    pub fn is_development(self) -> bool {
        matches!(self, Self::Development)
    }

    pub fn is_test(self) -> bool {
        matches!(self, Self::Test)
    }

    pub fn is_staging(self) -> bool {
        matches!(self, Self::Staging)
    }

    pub fn is_production(self) -> bool {
        matches!(self, Self::Production)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "test" | "testing" => Ok(Self::Test),
            "staging" => Ok(Self::Staging),
            "production" | "prod" => Ok(Self::Production),
            _ => Err(format!("unknown environment: {s}")),
        }
    }
}
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Deserializer};

use crate::alert::Alert;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Map of named alerts.
    #[serde(rename = "alert", deserialize_with = "deserialize_alerts")]
    pub alerts: BTreeMap<String, Alert>,
}

impl Config {
    /// Parse a config file from `path`.
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let config_str = fs::read_to_string(path)
            .context(format!("unable to read config file at `{}`", path.display()))?;

        toml::from_str::<Self>(&config_str).context("invalid config file")
    }
}


/// Deserialize a [`BTreeMap`] of named [`Alert`]s and ensure that the
/// [`BTreeMap`] is non-empty.
fn deserialize_alerts<'de, D>(deserializer: D) -> Result<BTreeMap<String, Alert>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = BTreeMap::<String, Alert>::deserialize(deserializer)?;

    if value.is_empty() {
        return Err(serde::de::Error::custom("alerts must not be empty"));
    }

    Ok(value)
}
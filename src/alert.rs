use std::process::Command;

use anyhow::Context;
use regex::Regex;
use serde::de::Error;
use serde::{Deserialize, Deserializer};

use crate::notification::{Notification, Urgency};

#[derive(Debug, Deserialize)]
pub struct Alert {
    /// Whether all or just some matchers need to be fulfilled
    /// for the alert action to be executed.
    quantity: Quantity,
    /// Matchers which need to be fulfilled by the notification.
    #[serde(deserialize_with = "deserialize_matchers")]
    matchers: Vec<Matcher>,
    /// Command to execute after receiving a [`Notification`] which fulfills
    /// the specified [`Quantity`] of the configured [`Matcher`]s.
    action: String,
}

impl Alert {
    /// Boolean whether `notification` fulfills the specified [`Quantity`] of
    /// the configured [`Matcher`]s.
    pub fn is_fulfilled_by(&self, notification: &Notification) -> bool {
        match self.quantity {
            Quantity::Any => {
                self.matchers.iter().any(|matcher| matcher.is_fulfilled_by(notification))
            }
            Quantity::All => {
                self.matchers.iter().all(|matcher| matcher.is_fulfilled_by(notification))
            }
        }
    }

    /// Execute the action of the [`Alert`].
    pub fn execute(&self) -> anyhow::Result<()> {
        let mut parts = self.action.split(" ");
        let command = parts.next().unwrap();
        let args = parts.collect::<Vec<_>>();

        Command::new(command)
            .args(args)
            .output()
            .context(format!("unable to execute action `{}`", self.action))
            .map(drop)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Quantity {
    /// If any of the specified [`Matcher`]s is fulfilled
    /// the action will be executed.
    Any,
    /// Only if all the specified [`Matcher`]s are fulfilled
    /// the action will be executed.
    All,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "kebab-case")]
pub enum Matcher {
    /// The summary of the notification should match the [`Regex`].
    #[serde(deserialize_with = "deserialize_regex")]
    Summary(Regex),
    /// The name of the app which emitted the notification should match
    /// the [`Regex`].
    #[serde(deserialize_with = "deserialize_regex")]
    AppName(Regex),
    /// The body of the notification should match the [`Regex`].
    #[serde(deserialize_with = "deserialize_regex")]
    Body(Regex),
    /// The notification should have the specified [`Urgency`].
    Urgency(Urgency),
}

impl Matcher {
    /// Boolean whether `notification` fulfills the [`Matcher`].
    pub fn is_fulfilled_by(&self, notification: &Notification) -> bool {
        match self {
            Matcher::Summary(regex) => regex.is_match(&notification.summary),
            Matcher::AppName(regex) => regex.is_match(&notification.app_name),
            Matcher::Body(regex) => regex.is_match(&notification.body),
            Matcher::Urgency(urgency) => &notification.urgency == urgency,
        }
    }
}

/// Deserialize a [`Vec`] of [`Matcher`]s and ensure that the [`Vec`] is
/// non-empty.
fn deserialize_matchers<'de, D>(deserializer: D) -> Result<Vec<Matcher>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Vec::<Matcher>::deserialize(deserializer)?;

    if value.is_empty() {
        return Err(serde::de::Error::custom("matchers must not be empty"));
    }

    Ok(value)
}

/// Deserialize a [`String`] as a [`Regex`].
fn deserialize_regex<'de, D>(deserializer: D) -> Result<Regex, D::Error>
where
    D: Deserializer<'de>,
{
    let str = String::deserialize(deserializer)?;
    Regex::new(&str).map_err(|err| D::Error::custom(format!("invalid regex: {err}")))
}

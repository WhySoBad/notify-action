use std::path::Path;

use anyhow::Context;
use clap::Parser;
use env_logger::Env;

use crate::cli::Cli;
use crate::config::Config;
use crate::notification::Notifications;

mod alert;
mod cli;
mod config;
mod notification;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(Env::new().default_filter_or("info"));

    let cli = Cli::parse();
    let config = Config::new(Path::new(&cli.config_path)).context("unable to initialize config")?;

    log::debug!("config = {config:?}");

    log::info!(
        "found alerts {}",
        config.alerts.keys().map(|name| format!("`{name}`")).collect::<Vec<_>>().join(" ")
    );

    let (_interface, mut rx) =
        Notifications::initialize().await.context("unable to initialize notifications")?;

    log::info!("initialized notification service");

    while let Some(notification) = rx.recv().await {
        log::info!("received notification");
        for (name, alert) in
            config.alerts.iter().filter(|(_, alert)| alert.is_fulfilled_by(&notification))
        {
            log::info!("alert `{name}` is fulfilled by notification");
            match alert.execute() {
                Ok(()) => log::info!("successfully executed action for alert `{name}`"),
                Err(err) => {
                    log::error!("error whilst executing action for alert `{name}`: {err:#}")
                }
            }
        }
    }

    Ok(())
}

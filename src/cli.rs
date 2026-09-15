use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// Path to the config file.
    #[arg(short = 'c', long = "config", default_value_t = String::from("config.toml"), env = "CONFIG_PATH")]
    pub config_path: String,
}

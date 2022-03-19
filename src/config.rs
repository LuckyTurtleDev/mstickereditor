use crate::{CONFIG_FILE, PROJECT_DIRS};
use anyhow::{self, Context};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct MatrixConfig {
	pub homeserver_url: String,
	pub user: String,
	pub access_token: String
}

#[derive(Deserialize)]
pub struct TelegramConfig {
	pub bot_key: String
}

#[derive(Deserialize)]
pub struct Config {
	pub telegram: TelegramConfig,
	pub matrix: MatrixConfig
}

pub fn load_config_file() -> anyhow::Result<Config> {
	let config: Config = toml::from_str(&fs::read_to_string(PROJECT_DIRS.config_dir().join(CONFIG_FILE)).with_context(
		|| {
			format!(
				"Failed to open {}",
				PROJECT_DIRS.config_dir().join(CONFIG_FILE).to_str().unwrap()
			)
		}
	)?)?;
	Ok(config)
}

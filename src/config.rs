use crate::{CONFIG_FILE, PROJECT_DIRS};
use anyhow::{self, Context};
use clap::Parser;
use rgb::RGBA;
use serde::Deserialize;
use std::fs;
use strum_macros::{Display, EnumString};

#[derive(Debug, Deserialize)]
pub struct MatrixConfig {
	pub homeserver_url: String,
	pub user: String,
	pub access_token: String
}

#[derive(Debug, Deserialize)]
pub struct TelegramConfig {
	pub bot_key: String
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Display, EnumString, Parser)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AnimationFormat {
	Gif,
	#[default]
	Webp
}

fn default_color() -> RGBA<u8, bool> {
	RGBA {
		r: 0,
		g: 0,
		b: 0,
		a: true
	}
}

#[derive(Debug, Default, Deserialize)]
pub struct Sticker {
	#[serde(default = "default_color")]
	pub transparent_color: RGBA<u8, bool>,
	#[serde(default)]
	pub animation_format: AnimationFormat
}

#[derive(Debug, Deserialize)]
pub struct Config {
	pub telegram: TelegramConfig,
	pub matrix: MatrixConfig,
	#[serde(default)]
	pub sticker: Sticker
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

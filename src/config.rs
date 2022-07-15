use crate::{CONFIG_FILE, PROJECT_DIRS};
use anyhow::{self, Context};
use clap::Parser;
use serde::Deserialize;
use std::fs;
use strum_macros::{Display, EnumString};

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

#[derive(Debug, Deserialize)]
pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub alpha: bool
}

impl Default for Color {
	fn default() -> Self {
		Color {
			r: 0,
			g: 0,
			b: 0,
			alpha: true
		}
	}
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Display, EnumString, Parser)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AnimationFormat {
	Gif,
	#[default]
	Webp
}

#[derive(Debug, Default, Deserialize)]
pub struct Sticker {
	#[serde(default)]
	pub transparent_color: Color,
	#[serde(default)]
	pub animation_format: AnimationFormat
}

#[derive(Deserialize)]
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

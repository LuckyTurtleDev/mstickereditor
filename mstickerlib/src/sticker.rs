use clap::Parser;
use serde::Deserialize;
use strum_macros::{Display, EnumString};

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
pub enum AnimationFormat {
	#[default]
	Gif,
	Webp
}

#[derive(Debug, Default, Deserialize)]
pub struct Sticker {
	#[serde(default)]
	pub transparent_color: Color,
	#[serde(default)]
	pub animation_format: AnimationFormat
}

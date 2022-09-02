use clap::Parser;
use serde::Deserialize;

use super::sticker::Sticker;

#[derive(Debug, Deserialize)]
pub struct Pack {
	pub name: String,
	pub title: String,
	pub is_video: bool,
	pub stickers: Vec<Sticker>
}

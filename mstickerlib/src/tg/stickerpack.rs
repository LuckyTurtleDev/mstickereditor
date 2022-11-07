use serde::Deserialize;

use super::sticker::Sticker;

#[derive(Debug, Deserialize)]
pub(crate) struct Pack {
	pub name: String,
	pub title: String,
	pub is_video: bool,
	pub stickers: Vec<Sticker>
}

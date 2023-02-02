use serde::Deserialize;

use super::sticker::Sticker;

#[derive(Debug, Deserialize)]
pub(crate) struct Pack {
	pub(crate) name: String,
	pub(crate) title: String,
	pub(crate) is_video: bool,
	pub(crate) stickers: Vec<Sticker>
}

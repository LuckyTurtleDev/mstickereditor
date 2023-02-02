//! Stickerpacks for the [maunium stickerpicker](https://github.com/maunium/stickerpicker), which can be used at matrix clients whitch use the current sticker format, like Element and SchildiChat.
//! The maunium stickerpicker does fully replace the default stickerpicker.

use super::ponies::MetaData;
use monostate::MustBe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StickerPack {
	pub title: String,
	pub id: String,
	#[serde(rename = "net.maunium.telegram.pack")]
	pub tg_pack: Option<TgPackRootInfo>,
	pub stickers: Vec<Sticker>
}

///information about the telegram pack, which was imported
#[derive(Debug, Serialize, Deserialize)]
pub struct TgPackRootInfo {
	pub short_name: String,
	pub hash: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sticker {
	pub body: String,
	pub url: String,
	pub metadata: MetaData,
	pub thumbnail_url: String,
	pub thumbnail_info: MetaData,
	msgtype: MustBe!("m.sticker"),
	pub id: String,
	#[serde(rename = "net.maunium.telegram.sticker")]
	pub tg_sticker: Option<TgStickerInfo>
}

///additonal informations about the original telegram sticker
///stored at stickers->net.maunium.telegram.sticker
#[derive(Debug, Serialize, Deserialize)]
pub struct TgStickerInfo {
	pub pack: TgPackInfo,
	pub id: String,
	pub emoticons: Vec<String>
}

///additonal informations about the original telegram stickerpack in witch the sticker was inculded
///stored at stickers->net.maunium.telegram.sticker->pack
#[derive(Debug, Serialize, Deserialize)]
pub struct TgPackInfo {
	pub id: String,
	pub short_name: String
}

impl From<crate::matrix::sticker::Sticker> for Sticker {
	fn from(value: crate::matrix::sticker::Sticker) -> Self {
		Self {
			body: value.body,
			url: value.image.url.clone(),
			metadata: value.image.meta_data.clone(),
			thumbnail_url: value
				.thumbnail
				.as_ref()
				.map(|f| f.url.to_owned())
				.unwrap_or(value.image.url.clone()),
			thumbnail_info: value.thumbnail.map(|f| f.meta_data).unwrap_or(value.image.meta_data),
			msgtype: Default::default(),
			id: value.image.url,
			tg_sticker: None
		}
	}
}

impl From<crate::matrix::stickerpack::StickerPack> for StickerPack {
	fn from(value: crate::matrix::stickerpack::StickerPack) -> Self {
		Self {
			title: value.title,
			id: value.id,
			tg_pack: None,
			stickers: value.stickers.into_iter().map(|f| f.into()).collect()
		}
	}
}

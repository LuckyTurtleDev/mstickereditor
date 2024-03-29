//! Stickerpacks for the [maunium stickerpicker](https://github.com/maunium/stickerpicker), which can be used at matrix clients whitch use the current sticker format, like Element and SchildiChat.
//! The maunium stickerpicker does fully replace the default stickerpicker.

use crate::matrix::Mxc;

use super::ponies::MetaData;
use monostate::MustBe;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StickerPack {
	pub title: String,
	pub id: String,
	#[serde(rename = "net.maunium.telegram.pack")]
	pub tg_pack: Option<TgPackRootInfo>,
	pub stickers: Vec<Sticker>
}

///information about the telegram pack, which was imported
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TgPackRootInfo {
	pub short_name: String,
	pub hash: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sticker {
	pub body: String,
	pub url: Mxc,
	pub info: StickerInfo,
	/// must always be "m.sticker", use `Default::default` to initialize it.
	#[serde(default)]
	pub msgtype: MustBe!("m.sticker"),
	pub id: String,
	#[serde(rename = "net.maunium.telegram.sticker")]
	pub tg_sticker: Option<TgStickerInfo>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StickerInfo {
	#[serde(flatten)]
	pub image_info: MetaData,
	pub thumbnail_url: Mxc,
	pub thumbnail_info: MetaData
}

///additonal informations about the original telegram sticker
///stored at stickers->net.maunium.telegram.sticker
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TgStickerInfo {
	pub pack: TgPackInfo,
	pub id: String,
	pub emoticons: Vec<String>
}

///additonal informations about the original telegram stickerpack in witch the sticker was inculded
///stored at stickers->net.maunium.telegram.sticker->pack
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TgPackInfo {
	pub id: String,
	pub short_name: String
}

impl From<crate::matrix::sticker::Sticker> for Sticker {
	fn from(value: crate::matrix::sticker::Sticker) -> Self {
		let info = StickerInfo {
			thumbnail_url: value
				.thumbnail
				.as_ref()
				.map(|f| f.url.to_owned())
				.unwrap_or_else(|| value.image.url.clone()),
			thumbnail_info: value
				.thumbnail
				.map(|f| f.meta_data)
				.unwrap_or_else(|| value.image.meta_data.clone()),
			image_info: value.image.meta_data
		};
		Self {
			body: value.body,
			url: value.image.url.clone(),
			info,
			msgtype: Default::default(),
			id: value.image.url.url().to_owned(),
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

use super::{
	sticker_formats::{ponies::MetaData, *},
	stickerpack::TgPackInfo
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sticker {
	pub body: String,
	pub image: Image,
	pub thumbnail: Option<Image>,
	///abbreviation for the sticker, to be used inline
	pub emoticons: Option<String>,
	///unicode emoji with are assioted with the sticker
	pub emoji: Vec<String>,
	pub tg_sticker: Option<TgStickerInfo>
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Image {
	pub url: String,
	pub meta_data: MetaData
}

///info about the original telegram sticker
///this field should not be change manual
#[derive(Debug, Serialize, Deserialize)]
pub struct TgStickerInfo {
	///pack where the sticker is from
	pub pack_info: TgPackInfo,
	pub bot_api_id: Option<String>,
	pub client_api_id: Option<String>,
	pub emoji: Vec<String>
}

impl From<maunium::TgStickerInfo> for TgStickerInfo {
	fn from(value: maunium::TgStickerInfo) -> Self {
		Self {
			pack_info: value.pack.into(),
			bot_api_id: None,
			client_api_id: Some(value.id),
			emoji: value.emoticons
		}
	}
}

impl From<maunium::Sticker> for Sticker {
	fn from(value: maunium::Sticker) -> Self {
		let image = Image {
			url: value.url,
			meta_data: value.metadata
		};
		let thumbnail = Image {
			url: value.thumbnail_url,
			meta_data: value.thumbnail_info
		};
		let thumbnail = if image == thumbnail { None } else { Some(thumbnail) };
		let tg_sticker: Option<TgStickerInfo> = value.tg_sticker.map(|f| f.into());
		Self {
			body: value.body,
			image,
			thumbnail,
			emoticons: None,
			emoji: tg_sticker.as_ref().map(|f| f.emoji.to_owned()).unwrap_or_default(),
			tg_sticker
		}
	}
}

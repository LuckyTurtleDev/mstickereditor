use super::{sticker_formats::ponies::MetaData, stickerpack::TgPackInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sticker {
	pub body: String,
	pub image: Image,
	pub thumbnail: Option<Image>,
	pub tg_sticker: Option<TgStickerInfo>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
	pub url: String,
	pub meta_data: MetaData
}

///info about the original telegram sticker
#[derive(Debug, Serialize, Deserialize)]
pub struct TgStickerInfo {
	///pack where the sticker is from
	pub pack_info: TgPackInfo,
	pub bot_api_id: Option<String>,
	pub client_api_id: Option<String>,
	pub emoticons: Vec<String>
}

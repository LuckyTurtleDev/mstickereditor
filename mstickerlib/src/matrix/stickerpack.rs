use super::{sticker::Sticker, sticker_formats::maunium};
use serde::{Deserialize, Serialize};

///additonal informations about the original telegram sticker pack
///stored at `net.maunium.telegram.pack`
#[derive(Debug, Serialize, Deserialize)]
pub struct TgPackInfo {
	pub name: String,
	pub title: String
}
impl From<&crate::tg::StickerPack> for TgPackInfo {
	fn from(value: &crate::tg::StickerPack) -> Self {
		Self {
			name: value.name.clone(),
			title: value.title.clone()
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StickerPack {
	pub title: String,
	///unique id
	pub id: String,
	pub tg_pack: Option<TgPackInfo>,
	pub stickers: Vec<Sticker>
}

impl From<maunium::TgPackInfo> for TgPackInfo {
	fn from(value: maunium::TgPackInfo) -> Self {
		Self {
			name: value.short_name.clone(),
			title: value.short_name
		}
	}
}
impl From<maunium::TgPackRootInfo> for TgPackInfo {
	fn from(value: maunium::TgPackRootInfo) -> Self {
		Self {
			name: value.short_name.clone(),
			title: value.short_name
		}
	}
}

impl From<maunium::StickerPack> for StickerPack {
	fn from(value: maunium::StickerPack) -> Self {
		Self {
			title: value.title,
			id: value.id,
			tg_pack: value.tg_pack.map(|f| f.into()),
			stickers: value.stickers.into_iter().map(|f| f.into()).collect()
		}
	}
}

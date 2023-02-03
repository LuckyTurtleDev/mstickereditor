use serde::Deserialize;

use crate::image::AnimationFormat;

use super::{sticker::Sticker, tg_get, Config};

#[derive(Debug, Deserialize)]
pub struct StickerPack {
	pub name: String,
	pub title: String,
	pub is_animated: bool,
	pub is_video: bool,
	pub stickers: Vec<Sticker>
}

impl StickerPack {
	pub async fn get(name: &str, tg_config: &Config) -> anyhow::Result<Self> {
		let mut pack: Result<Self, anyhow::Error> = tg_get(tg_config, "getStickerSet", [("name", name)]).await;
		if let Ok(ref mut pack) = pack {
			for (i, sticker) in pack.stickers.iter_mut().enumerate() {
				sticker.pack_name = pack.name.clone();
				sticker.positon = i;
			}
		}
		pack
	}

	///unimplementetd
	pub async fn import_to_matrix(
		self,
		tg_config: &Config,
		animation_format: Option<AnimationFormat>,
		matrix_config: crate::matrix::Config
	) -> anyhow::Result<crate::matrix::stickerpack::StickerPack> {
		let test = self.stickers.iter().enumerate();
		unimplemented!()
	}
}

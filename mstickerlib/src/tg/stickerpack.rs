use serde::Deserialize;

use crate::image::AnimationFormat;

use super::{sticker::Sticker, tg_get, Config};

#[derive(Debug, Deserialize)]
pub struct Pack {
	pub name: String,
	pub title: String,
	pub is_animated: bool,
	pub is_video: bool,
	pub stickers: Vec<Sticker>
}

impl Pack {
	pub async fn get(name: &str, tg_config: &Config) -> anyhow::Result<Pack> {
		tg_get(tg_config, "getStickerSet", [("name", name)]).await
	}

	///unimplementetd
	pub async fn import_to_matrix(
		self,
		tg_config: &Config,
		animation_format: Option<AnimationFormat>,
		matrix_config: crate::matrix::Config
	) -> anyhow::Result<crate::matrix::stickerpack::StickerPack> {
		unimplemented!()
	}
}

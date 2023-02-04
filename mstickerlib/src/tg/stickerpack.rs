use super::{sticker::Sticker, tg_get, Config};
use crate::{database::Database, image::AnimationFormat, matrix};
use anyhow::anyhow;
use futures_util::future::join_all;
use serde::Deserialize;

#[cfg(feature = "log")]
use log::{info, warn};

#[derive(Clone, Debug, Deserialize, Hash)]
pub struct StickerPack {
	pub name: String,
	pub title: String,
	pub is_animated: bool,
	pub is_video: bool,
	pub stickers: Vec<Sticker>
}

impl StickerPack {
	///request a stickerpack, by its name
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

	///Import pack to matrix.
	///Function can partially fail, where some sticker can not be imported.
	///Because of this the postion of failed stickers and the error, which has occurred will is also returned.
	///It should be checked if the stickerpack is empty.
	pub async fn import<D>(
		self,
		animation_format: Option<AnimationFormat>,
		database: Option<&D>,
		tg_config: &Config,
		matrix_config: &matrix::Config
	) -> (matrix::stickerpack::StickerPack, Vec<(usize, anyhow::Error)>)
	where
		D: Database
	{
		#[cfg(feature = "log")]
		info!("import Telegram stickerpack {}({})", self.title, self.name);
		#[cfg(feature = "log")]
		if self.is_video {
			warn!(
				"sticker pack {} include video stickers. Import of video sticker is not supported and will be skip.",
				self.name
			);
		}
		let stickers_import_features = self
			.stickers
			.iter()
			.map(|f| f.import(animation_format.clone(), database, tg_config, matrix_config));
		let stickers = join_all(stickers_import_features).await;
		let mut ok_stickers = Vec::new();
		let mut err_stickers = Vec::new();
		for (i, sticker) in stickers.into_iter().enumerate() {
			match sticker {
				Ok(value) => ok_stickers.push(value),
				Err(err) => err_stickers.push((i, err))
			}
		}
		let stickerpack = matrix::stickerpack::StickerPack {
			title: self.title.clone(),
			id: format!("tg_name_{}", self.name),
			tg_pack: Some((&self).into()),
			stickers: ok_stickers
		};
		#[cfg(feature = "log")]
		if stickerpack.stickers.is_empty() {
			warn!("imported pack {} is empty", self.name);
		}
		(stickerpack, err_stickers)
	}
}

///convert telegram stickerpack url to pack name.
///Url must start with `https://t.me/addstickers/`, `t.me/addstickers/` or `tg://addstickers?set=`
pub fn pack_url_to_name<'a>(url: &'a str) -> anyhow::Result<&'a str> {
	let mut name = url.strip_prefix("https://t.me/addstickers/");
	if name.is_none() {
		name = url.strip_prefix("t.me/addstickers/");
	};
	if name.is_none() {
		name = url.strip_prefix("tg://addstickers?set=");
	};
	name.ok_or(anyhow!("{url:?} does not look like a Telegram StickerPack\nPack url should start with \"https://t.me/addstickers/\", \"t.me/addstickers/\" or \"tg://addstickers?set=\""))
}

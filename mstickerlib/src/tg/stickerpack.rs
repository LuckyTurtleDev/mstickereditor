use super::{sticker::Sticker, tg_get, Config, ImportConfig};
use crate::{database::Database, matrix};
use anyhow::anyhow;
use derive_getters::Getters;
use futures_util::future::join_all;
use serde::Deserialize;

#[cfg(feature = "log")]
use log::{info, warn};

#[derive(Clone, Debug, Deserialize, Getters, Hash)]
#[non_exhaustive]
pub struct StickerPack {
	pub(crate) name: String,
	pub(crate) title: String,
	pub(crate) is_animated: bool,
	pub(crate) is_video: bool,
	pub(crate) stickers: Vec<Sticker>
}

impl StickerPack {
	/// Request a stickerpack by its name.
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

	/// Import this pack to matrix.
	///
	/// This function can partially fail, when the import of some stickers has failed (for example sticker use webm format, or reqwest has failed).
	/// Because of this, the result error type inculde the successful part of the Stickerpack
	/// and a tupple with the postion of failed stickers and the associated error.
	pub async fn import<'a, D>(
		&self,
		tg_config: &Config,
		matrix_config: &matrix::Config,
		advance_config: &ImportConfig<'a, D>
	) -> Result<matrix::stickerpack::StickerPack, (matrix::stickerpack::StickerPack, Vec<(usize, anyhow::Error)>)>
	where
		D: Database
	{
		#[cfg(feature = "log")]
		if log::log_enabled!(log::Level::Info) {
			let mut output = "".to_owned();
			if self.is_animated {
				output += " animations";
			}
			if self.is_video {
				output += " videos";
			}
			if !output.is_empty() {
				output = format!(", include:{output}");
			}
			info!(
				"import Telegram stickerpack {:?}({})    {{{} Stickers{}}}",
				self.title,
				self.name,
				self.stickers.len(),
				output
			);
		}

		let stickers_import_futures = self
			.stickers
			.iter()
			.map(|f| f.import(tg_config, matrix_config, advance_config));
		let stickers = join_all(stickers_import_futures).await;

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
			tg_pack: Some((&self).to_owned().into()),
			stickers: ok_stickers
		};
		#[cfg(feature = "log")]
		if stickerpack.stickers.is_empty() {
			warn!("imported pack {} is empty", self.name);
		}
		if err_stickers.is_empty() {
			Ok(stickerpack)
		} else {
			Err((stickerpack, err_stickers))
		}
	}
}

/// Convert telegram stickerpack url to pack name.
///
/// The url must start with `https://t.me/addstickers/`, `t.me/addstickers/` or
/// `tg://addstickers?set=`.
pub fn pack_url_to_name(url: &str) -> anyhow::Result<&str> {
	url.strip_prefix("https://t.me/addstickers/").or_else(|| {
		url.strip_prefix("t.me/addstickers/")
	}).or_else(|| {
		url.strip_prefix("tg://addstickers?set=")
	}).ok_or_else(|| {
		anyhow!("{url:?} does not look like a Telegram StickerPack\nPack url should start with \"https://t.me/addstickers/\", \"t.me/addstickers/\" or \"tg://addstickers?set=\"")
	})
}

#[cfg(test)]
mod tests {

	use super::{ImportConfig, StickerPack};
	use crate::{database::DummyDatabase, image::AnimationFormat};
	#[cfg(feature = "lottie")]
	use lottieconv::Rgba;
	use std::env;

	async fn import(pack: &str, animation_format: Option<AnimationFormat>) {
		let matrix_config = crate::matrix::Config {
			homeserver_url: "none".to_owned(),
			user: "none".to_owned(),
			access_token: "none".to_owned()
		};
		let tg_config = crate::tg::Config {
			bot_key: env::var("TG_BOT_KEY").expect("environment variables TG_BOT_KEY is not set")
		};
		let pack = StickerPack::get(pack, &tg_config).await.unwrap();
		let import_config = ImportConfig::<DummyDatabase> {
			animation_format,
			database: None,
			dry_run: true,
			..Default::default()
		};
		pack.import(&tg_config, &matrix_config, &import_config).await.unwrap();
	}

	#[tokio::test]
	#[ignore]
	async fn import_simple() {
		import("LINE_Menhera_chan_ENG", Some(AnimationFormat::Webp)).await;
	}

	#[cfg(feature = "lottie")]
	#[tokio::test]
	#[ignore]
	async fn import_webp() {
		import("NSanimated", Some(AnimationFormat::Webp)).await;
	}

	#[cfg(feature = "lottie")]
	#[tokio::test]
	#[ignore]
	async fn import_gif() {
		import(
			"NSanimated",
			Some(AnimationFormat::Gif {
				transparent_color: Rgba {
					r: 0,
					g: 0,
					b: 0,
					a: true
				}
			})
		)
		.await;
	}

	#[tokio::test]
	#[ignore]
	async fn import_none() {
		import("NSanimated", None).await;
	}

	#[cfg(feature = "ffmpeg")]
	#[tokio::test]
	#[ignore]
	async fn import_video_pack_webp() {
		import("pingu_animated", Some(AnimationFormat::Webp)).await;
	}

	/*
	#[cfg(feature = "ffmpeg")]
	#[tokio::test]
	#[ignore]
	async fn import_video_pack_webp_invalid_buffer_size() {
		// test invalid buffer size:
		// BufferSizeFailed: Expected (width * height * 4 = 708608) bytes as input buffer, got 720896 bytes
		// see https://github.com/LuckyTurtleDev/mstickereditor/issues/34
		import("LANI_Kurumi_chan_2_ENG", Some(AnimationFormat::Webp)).await;
	}
	*/
}

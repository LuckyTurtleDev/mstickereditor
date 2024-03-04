use std::sync::Arc;

use super::ImportConfig;
use crate::{
	error::Error,
	image::Image,
	matrix::{self, sticker_formats::ponies, Mxc},
	CLIENT
};
use derive_getters::Getters;
use serde::Deserialize;

#[cfg(feature = "log")]
use log::{info, warn};

///see <https://core.telegram.org/bots/api#photosize>
#[derive(Clone, Debug, Deserialize, Hash)]
#[non_exhaustive]
pub struct PhotoSize {
	/// Identifier for this file, which can be used to download or reuse the file.
	pub file_id: String,
	/// Unique identifier for this file, which is supposed to be the same over time and
	/// for different bots. Can't be used to download or reuse the file.
	pub file_unique_id: String,
	/// Sticker width
	pub width: u32,
	/// Sticker height
	pub height: u32
}
impl PhotoSize {
	/// download the image of the PhotoSize
	pub async fn download(&self, tg_config: &super::Config) -> Result<Image, Error> {
		let file: super::File = super::tg_get(tg_config, "getFile", [("file_id", &self.file_id)]).await?;
		let data = CLIENT
			.get()
			.get(format!(
				"https://api.telegram.org/file/bot{}/{}",
				tg_config.bot_key, file.file_path
			))
			.send()
			.await?
			.bytes()
			.await?
			.to_vec();
		Ok(Image {
			data: Arc::new(data),
			file_name: file.file_path,
			width: self.width,
			height: self.height
		})
	}

	pub async fn import<'a, D>(
		&self,
		tg_config: &super::Config,
		matrix_config: &crate::matrix::Config,
		advance_config: &ImportConfig<'a, D>,
		pack_name: &str,
		positon: usize,
		emoji: Option<&str>,
		thumb: bool
	) -> Result<matrix::sticker::Image, Error>
	where
		D: crate::database::Database
	{
		#[cfg(not(feature = "log"))]
		{
			//disable unused param warning
			let _ = pack_name;
			let _ = positon;
			let _ = thumb;
			let _ = emoji;
		}
		#[cfg(feature = "log")]
		let emoji = emoji.unwrap_or_default();
		#[cfg(feature = "log")]
		let thumb = if thumb { "(Thumbnail)" } else { "" };
		#[cfg(feature = "log")]
		info!("download sticker {pack_name}:{positon:03} {emoji:<2} {thumb}");
		// download and convert sticker from telegram
		let mut image = self.download(tg_config).await?;
		image = image.unpack_tgs().await?;
		if image.file_name.ends_with(".lottie") && !advance_config.keep_lottie {
			// file extension is now checked double.
			// Here and inside `convert_...`
			// But `convert_...` function does not exist, if feature is dissable.
			#[cfg(feature = "lottie")]
			{
				image = image.convert_lottie(advance_config.animation_format).await?;
			}
			#[cfg(not(feature = "lottie"))]
			anyhow::bail!("animated sticker can not be converted, if mstickerlib is compliled without the `lottie` feature.")
		}
		if image.file_name.ends_with(".webm") && !advance_config.keep_webm {
			#[cfg(feature = "ffmpeg")]
			{
				image = image.convert_webm2webp().await?;
			}
			#[cfg(not(feature = "ffmpeg"))]
			anyhow::bail!("video sticker can not be converted, if mstickerlib is compliled without the `ffmpeg` feature.")
		}
		#[cfg(feature = "log")]
		info!("  upload sticker {pack_name}:{positon:03} {emoji:<2} {thumb}");
		let mxc = if advance_config.dry_run {
			#[cfg(feature = "log")]
			{
				warn!("  upload skipped; dryrun");
			}
			Mxc::new("!!! DRY_RUN !!!".to_owned(), Some(image.data.clone())) //cloning Arc is cheap
		} else {
			let (mxc, has_uploded) = image.upload(matrix_config, advance_config.database).await?;
			#[cfg(feature = "log")]
			if !has_uploded {
				info!("  upload skipped; file with this hash was already uploaded");
			}
			#[cfg(not(feature = "log"))]
			let _ = has_uploded; //fix unused warning
			mxc
		};
		let meta_data = ponies::MetaData::try_from(image)?;
		Ok(matrix::sticker::Image { url: mxc, meta_data })
	}
}

#[derive(Clone, Debug, Deserialize, Getters, Hash)]
#[non_exhaustive]
pub struct Sticker {
	/// Emoji associated with the sticker.
	emoji: Option<String>,
	/// Identifier for this file, which can be used to download or reuse the file.
	#[serde(flatten)]
	image: PhotoSize,
	thumbnail: Option<PhotoSize>,
	#[serde(default)] //will be initialize in super::stickerpack::StickerPack::get()
	/// Positon in the stickerpack
	pub(crate) positon: usize,
	#[serde(default)] //will be initialize in … 	TODO: make this less ugly
	pub(crate) pack_name: String,
	/// True if the sticker is [animated](https://telegram.org/blog/animated-stickers).
	is_animated: bool,
	/// True if the sticker is a [video sticker](https://telegram.org/blog/video-stickers-better-reactions).
	is_video: bool
}

impl Sticker {
	/// Import sticker to matrix
	pub async fn import<'a, D>(
		&self,
		tg_config: &super::Config,
		matrix_config: &crate::matrix::Config,
		advance_config: &ImportConfig<'a, D>
	) -> Result<crate::matrix::sticker::Sticker, Error>
	where
		D: crate::database::Database
	{
		// download sticker from telegram
		let image = self
			.image
			.import(
				tg_config,
				matrix_config,
				advance_config,
				&self.pack_name,
				self.positon,
				self.emoji.as_deref(),
				false
			)
			.await?;
		let thumb = match self.thumbnail.as_ref() {
			None => None, //async map is currently not supported by std
			Some(thumb) => Some(
				thumb
					.import(
						tg_config,
						matrix_config,
						advance_config,
						&self.pack_name,
						self.positon,
						self.emoji.as_deref(),
						true
					)
					.await?
			)
		};

		//construct Sticker Struct
		let tg_info = matrix::sticker::TgStickerInfo {
			bot_api_id: Some(self.image.file_id.clone()),
			client_api_id: None,
			emoji: self.emoji.clone().into_iter().collect(),
			pack_name: self.pack_name.clone(),
			index: Some(self.positon)
		};
		let sticker = matrix::sticker::Sticker {
			body: self.emoji.clone().unwrap_or_default(),
			image,
			thumbnail: thumb,
			emoji: self.emoji.clone().into_iter().collect(),
			emoticon: None,
			tg_sticker: Some(tg_info)
		};

		#[cfg(feature = "log")]
		info!(
			"  finish sticker {}:{:03} {}",
			self.pack_name,
			self.positon,
			self.emoji.as_deref().unwrap_or_default()
		);
		Ok(sticker)
	}
}

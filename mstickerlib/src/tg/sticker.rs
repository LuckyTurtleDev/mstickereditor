use crate::{
	image::{AnimationFormat, Image},
	matrix,
	matrix::sticker_formats::ponies,
	CLIENT
};
use anyhow::{anyhow, bail};
use serde::Deserialize;
use std::path::Path;

#[cfg(feature = "log")]
use log::info;

#[derive(Debug, Deserialize)]
pub struct Sticker {
	///Emoji associated with the sticker
	pub emoji: Option<String>,
	///Identifier for this file, which can be used to download or reuse the file
	pub file_id: String,
	///Unique identifier for this file, which is supposed to be the same over time and for different bots. Can't be used to download or reuse the file.
	pub file_unique_id: String,
	//pub thumb: Option<PhotoSize>	TODO
	///Sticker width
	pub width: u32,
	///Sticker height
	pub height: u32,
	#[serde(default)] //will be initialize in super::stickerpack::StickerPack::get()
	///positon at the stickerpack
	pub positon: usize,
	#[serde(default)] //will be initialize in … 	TODO: make this less ugly
	pub pack_name: String,
	///True, if the sticker is [animated](https://telegram.org/blog/animated-stickers)
	pub is_animated: bool,
	///True, if the sticker is a [video sticker](https://telegram.org/blog/video-stickers-better-reactions)
	pub is_video: bool
}

impl Sticker {
	pub async fn download_image(&self, tg_config: &super::Config) -> anyhow::Result<Image> {
		let file: super::File = super::tg_get(tg_config, "getFile", [("file_id", &self.file_id)]).await?;
		let data = CLIENT
			.get()
			.await
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
			data,
			file_name: file.file_path,
			width: self.width,
			height: self.height
		})
	}

	///import sticker to matrix
	pub async fn import<D>(
		&self,
		animation_format: Option<AnimationFormat>,
		database: Option<&D>,
		tg_config: &super::Config,
		matrix_config: &crate::matrix::Config
	) -> anyhow::Result<crate::matrix::sticker::Sticker>
	where
		D: crate::database::Database
	{
		if self.is_video {
			#[cfg(feature = "log")]
			info!(
				"    skip Sticker {}:{:02} {},	is a video",
				self.pack_name,
				self.positon,
				self.emoji.as_deref().unwrap_or_default()
			);
			bail!("sticker is video")
		}
		#[cfg(feature = "log")]
		info!(
			"download sticker {}:{:02} {}",
			self.pack_name,
			self.positon,
			self.emoji.as_deref().unwrap_or_default()
		);

		// download sticker from telegram
		let mut image = self.download_image(tg_config).await?;
		// convert sticker from lottie to gif if neccessary
		if let Some(format) = animation_format {
			image = image.convert_tgs(format.to_owned()).await?;
		}

		// store file on disk if desired
		/*
		if save_to_disk {
			pb.println(format!(
				"    save sticker {:02} {}",
				i + 1,
				self.emoji.as_deref().unwrap_or_default()
			));
			let file_path: &Path = image.file_name.as_ref();
			fs::write(
				Path::new(&format!("./stickers/{}", selfpack.name)).join(file_path.file_name().unwrap()),
				&image.data
			)
			.await?;
		}*/

		let mimetype = format!(
			"image/{}",
			Path::new(&image.file_name)
				.extension()
				.ok_or_else(|| anyhow!("ERROR: extracting mimetype from path {}", image.file_name))?
				.to_str()
				.ok_or_else(|| anyhow!("ERROR: converting mimetype to string"))?
		);

		#[cfg(feature = "log")]
		info!(
			"  upload sticker {}:{:02} {}",
			self.pack_name,
			self.positon,
			self.emoji.as_deref().unwrap_or_default()
		);
		let (mxc_url, has_uploded) = image.upload(matrix_config, database).await?;
		#[cfg(feature = "log")]
		if !has_uploded {
			info!("upload skipped; file with this hash was already uploaded");
		}

		//construct Sticker Struct
		let tg_info = matrix::sticker::TgStickerInfo {
			bot_api_id: Some(self.file_id.clone()),
			client_api_id: None,
			emoji: self.emoji.clone().into_iter().collect(),
			pack_name: self.pack_name.clone()
		};
		let meta_data = ponies::MetaData {
			w: image.width,
			h: image.height,
			size: image.data.len(),
			mimetype
		};
		let sticker_imag = matrix::sticker::Image { url: mxc_url, meta_data };
		let sticker = matrix::sticker::Sticker {
			body: self.emoji.clone().unwrap_or_default(),
			image: sticker_imag,
			thumbnail: None,
			emoji: self.emoji.clone().into_iter().collect(),
			emoticon: None,
			tg_sticker: Some(tg_info)
		};

		#[cfg(feature = "log")]
		info!(
			"  finish sticker {}:{:02} {}",
			self.pack_name,
			self.positon,
			self.emoji.as_deref().unwrap_or_default()
		);
		Ok(sticker)
	}
}

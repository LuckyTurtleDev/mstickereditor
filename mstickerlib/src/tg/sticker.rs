use anyhow::{anyhow, bail};
use clap::Parser;
use flate2::write::GzDecoder;
use lottie2gif::Color;
use lottie2webp;
use std::io::Write;
use tempfile::NamedTempFile;

use serde::Deserialize;
use strum_macros::{Display, EnumString};

#[derive(Clone, Copy, Debug, Default, Deserialize, Display, EnumString, Parser)]
#[strum(serialize_all = "lowercase")]
pub enum AnimationFormat {
	#[default]
	Gif,
	Webp
}

#[derive(Debug, Deserialize)]
pub struct Sticker {
	pub emoji: String,
	pub file_id: String,
	//pub thumb: Option<PhotoSize>	TODO
	pub width: u32,
	pub height: u32,
	pub is_video: bool
}

/// dowloaded image
#[derive(Debug, Deserialize)]
pub(crate) struct Image {
	pub(crate) path: String,
	pub(crate) data: Vec<u8>,
	pub(crate) width: u32,
	pub(crate) height: u32
}

impl Sticker {
	pub(crate) fn download(&self, tg_config: &super::Config) -> anyhow::Result<Image> {
		let file: super::File = super::tg_get(tg_config, "getFile", [("file_id", self.file_id)])?;
		let data = attohttpc::get(format!(
			"https://api.telegram.org/file/bot{}/{}",
			tg_config.bot_key, file.file_path
		))
		.send()?
		.bytes()?;
		Ok(Image {
			data,
			path: file.file_path,
			width: self.width,
			height: self.height
		})
	}
}

impl Image {
	///convert `tgs` sticker to webp or gif
	/// return an error if image is not a tgs
	pub fn convert(mut self, background_color: lottie2gif::Color, format: AnimationFormat) -> anyhow::Result<Self> {
		//save to image to file
		let mut tmp = NamedTempFile::new()?;
		{
			let mut out = GzDecoder::new(&mut tmp);
			out.write_all(&self.data)?;
		}
		tmp.flush()?;
		let animation = rlottie::Animation::from_file(tmp.path()).ok_or_else(|| anyhow!("Failed to load sticker"))?;
		let size = animation.size();
		self.data.clear();
		self.path.truncate(self.path.len() - 3);
		match format {
			AnimationFormat::Gif => {
				lottie2gif::convert(animation, background_color, &mut self.data)?;
				self.path += "gif";
			},
			AnimationFormat::Webp => {
				self.data = match lottie2webp::convert(animation) {
					Ok(value) => value.to_vec(),
					Err(error) => bail!("error converting tgs to webp: {error:?}")
				};
				self.path += "webp";
			}
		}
		self.width = size.width as u32;
		self.height = size.height as u32;
		Ok(self)
	}
}

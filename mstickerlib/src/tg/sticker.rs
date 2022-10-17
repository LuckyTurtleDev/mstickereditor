use anyhow::{anyhow, bail};
use clap::Parser;
use flate2::write::GzDecoder;
use lottie2gif::Color;
use lottie2webp;
use std::io::Write;
use tempfile::NamedTempFile;

use crate::image::Image;
use serde::Deserialize;
use strum_macros::{Display, EnumString};

#[derive(Debug, Deserialize)]
pub struct Sticker {
	pub emoji: String,
	pub file_id: String,
	//pub thumb: Option<PhotoSize>	TODO
	pub width: u32,
	pub height: u32,
	pub is_video: bool
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

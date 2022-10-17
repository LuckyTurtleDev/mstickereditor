use anyhow::{self, bail};
use clap::Parser;
use flate2::write::GzDecoder;
use lottie2gif::Color;
use lottie2webp;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{io::Write, path::Path};
use strum_macros::{Display, EnumString};
use tempfile::NamedTempFile;

use crate::{database, matrix};

#[derive(Clone, Copy, Debug, Default, Deserialize, Display, EnumString, Parser)]
#[strum(serialize_all = "lowercase")]
pub enum AnimationFormat {
	#[default]
	Gif(Color),
	Webp
}

/// cotain a image and its meta data
pub(crate) struct Image {
	pub(crate) path: String,
	pub(crate) data: Vec<u8>,
	pub(crate) width: u32,
	pub(crate) height: u32
}

impl Image {
	pub fn mime_type(&self) -> anyhow::Result<String> {
		Ok(format!(
			"image/{}",
			Path::new(&self.path)
				.extension()
				.ok_or_else(|| anyhow::anyhow!("ERROR: extracting mimetype from path {}", self.path))?
				.to_str()
				.ok_or_else(|| anyhow::anyhow!("ERROR: converting mimetype to string"))?
		))
	}

	/// convert `tgs` image to webp or gif
	/// ignore image if its path does not end with `.tgs`
	pub fn convert_if_tgs(mut self, animation_format: AnimationFormat) -> anyhow::Result<Self> {
		if self.path.ends_with(".tgs") {
			self.convert_tgs(animation_format)
		} else {
			Ok(self)
		}
	}
	/// convert `tgs` image to webp or gif
	pub fn convert_tgs(mut self, animation_format: AnimationFormat) -> anyhow::Result<Self> {
		//save to image to file
		let mut tmp = NamedTempFile::new()?;
		{
			let mut out = GzDecoder::new(&mut tmp);
			out.write_all(&self.data)?;
		}
		tmp.flush()?;
		let animation = rlottie::Animation::from_file(tmp.path()).ok_or_else(|| anyhow::anyhow!("Failed to load image"))?;
		let size = animation.size();
		self.data.clear();
		self.path.truncate(self.path.len() - 3);
		match animation_format {
			AnimationFormat::Gif(background_color) => {
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

	///upload image to matrix
	pub(crate) fn upload<D>(&self, database: Option<D>) -> anyhow::Result<String>
	where
		D: database::Database
	{
		let hash = Lazy::new(|| database::hash(self.data));
		let mxc_url = if let Some(url) = database.map(|db| db.get(&*hash)).flatten() {
			pb.println(format!(
				"  upload sticker {:02} {} skipped; file with this hash was already uploaded",
				i + 1,
				tg_sticker.emoji
			));
			url.clone()
		} else {
			pb.println(format!("  upload sticker {:02} {}", i + 1, tg_sticker.emoji));
			let url = matrix::upload(&config.matrix, self.path, &self.data, &self.mime_type()?)?;
			if let Some(db) = database {
				db.add(*hash, url);
			}
			url
		};
		Ok(mxc_url)
	}
}

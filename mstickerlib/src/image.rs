use anyhow::{anyhow, bail};
use clap::Parser;
use flate2::write::GzDecoder;
use lottieconv::{Animation, Converter, Rgba};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{io::Write, path::Path};
use strum_macros::{Display, EnumString};
use tempfile::NamedTempFile;

use crate::{database, matrix, matrix::Config};

#[derive(Clone, Copy, Debug, Default, Deserialize, Display)]
#[strum(serialize_all = "lowercase")]
pub enum AnimationFormat {
	Gif(Rgba),
	#[default]
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
		let animation = Animation::from_file(tmp.path()).ok_or_else(|| anyhow!("Failed to load sticker"))?;

		let size = animation.size();
		self.data.clear();
		self.path.truncate(self.path.len() - 3);
		let converter = Converter::new(animation);
		match animation_format {
			AnimationFormat::Gif(background_color) => {
				converter.gif(background_color, &mut self.data)?.convert()?;
				self.path += "gif";
			},
			AnimationFormat::Webp => {
				self.data = converter.webp()?.convert()?.to_vec();
				self.path += "webp";
			}
		}
		self.width = size.width as u32;
		self.height = size.height as u32;
		Ok(self)
	}

	///upload image to matrix
	/// return mxc_url and true if image was uploaded now; false if it was already uploaded before and exist at the database
	pub(crate) fn upload<D>(&self, matrix_config: &Config, database: Option<D>) -> anyhow::Result<(String, bool)>
	where
		D: database::Database
	{
		let hash = Lazy::new(|| database::hash(self.data));
		// if database is some and datbase.unwrap().get() is also some
		let ret = if let Some(url) = database.map(|db| db.get(&*hash)).flatten() {
			(url.clone(), false)
		} else {
			let url = matrix::upload(matrix_config, self.path, &self.data, &self.mime_type()?)?;
			if let Some(db) = database {
				db.add(*hash, url);
			}
			(url, true)
		};
		Ok(ret)
	}
}

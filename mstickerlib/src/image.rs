use crate::{database, matrix, matrix::Config};
use anyhow::anyhow;
use flate2::write::GzDecoder;
use lottieconv::{Animation, Converter, Rgba};
use once_cell::sync::Lazy;
use rayon;
use serde::Deserialize;
use std::{io::Write, path::Path};
use strum_macros::Display;
use tempfile::NamedTempFile;
use tokio;

#[derive(Clone, Copy, Debug, Default, Deserialize, Display)]
#[serde(tag = "animation_format", rename_all = "lowercase")]
pub enum AnimationFormat {
	Gif {
		transparent_color: Rgba
	},

	#[default]
	Webp
}

/// generic image struct cotaining, the image data and its meta data
pub struct Image {
	pub file_name: String,
	pub data: Vec<u8>,
	pub width: u32,
	pub height: u32
}

impl Image {
	pub fn mime_type(&self) -> anyhow::Result<String> {
		Ok(format!(
			"image/{}",
			Path::new(&self.file_name)
				.extension()
				.ok_or_else(|| anyhow::anyhow!("ERROR: extracting mimetype from path {}", self.file_name))?
				.to_str()
				.ok_or_else(|| anyhow::anyhow!("ERROR: converting mimetype to string"))?
		))
	}

	pub async fn convert_tgs_if_some(self, animation_format: Option<AnimationFormat>) -> anyhow::Result<Self> {
		match animation_format {
			None => Ok(self),
			Some(animation_format) => self.convert_tgs(animation_format).await
		}
	}

	/// convert `tgs` image to webp or gif, ignore other formats
	pub async fn convert_tgs(mut self, animation_format: AnimationFormat) -> anyhow::Result<Self> {
		if !self.file_name.ends_with(".tgs") {
			return Ok(self);
		}
		fn rayon_run<F, T>(callback: F) -> T
		where
			F: FnOnce() -> T + Send,
			T: Send,
			for<'a> &'a mut T: Send
		{
			let mut result: Option<T> = None;
			rayon::scope(|s| {
				s.spawn(|_| result = Some(callback()));
			});
			result.unwrap()
		}

		tokio::task::spawn_blocking(move || {
			rayon_run(move || {
				//save to image to file
				let mut tmp = NamedTempFile::new()?;
				GzDecoder::new(&mut tmp).write_all(&self.data)?;
				tmp.flush()?;

				let animation = Animation::from_file(tmp.path()).ok_or_else(|| anyhow!("Failed to load sticker"))?;

				let size = animation.size();
				self.data.clear();
				self.file_name.truncate(self.file_name.len() - 3);
				let converter = Converter::new(animation);
				match animation_format {
					AnimationFormat::Gif { transparent_color } => {
						converter.gif(transparent_color, &mut self.data)?.convert()?;
						self.file_name += "gif";
					},
					AnimationFormat::Webp => {
						self.data = converter.webp()?.convert()?.to_vec();
						self.file_name += "webp";
					}
				}
				self.width = size.width as u32;
				self.height = size.height as u32;

				Ok(self)
			})
		})
		.await?
	}

	///upload image to matrix
	/// return mxc_url and true if image was uploaded now; false if it was already uploaded before and exist at the database
	pub async fn upload<D>(&self, matrix_config: &Config, database: Option<&D>) -> anyhow::Result<(String, bool)>
	where
		D: database::Database
	{
		let hash = Lazy::new(|| database::hash(&self.data));

		// if database is some and datbase.unwrap().get() is also some
		if let Some(db) = database {
			if let Some(url) = db.get(&hash).await {
				return Ok((url, false));
			}
		}

		let url = matrix::upload(matrix_config, &self.file_name, &self.data, &self.mime_type()?).await?;
		if let Some(db) = database {
			db.add(*hash, url.clone()).await?;
		}
		Ok((url, true))
	}
}

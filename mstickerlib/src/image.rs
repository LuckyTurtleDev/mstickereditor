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

use crate::{database, matrix, matrix::Config};

#[derive(Clone, Debug, Default, Deserialize, Display)]
#[serde(tag = "animation_format", rename_all = "lowercase")]
pub enum AnimationFormat {
	Gif {
		transparent_color: Rgba
	},

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
	pub(crate) fn mime_type(&self) -> anyhow::Result<String> {
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
	pub(crate) async fn convert_tgs(mut self, animation_format: AnimationFormat) -> anyhow::Result<Self> {
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
				self.path.truncate(self.path.len() - 3);
				let converter = Converter::new(animation);
				match animation_format {
					AnimationFormat::Gif { transparent_color } => {
						converter.gif(transparent_color, &mut self.data)?.convert()?;
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
			})
		})
		.await?
	}

	///upload image to matrix
	/// return mxc_url and true if image was uploaded now; false if it was already uploaded before and exist at the database
	pub(crate) async fn upload<D>(&self, matrix_config: &Config, database: Option<&D>) -> anyhow::Result<(String, bool)>
	where
		D: database::Database
	{
		let hash = Lazy::new(|| database::hash(&self.data));
		// if database is some and datbase.unwrap().get() is also some
		let ret = if let Some(url) = database.and_then(|db| db.get(&hash)) {
			(url, false)
		} else {
			let url = matrix::upload(matrix_config, &self.path, &self.data, &self.mime_type()?).await?;
			if let Some(db) = database {
				db.add(*hash, url.clone())?;
			}
			(url, true)
		};
		Ok(ret)
	}
}

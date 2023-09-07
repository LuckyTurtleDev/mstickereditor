#[cfg(feature = "ffmpeg")]
use crate::video::webm2webp;
use crate::{
	database,
	matrix::{self, Config, Mxc}
};
#[cfg(feature = "lottie")]
use anyhow::anyhow;
#[cfg(feature = "lottie")]
use lottieconv::{Animation, Converter, Rgba};
use once_cell::sync::Lazy;
use serde::Deserialize;
#[cfg(any(feature = "ffmpeg", feature = "lottie"))]
use std::io::Write;
use std::{io::Read, path::Path, sync::Arc};
use strum_macros::Display;
#[cfg(feature = "lottie")]
use tempfile::NamedTempFile;

// todo: remove copy trait. Or will gif support droppet first?
#[derive(Clone, Copy, Debug, Default, Deserialize, Display)]
#[serde(tag = "animation_format", rename_all = "lowercase")]
pub enum AnimationFormat {
	#[cfg(feature = "lottie")]
	Gif { transparent_color: Rgba },

	#[default]
	Webp
}

/// Generic image struct, containing the image data and its meta data.
pub struct Image {
	pub file_name: String,
	pub data: Arc<Vec<u8>>,
	pub width: u32,
	pub height: u32
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

impl Image {
	pub fn mime_type(&self) -> anyhow::Result<String> {
		let extension = Path::new(&self.file_name)
			.extension()
			.ok_or_else(|| anyhow::anyhow!("ERROR: extracting mimetype from path {}", self.file_name))?
			.to_str()
			.ok_or_else(|| anyhow::anyhow!("ERROR: converting mimetype to string"))?;
		Ok(if extension == "webm" {
			format!("video/{extension}",)
		} else {
			format!("image/{extension}",)
		})
	}

	/// unpack gzip compression `tgs`, converting it to `lottie`, ignore other formats
	pub async fn unpack_tgs(mut self) -> anyhow::Result<Self> {
		if !self.file_name.ends_with(".tgs") {
			return Ok(self);
		}
		tokio::task::spawn_blocking(move || {
			rayon_run(move || {
				let mut output = Vec::new();
				let input_reader = &**self.data;
				flate2::read::GzDecoder::new(input_reader).read_to_end(&mut output)?;
				self.data = Arc::new(output);
				self.file_name.truncate(self.file_name.len() - 3);
				self.file_name += "lottie";
				Ok(self)
			})
		})
		.await?
	}

	/// convert `tgs` image to webp or gif, ignore other formats
	#[cfg(feature = "lottie")]
	pub async fn convert_lottie(self, animation_format: AnimationFormat) -> anyhow::Result<Self> {
		if !self.file_name.ends_with(".lottie") {
			return Ok(self);
		}
		let mut image = self.unpack_tgs().await?;
		tokio::task::spawn_blocking(move || {
			rayon_run(move || {
				//save to image to file
				let mut tmp = NamedTempFile::new()?;
				tmp.write_all(&image.data)?;
				tmp.flush()?;
				let animation = Animation::from_file(tmp.path()).ok_or_else(|| anyhow!("Failed to load sticker"))?;
				let size = animation.size();
				image.file_name.truncate(image.file_name.len() - 6);
				let converter = Converter::new(animation);
				match animation_format {
					AnimationFormat::Gif { transparent_color } => {
						let mut data = Vec::new();
						converter.gif(transparent_color, &mut data)?.convert()?;
						image.data = Arc::new(data);
						image.file_name += "gif";
					},
					AnimationFormat::Webp => {
						image.data = Arc::new(converter.webp()?.convert()?.to_vec());
						image.file_name += "webp";
					}
				}
				image.width = size.width as u32;
				image.height = size.height as u32;
				Ok(image)
			})
		})
		.await?
	}

	#[cfg(feature = "ffmpeg")]
	/// convert `webm` video stickers to webp, ignore other formats
	pub async fn convert_webm2webp(mut self) -> anyhow::Result<Self> {
		if !self.file_name.ends_with(".webm") {
			return Ok(self);
		}

		tokio::task::spawn_blocking(move || {
			rayon_run(move || {
				let mut tmp = tempfile::Builder::new().suffix(".webm").tempfile()?;
				tmp.write_all(&self.data)?;
				tmp.flush()?;

				self.file_name.truncate(self.file_name.len() - 1);
				self.file_name += "p";
				let (webp, width, height) = webm2webp(&tmp.path())?;
				self.data = Arc::new(webp.to_vec());
				self.width = width;
				self.height = height;

				Ok(self)
			})
		})
		.await?
	}

	///upload image to matrix
	/// return mxc_url and true if image was uploaded now; false if it was already uploaded before and exist at the database
	pub async fn upload<D>(&self, matrix_config: &Config, database: Option<&D>) -> anyhow::Result<(Mxc, bool)>
	where
		D: database::Database
	{
		let hash = Lazy::new(|| database::hash(&self.data));

		// if database is some and datbase.unwrap().get() is also some
		if let Some(db) = database {
			if let Some(url) = db.get(&hash).await {
				return Ok((url.into(), false));
			}
		}

		let mxc = matrix::upload(matrix_config, &self.file_name, self.data.clone(), &self.mime_type()?).await?;
		if let Some(db) = database {
			db.add(*hash, mxc.url().to_owned()).await?;
		}
		Ok((mxc, true))
	}
}

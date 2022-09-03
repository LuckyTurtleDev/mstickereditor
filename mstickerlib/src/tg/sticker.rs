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
	pub is_video: bool
}

pub fn convert(
	image: Vec<u8>,
	image_name: String,
	color: lottie2gif::Color,
	format: AnimationFormat
) -> anyhow::Result<Vec<u8>> {
	//save to image to file
	let mut tmp = NamedTempFile::new()?;
	{
		let mut out = GzDecoder::new(&mut tmp);
		out.write_all(&image)?;
	}
	tmp.flush()?;
	let animation = rlottie::Animation::from_file(tmp.path()).ok_or_else(|| anyhow!("Failed to load sticker"))?;
	let size = animation.size();
	image.clear();
	image_name.truncate(image_name.len() - 3);
	match format {
		AnimationFormat::Gif => {
			lottie2gif::convert(animation, color, &mut image)?;
			image_name += "gif";
		},
		AnimationFormat::Webp => {
			image = match lottie2webp::convert(animation) {
				Ok(value) => value.to_vec(),
				Err(error) => bail!("error converting tgs to webp: {error:?}")
			};
			image_name += "webp";
		}
	}
	Ok(image)
}

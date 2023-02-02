use crate::{image::Image, CLIENT};
use serde::Deserialize;

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
}

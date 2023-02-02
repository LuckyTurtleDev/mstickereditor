use crate::{image::Image, CLIENT};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Sticker {
	pub(crate) emoji: String,
	pub(crate) file_id: String,
	//pub thumb: Option<PhotoSize>	TODO
	pub(crate) width: u32,
	pub(crate) height: u32,
	pub(crate) is_video: bool
}

impl Sticker {
	pub(crate) async fn download(&self, tg_config: &super::Config) -> anyhow::Result<Image> {
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
			path: file.file_path,
			width: self.width,
			height: self.height
		})
	}
}

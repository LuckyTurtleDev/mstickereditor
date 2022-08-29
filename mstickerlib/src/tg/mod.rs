use anyhow::bail;
use monostate::MustBe;
use serde::{de::DeserializeOwned, Deserialize};
use std::borrow::Borrow;

#[derive(Deserialize)]
pub struct Config {
	pub bot_key: String
}

#[derive(Debug, Deserialize)]
pub struct Sticker {
	pub emoji: String,
	pub file_id: String,
	pub is_video: bool
}

#[derive(Debug, Deserialize)]
pub struct StickerPack {
	pub name: String,
	pub title: String,
	pub is_video: bool,
	pub stickers: Vec<Sticker>
}

#[derive(Debug, Deserialize)]
pub struct StickerFile {
	file_path: String
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TgResponse<T> {
	Ok {
		#[allow(dead_code)]
		ok: MustBe!(true),

		result: T
	},
	Err {
		#[allow(dead_code)]
		ok: MustBe!(false),

		error_code: u32,
		description: String
	}
}

fn tg_get<T, P, K, V>(tg_config: &Config, operation: &str, params: P) -> anyhow::Result<T>
where
	T: DeserializeOwned,
	P: IntoIterator,
	P::Item: Borrow<(K, V)>,
	K: AsRef<str>,
	V: ToString
{
	let resp: TgResponse<T> = attohttpc::get(format!("https://api.telegram.org/bot{}/{}", tg_config.bot_key, operation))
		.params(params)
		.send()?
		.json()?;
	let result = match resp {
		TgResponse::Ok { result, .. } => result,
		TgResponse::Err {
			error_code, description, ..
		} => bail!("Telegram request was not successful: {error_code} {description}")
	};
	Ok(result)
}

pub fn get_stickerpack(tg_config: &Config, name: &str) -> anyhow::Result<StickerPack> {
	tg_get(tg_config, "getStickerSet", [("name", name)])
}

pub fn get_sticker_file(tg_config: &Config, sticker: &Sticker) -> anyhow::Result<StickerFile> {
	tg_get(tg_config, "getFile", [("file_id", &sticker.file_id)])
}

impl StickerFile {
	pub fn download(&self, tg_config: &Config) -> attohttpc::Result<Vec<u8>> {
		attohttpc::get(format!(
			"https://api.telegram.org/file/bot{}/{}",
			tg_config.bot_key, self.file_path
		))
		.send()?
		.bytes()
	}

	pub fn get_file_name(self) -> String {
		self.file_path
	}
}

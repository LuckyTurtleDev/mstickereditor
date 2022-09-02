use anyhow::bail;
use monostate::MustBe;
use serde::{de::DeserializeOwned, Deserialize};
use std::borrow::Borrow;

pub mod sticker;
use sticker::Sticker;
pub mod stickerpack;
use stickerpack::Pack;

#[derive(Deserialize)]
pub struct Config {
	pub bot_key: String
}

/// File storage at Telegram
#[derive(Debug, Deserialize)]
pub struct File {
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

pub fn get_stickerpack(tg_config: &Config, name: &str) -> anyhow::Result<Pack> {
	tg_get(tg_config, "getStickerSet", [("name", name)])
}

impl Sticker {
	pub fn get_file(&self, tg_config: &Config) -> anyhow::Result<File> {
		tg_get(tg_config, "getFile", [("file_id", self.file_id)])
	}
}

impl File {
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

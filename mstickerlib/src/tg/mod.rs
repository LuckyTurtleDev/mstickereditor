use crate::CLIENT;
use anyhow::bail;
use monostate::MustBe;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

mod sticker;
pub use sticker::{PhotoSize, Sticker};

mod stickerpack;
pub use stickerpack::{pack_url_to_name, StickerPack};

#[derive(Debug, Deserialize)]
pub struct Config {
	pub bot_key: String
}

/// File storage at Telegram; see https://core.telegram.org/bots/api#file
#[derive(Debug, Deserialize)]
struct File {
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

async fn tg_get<T, P>(tg_config: &Config, operation: &str, params: P) -> anyhow::Result<T>
where
	T: DeserializeOwned,
	P: Serialize,
	P: Sized
{
	let resp: TgResponse<T> = CLIENT
		.get()
		.await
		.get(format!("https://api.telegram.org/bot{}/{}", tg_config.bot_key, operation))
		.query(&params)
		.send()
		.await?
		.json()
		.await?;
	let result = match resp {
		TgResponse::Ok { result, .. } => result,
		TgResponse::Err {
			error_code, description, ..
		} => bail!("Telegram request was not successful: {error_code} {description}")
	};
	Ok(result)
}

pub(crate) async fn get_stickerpack(tg_config: &Config, name: &str) -> anyhow::Result<StickerPack> {
	tg_get(tg_config, "getStickerSet", [("name", name)]).await
}

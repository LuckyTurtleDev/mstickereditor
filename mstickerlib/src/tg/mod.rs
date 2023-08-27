use crate::{database::Database, image::AnimationFormat, CLIENT};
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

/// additonal, optional configuration for importing stickers
#[non_exhaustive]
pub struct ImportConfig<'a, D = crate::database::DummyDatabase>
where
	D: Database
{
	/// animaton format, to which animated sticker will be converted.
	/// If `None` original format will be used, this is propably not supported by matrix cilents.
	pub animation_format: Option<AnimationFormat>,
	/// database to track, which files was already uploaded,
	/// to aviod duplicaded uploads of the same file
	pub database: Option<&'a D>,
	/// Do not upload anythink to matirx.
	/// **WARNING:** the generate stickerpack will not have valid matrix urls.
	/// Use this function only for testing and to prevent your homesever from being spammed with files while testing.
	pub dry_run: bool
}

impl<D> Default for ImportConfig<'_, D>
where
	D: Database
{
	fn default() -> Self {
		Self {
			animation_format: Some(AnimationFormat::Webp),
			database: None,
			dry_run: false
		}
	}
}

/// File storage at Telegram; see <https://core.telegram.org/bots/api#file>
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

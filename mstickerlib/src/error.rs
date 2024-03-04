use thiserror::Error;

#[derive(Error, Debug)]
#[error("{0:?} does not look like a Telegram StickerPack\nPack url should start with \"https://t.me/addstickers/\", \"t.me/addstickers/\" or \"tg://addstickers?set=\"")]
pub struct InvalidPackUrl(pub String);

#[derive(Error, Debug)]
#[error("Telegram request was not successful: {error_code} {description}")]
pub struct TelgramApiError {
	pub error_code: u32,
	pub description: String
}

#[derive(Error, Debug)]
pub enum Error {
	#[error("{0:?}")]
	Anyhow(#[from] anyhow::Error),
	#[error("{0}")]
	InvalidPackUrl(#[from] InvalidPackUrl),
	#[error("failed to perform request: {0}")]
	Reqwest(#[from] reqwest::Error),
	/// Telegram api has return an error
	#[error("failed to perform request: {0}")]
	Telegram(#[from] TelgramApiError)
}

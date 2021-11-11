use serde::Deserialize;

#[derive(Deserialize)]
pub struct MatrixConfig {
	pub homeserver_url: String,
	pub user: String,
	pub access_token: String
}

#[derive(Deserialize)]
pub struct TelegramConfig {
	pub bot_key: String
}

#[derive(Deserialize)]
pub struct Config {
	pub telegram: TelegramConfig,
	pub matrix: MatrixConfig
}

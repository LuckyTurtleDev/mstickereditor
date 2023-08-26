use crate::CLIENT;
use anyhow::bail;
use reqwest::Url;
use serde::Deserialize;

mod stickerpicker;
use stickerpicker::StickerWidget;
pub mod sticker;
pub mod sticker_formats;
pub mod stickerpack;

#[derive(Debug, Deserialize)]
pub struct Config {
	pub homeserver_url: String,
	pub user: String,
	pub access_token: String
}

#[derive(Debug, Deserialize)]
struct MatrixError {
	errcode: String,
	error: String,
	_retry_after_ms: Option<u32>
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Whoami {
	user_id: String,
	device_id: String
}

#[derive(Debug, Deserialize)]
struct MatrixContentUri {
	content_uri: String
}

pub async fn set_widget(matrix: &Config, sender: String, url: String) -> anyhow::Result<()> {
	let stickerwidget = StickerWidget::new(url, sender);
	let answer = CLIENT
		.get()
		.await
		.put(format!(
			"{}/_matrix/client/r0/user/{}/account_data/m.widgets",
			matrix.homeserver_url, matrix.user
		))
		.query(&[("access_token", &matrix.access_token)])
		.header("Content-Type", "application/json")
		.json(&stickerwidget)
		.send()
		.await?;
	if answer.status() != 200 {
		let status = answer.status();
		let error: Result<MatrixError, _> = answer.json().await;
		let error = error.map(|err| format!(": {} {}", err.errcode, err.error));
		bail!("{} {}", status, error.unwrap_or_default())
	}
	Ok(())
}

pub async fn whoami(matrix: &Config) -> anyhow::Result<Whoami> {
	Url::parse(&matrix.homeserver_url)?; //check if homeserver_url is a valid url
	let answer = CLIENT
		.get()
		.await
		.get(format!("{}/_matrix/client/r0/account/whoami", matrix.homeserver_url))
		.query(&[("access_token", &matrix.access_token)])
		.send()
		.await?;
	if answer.status() != 200 {
		let status = answer.status();
		let error: Result<MatrixError, _> = answer.json().await;
		let error = error.map(|err| format!(": {} {}", err.errcode, err.error));
		bail!("{} {}", status, error.unwrap_or_default())
	} else {
		Ok(answer.json().await?)
	}
}

pub(crate) async fn upload(matrix: &Config, filename: &String, data: &[u8], mimetype: &str) -> anyhow::Result<String> {
	let answer = CLIENT
		.get()
		.await
		.post(&format!("{}/_matrix/media/r0/upload", matrix.homeserver_url))
		.query(&[("access_token", &matrix.access_token), ("filename", filename)])
		.header("Content-Type", mimetype)
		.body(data.to_owned()) //TODO check for better solution
		.send()
		.await?;
	if answer.status() != 200 {
		let status = answer.status();
		let error: Result<MatrixError, _> = answer.json().await;
		let error = error.map(|err| format!(": {} {}", err.errcode, err.error));
		bail!(
			"failed to upload sticker {}: {}{}",
			filename,
			status,
			error.unwrap_or_default()
		);
	}
	let content_uri: MatrixContentUri = answer.json().await?;
	Ok(content_uri.content_uri)
}

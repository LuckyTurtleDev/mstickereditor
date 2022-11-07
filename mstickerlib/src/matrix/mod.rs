use anyhow::bail;
use serde::Deserialize;

mod stickerpicker;
use stickerpicker::StickerWidget;
mod sticker;
pub mod stickerpack;

#[derive(Deserialize)]
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

pub fn set_widget(matrix: &Config, sender: String, url: String) -> anyhow::Result<()> {
	let stickerwidget = StickerWidget::new(url, sender);
	let answer = attohttpc::put(format!(
		"{}/_matrix/client/r0/user/{}/account_data/m.widgets",
		matrix.homeserver_url, matrix.user
	))
	.param("access_token", &matrix.access_token)
	.header("Content-Type", "application/json")
	.json(&stickerwidget)?
	.send()?;
	if answer.status() != 200 {
		let status = answer.status();
		let error: anyhow::Result<String> = (|| {
			let matrix_error: MatrixError = serde_json::from_value(answer.json()?)?;
			Ok(format!(": {} {}", matrix_error.errcode, matrix_error.error))
		})();
		bail!("{} {}", status, error.unwrap_or_default())
	}
	Ok(())
}

pub fn whoami(matrix: &Config) -> anyhow::Result<Whoami> {
	url::Url::parse(&matrix.homeserver_url)?;
	let answer = attohttpc::get(format!("{}/_matrix/client/r0/account/whoami", matrix.homeserver_url))
		.param("access_token", &matrix.access_token)
		.send()?;
	if answer.status() != 200 {
		let status = answer.status();
		let error: anyhow::Result<String> = (|| {
			let matrix_error: MatrixError = serde_json::from_value(answer.json()?)?;
			Ok(format!(": {} {}", matrix_error.errcode, matrix_error.error))
		})();
		bail!("{} {}", status, error.unwrap_or_default())
	} else {
		Ok(answer.json()?)
	}
}

pub(crate) fn upload(matrix: &Config, filename: &String, data: &[u8], mimetype: &str) -> anyhow::Result<String> {
	let answer = attohttpc::post(format!("{}/_matrix/media/r0/upload", matrix.homeserver_url))
		.params([("access_token", &matrix.access_token), ("filename", filename)])
		.header("Content-Type", mimetype)
		.bytes(data)
		.send()?;
	if answer.status() != 200 {
		let status = answer.status();
		let error: anyhow::Result<String> = (|| {
			let matrix_error: MatrixError = serde_json::from_value(answer.json()?)?;
			Ok(format!(": {} {}", matrix_error.errcode, matrix_error.error))
		})();
		bail!(
			"failed to upload sticker {}: {}{}",
			filename,
			status,
			error.unwrap_or_default()
		);
	}
	let content_uri: MatrixContentUri = serde_json::from_value(answer.json()?)?;
	Ok(content_uri.content_uri)
}

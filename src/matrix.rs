use super::MatrixConfig;
use anyhow::bail;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MatrixError {
	errcode: String,
	error: String,
	_retry_after_ms: Option<u32>
}

#[derive(Debug, Deserialize)]
pub struct MatrixWhoami {
	errcode: String,
	error: String
}

pub fn whoami(matrix: &MatrixConfig) -> anyhow::Result<MatrixWhoami> {
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
		bail!("{} {}", status, error.unwrap_or(String::new()))
	} else {
		Ok(answer.json()?)
	}
}

pub fn upload_to_matrix(
	matrix: &MatrixConfig,
	filename: String,
	image_data: &[u8],
	mimetype: &str
) -> anyhow::Result<String> {
	let answer = attohttpc::put(format!("{}/_matrix/media/r0/upload", matrix.homeserver_url))
		.params([("access_token", &matrix.access_token), ("filename", &filename)])
		.header("Content-Type", mimetype)
		.bytes(image_data)
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
			error.unwrap_or(String::new())
		);
	}
	Ok(answer.text()?)
}

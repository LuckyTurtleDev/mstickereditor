use super::MatrixConfig;
use anyhow::bail;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MatrixError {
	errcode: String,
	error: String,
	_retry_after_ms: Option<u32>
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

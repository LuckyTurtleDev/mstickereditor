use super::MatrixConfig;
use anyhow::{anyhow, bail};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct MatrixError {
	errcode: String,
	error: String,
	_retry_after_ms: Option<u32>
}

pub fn upload_to_matrix(
	matrix: &MatrixConfig,
	filename: String,
	image_data: Vec<u8>,
	mimetype: Option<String>
) -> anyhow::Result<String> {
	let url = format!("{}/_matrix/media/r0/upload", matrix.homeserver_url);
	let mimetype = match mimetype {
		Some(value) => value,
		None => format!(
			"image/{}",
			Path::new(&filename)
				.extension()
				.ok_or_else(|| anyhow!("ERROR: extracting mimetype from path {}", filename))?
				.to_str()
				.ok_or_else(|| anyhow!("ERROR: converting mimetype to string"))?
		)
	};
	let answer = attohttpc::put(url)
		.params([("access_token", &matrix.access_token), ("filename", &filename)])
		.header("Content-Type", mimetype)
		.bytes(image_data)
		.send()?; //TODO
	if answer.status() != 200 {
		let status = answer.status();
		let error: Result<String, anyhow::Error> = (|| {
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
	// TODO return the real url here
	Ok(String::new())
}

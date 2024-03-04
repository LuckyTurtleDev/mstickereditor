pub mod sticker;
pub mod sticker_formats;
pub mod stickerpack;
mod stickerpicker;

use crate::{
	error::{Error, MatrixUploadApiError},
	CLIENT
};
use anyhow::bail;
use derive_getters::Getters;
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
	fmt::{Debug, Display},
	ops::Deref,
	sync::Arc
};
use stickerpicker::StickerWidget;
use thiserror::Error;

/// Matrix file url.
///
/// Store matrix file url and provide a cache for the data of the file, to avoid unecessary reqwest
/// Will be serialize/deserialize as normal [String], conaining only the url.
#[derive(Clone, Getters)]
pub struct Mxc {
	url: String,
	/// file data of the url, if cached
	pub(crate) data: Option<Arc<Vec<u8>>>
}
impl Mxc {
	/// create new [Mxc] from matrix url and optional assioated file data
	pub fn new(url: String, data: Option<Arc<Vec<u8>>>) -> Self {
		Self { url, data }
	}

	/// fetch data, if not cached
	pub async fn fetch_data(&self) -> &Vec<u8> {
		if let Some(data) = &self.data {
			return data;
		};
		unimplemented!() //TODO
	}
}
impl From<String> for Mxc {
	fn from(val: String) -> Self {
		Mxc { url: val, data: None }
	}
}

impl AsRef<String> for Mxc {
	fn as_ref(&self) -> &String {
		&self.url
	}
}
impl Deref for Mxc {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.url
	}
}
impl Debug for Mxc {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Debug::fmt(&self.url, f)
	}
}
impl Display for Mxc {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&self.url, f)
	}
}
impl PartialEq for Mxc {
	fn eq(&self, other: &Self) -> bool {
		self.url.eq(&other.url)
	}
}
impl Eq for Mxc {
	fn assert_receiver_is_total_eq(&self) {
		self.url.assert_receiver_is_total_eq()
	}
}

impl<'de> Deserialize<'de> for Mxc {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>
	{
		let url = String::deserialize(deserializer)?;
		Ok(Self { url, data: None })
	}
}
impl Serialize for Mxc {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer
	{
		serializer.serialize_str(&self.url)
	}
}

#[derive(Debug, Deserialize)]
pub struct Config {
	pub homeserver_url: String,
	pub user: String,
	pub access_token: String
}

/// see <https://spec.matrix.org/latest/client-server-api/#standard-error-response>
#[derive(Debug, Deserialize, Error)]
#[error("Matrix api request was not successful: {errcode} {error}")]
pub struct MatrixError {
	/// see <https://spec.matrix.org/latest/client-server-api/#common-error-codes>
	pub errcode: String,
	/// A human-readable error message.
	pub error: String,
	pub retry_after_ms: Option<u32>
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

pub(crate) async fn upload(matrix: &Config, filename: &String, data: Arc<Vec<u8>>, mimetype: &str) -> Result<Mxc, Error> {
	let mut mxc = upload_ref(matrix, filename, data.as_slice(), mimetype).await?;
	mxc.data = Some(data);
	Ok(mxc)
}

pub(crate) async fn upload_ref(matrix: &Config, filename: &String, data: &[u8], mimetype: &str) -> Result<Mxc, Error> {
	let answer = CLIENT
		.get()
		.post(&format!("{}/_matrix/media/r0/upload", matrix.homeserver_url))
		.query(&[("access_token", &matrix.access_token), ("filename", filename)])
		.header("Content-Type", mimetype)
		.body(data.to_owned()) //TODO check for better solution
		.send()
		.await?;
	if answer.status() != 200 {
		let status = answer.status();
		let error: Result<MatrixError, _> = answer.json().await;
		return Err(Error::MatrixUpload(MatrixUploadApiError {
			status_code: status,
			filename: filename.to_owned(),
			matrix_error: error
		}));
	}
	let content_uri: MatrixContentUri = answer.json().await?;
	Ok(content_uri.content_uri.into())
}

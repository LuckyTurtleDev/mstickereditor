pub use crate::matrix::MatrixError;
use reqwest::StatusCode;
use std::{fmt::Display, io};
use thiserror::Error;
use tokio::task::JoinError;

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
#[error("no extension/mimetyp found at sticker filename")]
pub struct NoMimeType;

#[derive(Error, Debug)]
pub struct MatrixUploadApiError {
	pub status_code: StatusCode,
	/// entry is a Result, since getting the error itself can also fail
	pub matrix_error: Result<MatrixError, reqwest::Error>,
	pub filename: String
}

impl Display for MatrixUploadApiError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"failed to upload sticker {:?} with status code {}:\n",
			self.filename, self.status_code
		)?;
		match &self.matrix_error {
			Ok(value) => write!(f, "{value}"),
			Err(error) => write!(f, "Error getting error: {error}")
		}
	}
}

#[cfg(any(not(feature = "ffmpeg"), not(feature = "lottie")))]
#[derive(Error, Debug)]
pub enum UnsupportedFormat {
	#[cfg(not(feature = "lottie"))]
	Lottie,
	#[cfg(not(feature = "ffmpeg"))]
	Webm
}

#[cfg(any(not(feature = "ffmpeg"), not(feature = "lottie")))]
impl Display for UnsupportedFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			#[cfg(not(feature = "lottie"))]
			Self::Lottie => write!(f, "animated")?,
			#[cfg(not(feature = "ffmpeg"))]
			Self::Webm => write!(f, "video")?
		}
		write!(f, " sticker are unsupported, since mstickerlib was compliled without the ",)?;
		match self {
			#[cfg(not(feature = "lottie"))]
			Self::Lottie => write!(f, "\"lottie\"")?,
			#[cfg(not(feature = "ffmpeg"))]
			Self::Webm => write!(f, "\"ffmpeg\"")?
		}
		write!(f, " feature.")
	}
}

#[derive(Error, Debug)]
pub enum Error {
	#[error(transparent)]
	InvalidPackUrl(#[from] InvalidPackUrl),
	#[error("failed to perform request: {0}")]
	Reqwest(#[from] reqwest::Error),
	/// Telegram api has return an error
	#[error("failed to perform request: {0}")]
	Telegram(#[from] TelgramApiError),
	#[error(transparent)]
	IoError(#[from] io::Error),
	#[cfg(feature = "ffmpeg")]
	#[error("failed to convert webm sticker: {0}")]
	Ffmpeg(#[from] ffmpeg::Error),
	#[error("failed to join task: {0}")]
	JoinError(#[from] JoinError),
	#[cfg(feature = "lottie")]
	/// sadly we do not get more information about the error from the lottie crate
	#[error("failed to load sticker from tmp file")]
	AnimationLoadError,
	#[cfg(feature = "lottie")]
	#[error("failed to deencode sticker as gif: {0}")]
	GifDecoding(#[from] gif::DecodingError),
	#[cfg(feature = "lottie")]
	#[error("failed to encode sticker as gif: {0}")]
	GifEncoding(#[from] gif::EncodingError),
	#[cfg(any(feature = "lottie", feature = "ffmpeg"))]
	#[error("failed to en- or decode sticker as webp: {0}")]
	Webp(#[from] webp_animation::Error),
	#[error(transparent)]
	NoMimeType(#[from] NoMimeType),
	/// to avoid that this struct is generic for the database error use anyhow
	/// This is the error crated by the user choosen databe trait impl at the import function function
	#[error("failed to insert or check for file duplicate at the database: {0:?}")]
	Database(anyhow::Error),
	#[error(transparent)]
	MatrixUpload(#[from] MatrixUploadApiError),
	#[cfg(any(not(feature = "ffmpeg"), not(feature = "lottie")))]
	#[error(transparent)]
	UnsupportedFormat(#[from] UnsupportedFormat)
}

use clap::Parser;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, Deserialize)]
pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub alpha: bool
}

impl Default for Color {
	fn default() -> Self {
		Color {
			r: 0,
			g: 0,
			b: 0,
			alpha: true
		}
	}
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Display, EnumString, Parser)]
#[strum(serialize_all = "lowercase")]
pub enum AnimationFormat {
	#[default]
	Gif,
	Webp
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
	#[serde(default)]
	pub transparent_color: Color,
	#[serde(default)]
	pub animation_format: AnimationFormat
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sticker {
	pub body: String,
	pub url: String,
	pub info: StickerInfo,
	#[serde(default = "default_msgtype")]
	pub msgtype: String,
	pub id: String,
	#[serde(rename = "net.maunium.telegram.sticker")]
	pub tg_sticker: TgInfo
}

fn default_msgtype() -> String {
	"m.sticker".to_owned()
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StickerInfo {
	#[serde(flatten)]
	pub metadata: Metadata,
	pub thumbnail_url: String,
	pub thumbnail_info: Metadata
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
	pub w: u32,
	pub h: u32,
	pub size: usize,
	pub mimetype: String
}

///additonal informations about the original telegram sticker
///stored at stickers->net.maunium.telegram.sticker
#[derive(Debug, Serialize, Deserialize)]
pub struct TgInfo {
	pack: TgPackInfo,
	id: String,
	emoticons: Vec<String>
}

///additonal informations about the original telegram stickerpack in witch the sticker was inculded
///stored at stickers->net.maunium.telegram.sticker->pack
#[derive(Debug, Serialize, Deserialize)]
pub struct TgPackInfo {
	pub id: String,
	pub short_name: String
}

//! Ponies([MSC2545](https://github.com/matrix-org/matrix-spec-proposals/pull/2545)) are the new upcomming matrix sticker standard.
//! They allow room and personal stickerpacks.
//! This is already supported by many matrix clients like Neko, Cinny, Fluffychat and more.
//! Keep in mind that ponies specification is not stable yet.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct PackInfo {
	display_name: String,
	avatar_url: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct StickerPack {
	images: IndexMap<String, Sticker>,
	pack: PackInfo
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Usage {
	Sticker,
	Emoticon
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaData {
	pub w: u32,
	pub h: u32,
	pub size: usize,
	pub mimetype: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sticker {
	body: String,
	info: MetaData,
	url: String,
	usage: HashSet<Usage>
}

impl From<crate::matrix::sticker::Sticker> for Sticker {
	fn from(value: crate::matrix::sticker::Sticker) -> Self {
		Self {
			body: value.body,
			url: value.image.url,
			info: value.image.meta_data,
			usage: [Usage::Sticker].into_iter().collect()
		}
	}
}

impl From<crate::matrix::stickerpack::StickerPack> for StickerPack {
	fn from(value: crate::matrix::stickerpack::StickerPack) -> Self {
		Self {
			images: value
				.stickers
				.into_iter()
				.enumerate()
				.map(|(i, sticker)| (i.to_string(), sticker.into()))
				.collect(),
			pack: PackInfo {
				display_name: value.title,
				avatar_url: None
			}
		}
	}
}

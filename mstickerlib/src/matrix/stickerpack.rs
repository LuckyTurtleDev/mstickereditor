use serde::{Deserialize, Serialize};

use super::sticker::{Metadata, Sticker, StickerInfo, TgInfo, TgPackInfo};

///additonal informations about the original telegram sticker pack
///stored at `net.maunium.telegram.pack`
#[derive(Debug, Serialize, Deserialize)]
pub struct TgPack {
	pub short_name: String,
	pub hash: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StickerPack {
	pub title: String,
	pub id: String,
	#[serde(rename = "net.maunium.telegram.pack")]
	pub tg_pack: TgPack,

	pub stickers: Vec<Sticker>
}

impl StickerPack {
	#[cfg(feature = "bin")]
	pub(crate) fn new(tg_pack: &tg::StickerPack, stickers: &[crate::sub_commands::import::Sticker]) -> Self {
		Self {
			title: tg_pack.title.clone(),
			id: format!("tg_name_{}", tg_pack.name.clone()),
			tg_pack: TgPack {
				short_name: tg_pack.name.clone(),
				hash: String::from("unimplemented!")
			},

			stickers: stickers
				.iter()
				.map(|sticker| {
					let divisor = (sticker.width as f32 / 256.0)
						.round()
						.max((sticker.height as f32 / 256.0).round()) as u32;
					let metadata = Metadata {
						w: sticker.width / divisor,
						h: sticker.height / divisor,
						size: sticker.file_size,
						mimetype: sticker.mimetype.clone()
					};
					Sticker {
						body: sticker.emoji.clone(),
						url: sticker.mxc_url.clone(),
						info: StickerInfo {
							metadata: metadata.clone(),
							thumbnail_url: sticker.mxc_url.clone(),
							thumbnail_info: metadata
						},
						msgtype: "m.sticker".to_owned(),
						id: format!("tg_file_id_{}", sticker.file_id),
						tg_sticker: TgInfo {
							pack: TgPackInfo {
								id: format!("tg_file_id_{}", sticker.file_id),
								short_name: tg_pack.name.clone()
							},
							id: sticker.file_id.clone(),
							emoticons: vec![sticker.emoji.clone()]
						}
					}
				})
				.collect()
		}
	}
}

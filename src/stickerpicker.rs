use crate::tg;
use serde::Serialize;

#[derive(Serialize)]
pub struct StickerPack {
	pub title: String,
	pub id: String,

	#[serde(rename = "net.maunium.telegram.pack")]
	pub tg_pack: TgPack,

	pub stickers: Vec<Sticker>
}

#[derive(Serialize)]
pub struct TgPack {
	pub short_name: String,
	pub hash: String
}

#[derive(Serialize)]
pub struct Sticker {
	pub body: String,
	pub url: String,
	pub info: StickerInfo,
	pub msgtype: &'static str,
	pub id: String,

	#[serde(rename = "net.maunium.telegram.sticker")]
	pub tg_sticker: TgSticker
}

#[derive(Clone, Serialize)]
pub struct Metadata {
	pub w: u32,
	pub h: u32,
	pub size: usize,
	pub mimetype: String
}

#[derive(Serialize)]
pub struct StickerInfo {
	#[serde(flatten)]
	pub metadata: Metadata,

	pub thumbnail_url: String,
	pub thumbnail_info: Metadata
}

#[derive(Serialize)]
pub struct TgSticker {
	pack: TgStickerPack,
	id: String,
	emoticons: Vec<String>
}

#[derive(Serialize)]
pub struct TgStickerPack {
	pub id: String,
	pub short_name: String
}

impl StickerPack {
	pub(crate) fn new(tg_pack: &tg::StickerPack, stickers: &[super::Sticker]) -> Self {
		Self {
			title: tg_pack.title.clone(),
			id: unimplemented!(),

			tg_pack: TgPack {
				short_name: tg_pack.name.clone(),
				hash: unimplemented!()
			},

			stickers: stickers
				.iter()
				.map(|sticker| {
					let metadata = Metadata {
						w: sticker.width,
						h: sticker.height,
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
						msgtype: "m.sticker",
						id: unimplemented!(),
						tg_sticker: TgSticker {
							pack: TgStickerPack {
								id: unimplemented!(),
								short_name: tg_pack.name.clone()
							},
							id: unimplemented!(),
							emoticons: unimplemented!()
						}
					}
				})
				.collect()
		}
	}
}

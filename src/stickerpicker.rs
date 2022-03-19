use crate::tg;
use serde::Serialize;

//###########################################################
//#### Stickerwidget
//###########################################################

#[derive(Serialize)]
pub struct StickerWidget {
	pub content: Content,
	pub sender: String,
	pub state_key: String,
	#[serde(rename = "stype")]
	pub stype: String,
	pub id: String
}

#[derive(Serialize)]
pub struct Content {
	#[serde(rename = "stype")]
	pub stype: String,
	pub url: String,
	pub name: String,
	pub data: String
}

impl StickerWidget {
	pub(crate) fn new(url: String, sender: String) -> Self {
		let content = Content {
			stype: String::from("m.stickerpicker"),
			url,
			name: String::from("Stickerpicker"),
			data: String::from("")
		};
		StickerWidget {
			content,
			sender,
			state_key: String::from("stickerpicker"),
			stype: String::from("m.widget"),
			id: String::from("stickerpicker")
		}
	}
}

//###########################################################
//#### Stickerpack
//###########################################################

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
						msgtype: "m.sticker",
						id: format!("tg_file_id_{}", sticker.file_id),
						tg_sticker: TgSticker {
							pack: TgStickerPack {
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

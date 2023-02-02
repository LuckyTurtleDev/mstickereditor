use super::{
	sticker::{Sticker, TgStickerInfo},
	sticker_formats::{maunium, ponies::MetaData}
};
use crate::{
	database,
	image::AnimationFormat,
	tg::{self, sticker::Sticker as TgSticker}
};
use anyhow::anyhow;
use colored::*;
use futures_util::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

///additonal informations about the original telegram sticker pack
///stored at `net.maunium.telegram.pack`
#[derive(Debug, Serialize, Deserialize)]
pub struct TgPackInfo {
	pub name: String,
	pub title: String
}
impl From<&crate::tg::stickerpack::Pack> for TgPackInfo {
	fn from(value: &crate::tg::stickerpack::Pack) -> Self {
		Self {
			name: value.name.clone(),
			title: value.title.clone()
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StickerPack {
	pub title: String,
	///unique id
	pub id: String,
	pub tg_pack: Option<TgPackInfo>,
	pub stickers: Vec<Sticker>
}

impl StickerPack {
	//this is ugly
	#[allow(clippy::too_many_arguments)]
	async fn import_stickers<D>(
		i: usize,
		tg_sticker: &TgSticker,
		pb: &ProgressBar,
		tg_config: &tg::Config,
		animation_format: Option<&AnimationFormat>,
		save_to_disk: bool,
		tg_stickerpack: &tg::stickerpack::Pack,
		dryrun: bool,
		matrix_config: &super::Config,
		database: Option<&D>
	) -> anyhow::Result<Option<Sticker>>
	where
		D: database::Database + Sync + Send
	{
		if tg_sticker.is_video {
			pb.println(
				format!("    skip Sticker {:02} {},	is a video", i + 1, tg_sticker.emoji)
					.yellow()
					.to_string()
			);
			return Ok(None);
		}
		pb.println(format!("download sticker {:02} {}", i + 1, tg_sticker.emoji));

		// download sticker from telegram
		let mut image = tg_sticker.download(tg_config).await?;
		// convert sticker from lottie to gif if neccessary
		if image.path.ends_with(".tgs") {
			if let Some(format) = animation_format {
				image = image.convert_tgs(format.to_owned()).await?;
			}
		}

		// store file on disk if desired
		if save_to_disk {
			pb.println(format!("    save sticker {:02} {}", i + 1, tg_sticker.emoji));
			let file_path: &Path = image.path.as_ref();
			fs::write(
				Path::new(&format!("./stickers/{}", tg_stickerpack.name)).join(file_path.file_name().unwrap()),
				&image.data
			)
			.await?;
		}

		let mut sticker = None;
		if !dryrun {
			let mimetype = format!(
				"image/{}",
				Path::new(&image.path)
					.extension()
					.ok_or_else(|| anyhow!("ERROR: extracting mimetype from path {}", image.path))?
					.to_str()
					.ok_or_else(|| anyhow!("ERROR: converting mimetype to string"))?
			);

			pb.println(format!("  upload sticker {:02} {}", i + 1, tg_sticker.emoji));
			let (mxc_url, has_uploded) = image.upload(matrix_config, database).await?;
			if !has_uploded {
				pb.println("upload skipped; file with this hash was already uploaded")
			}

			//construct Sticker Struct
			let tg_info = TgStickerInfo {
				bot_api_id: Some(tg_sticker.file_id.clone()),
				client_api_id: None,
				emoji: vec![tg_sticker.emoji.to_owned()],
				pack_info: tg_stickerpack.into()
			};
			let meta_data = MetaData {
				w: image.width,
				h: image.height,
				size: image.data.len(),
				mimetype
			};
			let sticker_imag = super::sticker::Image { url: mxc_url, meta_data };
			sticker = Some(Sticker {
				body: tg_sticker.emoji.clone(),
				image: sticker_imag,
				thumbnail: None,
				emoji: vec![tg_sticker.emoji.clone()],
				emoticons: None,
				tg_sticker: Some(tg_info)
			});
		}

		pb.println(format!("  finish sticker {:02} {}", i + 1, tg_sticker.emoji));
		pb.inc(1);
		Ok(sticker)
	}

	/// import a telegram sticker pack
	pub async fn import_pack<D>(
		pack: &str,
		database: Option<&D>,
		tg_config: &tg::Config,
		dryrun: bool,
		save_to_disk: bool,
		matrix_config: &super::Config,
		animation_format: Option<&AnimationFormat>
	) -> anyhow::Result<Self>
	where
		D: database::Database + Sync + Send
	{
		let tg_stickerpack = tg::get_stickerpack(tg_config, pack).await?;
		println!("found Telegram stickerpack {}({})", tg_stickerpack.title, tg_stickerpack.name);
		if save_to_disk {
			fs::create_dir_all(format!("./stickers/{}", tg_stickerpack.name)).await?;
		}
		let pb = ProgressBar::new(tg_stickerpack.stickers.len() as u64);
		pb.set_style(
			ProgressStyle::default_bar()
				.template("[{wide_bar:.cyan/blue}] {pos:>3}/{len} {msg}")
				.progress_chars("#> ")
		);
		if tg_stickerpack.is_video {
			pb.println(
				format!(
					"WARNING: sticker pack {} include video stickers. This are current not supported and will be skipped.",
					tg_stickerpack.name
				)
				.yellow()
				.to_string()
			);
		}

		let sticker_futures = tg_stickerpack.stickers.iter().enumerate().map(|(i, tg_sticker)| {
			Self::import_stickers(
				i,
				tg_sticker,
				&pb,
				tg_config,
				animation_format,
				save_to_disk,
				&tg_stickerpack,
				dryrun,
				matrix_config,
				database
			)
		});
		let stickers: Vec<Sticker> = join_all(sticker_futures)
			.await
			.into_iter()
			.filter_map(|res: anyhow::Result<Option<Sticker>>| match res {
				Ok(sticker) => sticker,
				Err(err) => {
					pb.println(format!("ERROR: {err:?}").red().to_string());
					pb.println("Stickerpack will not be complete".yellow().to_string());
					None
				}
			})
			.collect();
		pb.finish();

		// save the stickerpack to file
		println!("save stickerpack {} to {}.json", tg_stickerpack.title, tg_stickerpack.name);
		let stickerpack = StickerPack {
			title: tg_stickerpack.title.clone(),
			id: format!("tg_name_{}", tg_stickerpack.name),
			tg_pack: Some((&tg_stickerpack).into()),
			stickers
		};
		Ok(stickerpack)
	}
}

impl From<maunium::TgPackInfo> for TgPackInfo {
	fn from(value: maunium::TgPackInfo) -> Self {
		Self {
			name: value.short_name.clone(),
			title: value.short_name
		}
	}
}
impl From<maunium::TgPackRootInfo> for TgPackInfo {
	fn from(value: maunium::TgPackRootInfo) -> Self {
		Self {
			name: value.short_name.clone(),
			title: value.short_name
		}
	}
}

impl From<maunium::StickerPack> for StickerPack {
	fn from(value: maunium::StickerPack) -> Self {
		Self {
			title: value.title,
			id: value.id,
			tg_pack: value.tg_pack.map(|f| f.into()),
			stickers: value.stickers.into_iter().map(|f| f.into()).collect()
		}
	}
}

#[cfg(test)]
mod tests {

	use super::StickerPack;
	use crate::{database::simple_file::FileDatabase, image::AnimationFormat};
	use lottieconv::Rgba;
	use std::env;

	async fn import(pack: &str, animation_format: Option<AnimationFormat>) {
		let matrix_config = crate::matrix::Config {
			homeserver_url: "none".to_owned(),
			user: "none".to_owned(),
			access_token: "none".to_owned()
		};
		let tg_config = crate::tg::Config {
			bot_key: env::var("TG_BOT_KEY").expect("environment variables TG_BOT_KEY is not set")
		};
		StickerPack::import_pack::<FileDatabase>(
			pack,
			None,
			&tg_config,
			true,
			false,
			&matrix_config,
			animation_format.as_ref()
		)
		.await
		.unwrap();
	}

	#[tokio::test]
	#[ignore]
	async fn import_simple() {
		import("LINE_Menhera_chan_ENG", Some(AnimationFormat::Webp)).await;
	}

	#[tokio::test]
	#[ignore]
	async fn import_webp() {
		import("NSanimated", Some(AnimationFormat::Webp)).await;
	}

	#[tokio::test]
	#[ignore]
	async fn import_gif() {
		import(
			"NSanimated",
			Some(AnimationFormat::Gif {
				transparent_color: Rgba {
					r: 0,
					g: 0,
					b: 0,
					a: true
				}
			})
		)
		.await;
	}

	#[tokio::test]
	#[ignore]
	async fn import_none() {
		import("NSanimated", None).await;
	}
}

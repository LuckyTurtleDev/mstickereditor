use crate::database;
use anyhow::anyhow;
use colored::*;
use futures_util::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

use super::sticker::{Metadata, Sticker, StickerInfo, TgInfo, TgPackInfo};
use crate::tg::{self, sticker::Sticker as TgSticker};

use crate::image::AnimationFormat;

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
	//this is ugly
	#[allow(clippy::too_many_arguments)]
	async fn import_stickers<D>(
		i: usize,
		tg_sticker: &TgSticker,
		pb: &ProgressBar,
		tg_config: &tg::Config,
		animation_format: &AnimationFormat,
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
		let image = tg_sticker.download(tg_config)?;
		// convert sticker from lottie to gif if neccessary
		let image = if image.path.ends_with(".tgs") {
			image.convert_if_tgs(animation_format)?
		} else {
			image
		};

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
			let (mxc_url, has_uploded) = image.upload(matrix_config, database)?;
			if !has_uploded {
				pb.println("upload skipped; file with this hash was already uploaded")
			}

			//construct Sticker Struct
			let tg_info = TgInfo {
				id: "unimplemented".to_owned(),
				emoticons: vec![tg_sticker.emoji.to_owned()],
				pack: TgPackInfo {
					id: "unimplemented".to_owned(),
					short_name: tg_stickerpack.name.clone()
				}
			};
			let meta_data = Metadata {
				w: image.width,
				h: image.height,
				size: image.data.len(),
				mimetype
			};
			let info = StickerInfo {
				metadata: meta_data.clone(),
				thumbnail_url: mxc_url.clone(),
				thumbnail_info: meta_data
			};
			sticker = Some(Sticker {
				body: tg_sticker.emoji.clone(),
				url: mxc_url,
				info,
				msgtype: "m.sticker".to_owned(),
				id: format!("tg_file_id_{}", tg_sticker.file_id),
				tg_sticker: tg_info
			});
		}

		pb.println(format!("  finish sticker {:02} {}", i + 1, tg_sticker.emoji));
		pb.inc(1);
		Ok(sticker)
	}

	pub async fn import_pack<D>(
		pack: &str,
		database: Option<&D>,
		tg_config: &tg::Config,
		dryrun: bool,
		save_to_disk: bool,
		matrix_config: &super::Config,
		animation_format: &AnimationFormat
	) -> anyhow::Result<Self>
	where
		D: database::Database + Sync + Send
	{
		let tg_stickerpack = tg::get_stickerpack(tg_config, pack)?;
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
					pb.println(format!("ERROR: {:?}", err).red().to_string());
					pb.println("Stickerpack will not be complete".yellow().to_string());
					None
				}
			})
			.collect();
		pb.finish();

		// save the stickerpack to file
		println!("save stickerpack {} to {}.json", tg_stickerpack.title, tg_stickerpack.name);
		let stickerpack = StickerPack {
			title: tg_stickerpack.title,
			id: format!("tg_name_{}", tg_stickerpack.name),
			tg_pack: TgPack {
				short_name: tg_stickerpack.name,
				hash: "unimplemented".to_owned()
			},
			stickers
		};
		Ok(stickerpack)
	}
}

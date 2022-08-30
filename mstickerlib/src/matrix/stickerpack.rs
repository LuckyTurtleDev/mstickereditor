use crate::database;
use anyhow::{anyhow, bail, Context};
use clap::Parser;
use colored::*;
use flate2::write::GzDecoder;
use generic_array::GenericArray;
use indicatif::{ProgressBar, ProgressStyle};
use libwebp::WebPGetInfo as webp_get_info;
use lottie2gif::Color;
use lottie2webp;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{digest::OutputSizeUser, Digest, Sha512};
use std::{
	collections::BTreeMap,
	fs::{self, File},
	io::{self, BufRead, Write},
	path::Path,
	process::exit
};
use tempfile::NamedTempFile;

use super::sticker::{Metadata, Sticker, StickerInfo, TgInfo, TgPackInfo};
use crate::tg;

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

	fn import_pack<D>(pack: &String, database: Option<D>, tg_config: &tg::Config, opt: &Opt) -> anyhow::Result<()>
	where
		D: database::Database
	{
		let stickerpack = tg::get_stickerpack(tg_config, &pack)?;
		println!("found Telegram stickerpack {}({})", stickerpack.title, stickerpack.name);
		if opt.save {
			fs::create_dir_all(format!("./stickers/{}", stickerpack.name))?;
		}
		let mut database_tree = BTreeMap::<GenericArray<u8, <Sha512 as OutputSizeUser>::OutputSize>, String>::new();

		let database = fs::OpenOptions::new()
			.write(true)
			.append(true)
			.create(true)
			.open(&database_file)
			.with_context(|| format!("WARNING: Failed to open or create database {}", database_file.display()));
		let mut database = match database {
			Ok(value) => Some(value),
			Err(error) => {
				eprintln!("{:?}", error);
				None
			}
		};
		let pb = ProgressBar::new(stickerpack.stickers.len() as u64);
		pb.set_style(
			ProgressStyle::default_bar()
				.template("[{wide_bar:.cyan/blue}] {pos:>3}/{len} {msg}")
				.progress_chars("#> ")
		);
		if stickerpack.is_video {
			pb.println(
				format!(
					"WARNING: sticker pack {} include video stickers. This are current not supported and will be skipped.",
					stickerpack.name
				)
				.yellow()
				.to_string()
			);
		}
		let stickers: Vec<Sticker> = stickerpack
			.stickers
			.par_iter()
			.enumerate()
			.map(|(i, tg_sticker)| {
				if tg_sticker.is_video {
					pb.println(
						format!("    skip Sticker {:02} {},	is a video", i + 1, tg_sticker.emoji)
							.yellow()
							.to_string()
					);
					return Ok(None);
				}
				pb.println(format!("download sticker {:02} {}", i + 1, tg_sticker.emoji));

				// get sticker from telegram
				let sticker_file = tg::get_sticker_file(&config.telegram, &tg_sticker)?;
				let mut sticker_image = sticker_file.download(&config.telegram)?;
				let mut sticker_image_name = sticker_file.get_file_name();

				// convert sticker from lottie to gif if neccessary
				let (width, height) = if sticker_image_name.ends_with(".tgs") {
					//save to image to file
					let mut tmp = NamedTempFile::new()?;
					{
						let mut out = GzDecoder::new(&mut tmp);
						out.write_all(&sticker_image)?;
					}
					tmp.flush()?;
					let animation =
						rlottie::Animation::from_file(tmp.path()).ok_or_else(|| anyhow!("Failed to load sticker"))?;
					let size = animation.size();
					if !opt.noformat {
						pb.println(format!(" convert sticker {:02} {}", i, tg_sticker.emoji));
						sticker_image.clear();
						sticker_image_name.truncate(sticker_image_name.len() - 3);
						match opt.animation_format.unwrap_or(config.sticker.animation_format) {
							AnimationFormat::Gif => {
								lottie2gif::convert(
									animation,
									Color {
										r: config.sticker.transparent_color.r,
										g: config.sticker.transparent_color.g,
										b: config.sticker.transparent_color.b,
										alpha: config.sticker.transparent_color.alpha
									},
									&mut sticker_image
								)?;
								sticker_image_name += "gif";
							},
							AnimationFormat::Webp => {
								sticker_image = match lottie2webp::convert(animation) {
									Ok(value) => value.to_vec(),
									Err(error) => bail!("error converting tgs to webp: {error:?}")
								};
								sticker_image_name += "webp";
							}
						}
					}
					(size.width as u32, size.height as u32)
				} else {
					webp_get_info(&sticker_image)?
				};

				// store file on disk if desired
				if opt.save {
					pb.println(format!("    save sticker {:02} {}", i + 1, tg_sticker.emoji));
					let file_path: &Path = sticker_image_name.as_ref();
					fs::write(
						Path::new(&format!("./stickers/{}", stickerpack.name)).join(file_path.file_name().unwrap()),
						&sticker_image
					)?;
				}

				let mut sticker = None;
				if !opt.noupload {
					let mut hasher = Sha512::new();
					hasher.update(&sticker_image);
					let hash = hasher.finalize();

					let mimetype = format!(
						"image/{}",
						Path::new(&sticker_image_name)
							.extension()
							.ok_or_else(|| anyhow!("ERROR: extracting mimetype from path {}", sticker_image_name))?
							.to_str()
							.ok_or_else(|| anyhow!("ERROR: converting mimetype to string"))?
					);

					let mxc_url = if let Some(value) = database_tree.get(&hash) {
						pb.println(format!(
							"  upload sticker {:02} {} skipped; file with this hash was already uploaded",
							i + 1,
							tg_sticker.emoji
						));
						value.clone()
					} else {
						pb.println(format!("  upload sticker {:02} {}", i + 1, tg_sticker.emoji));
						let url = upload_to_matrix(&config.matrix, sticker_image_name, &sticker_image, &mimetype)?;
						url
					};

					sticker = Some(Sticker {
						file_hash: hash,
						mxc_url,
						file_id: tg_sticker.file_id.clone(),
						emoji: tg_sticker.emoji.clone(),
						width,
						height,
						file_size: sticker_image.len(),
						mimetype
					});
				}

				pb.println(format!("  finish sticker {:02} {}", i + 1, tg_sticker.emoji));
				pb.inc(1);
				Ok(sticker)
			})
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

		// write new entries into the database
		if !opt.noupload {
			if let Some(ref mut db) = database {
				for sticker in &stickers {
					let hash_url = HashUrl {
						hash: sticker.file_hash,
						url: sticker.mxc_url.clone()
					};
					writeln!(db, "{}", serde_json::to_string(&hash_url)?)?;
					// TODO write into database_tree
				}
			}
		}

		// save the stickerpack to file
		if !stickers.is_empty() {
			println!("save stickerpack {} to {}.json", stickerpack.title, stickerpack.name);
			let pack_json = stickerpicker::StickerPack::new(&stickerpack, &stickers);
			fs::write(
				Path::new(&format!("./{}.json", stickerpack.name)),
				serde_json::to_string(&pack_json)?
			)?;
		} else {
			println!("{}", "WARNING: stickerpack is empty. Skip save.".yellow())
		}
		Ok(())
	}
}

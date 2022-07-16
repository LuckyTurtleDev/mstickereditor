use crate::{
	config::{load_config_file, AnimationFormat, Config},
	matrix,
	matrix::upload_to_matrix,
	stickerpicker, tg, DATABASE_FILE, PROJECT_DIRS
};
use anyhow::{anyhow, bail, Context};
use clap::Parser;
use colored::*;
use flate2::write::GzDecoder;
use generic_array::GenericArray;
use indicatif::{ProgressBar, ProgressStyle};
use libwebp::WebPGetInfo as webp_get_info;
use lottieconv::{Animation, Converter};
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

#[derive(Debug, Parser)]
pub struct Opt {
	/// Pack url
	#[clap(required = true)]
	packs: Vec<String>,

	/// Save stickers to disk
	#[clap(short, long)]
	save: bool,

	/// Does not upload the sticker to Matrix
	#[clap(short = 'U', long)]
	noupload: bool,

	/// Do not format the stickers;
	/// The stickers can may not be shown by a matrix client
	#[clap(short = 'F', long)]
	noformat: bool,

	/// format to which the stickers well be converted
	#[clap(long)]
	animation_format: Option<AnimationFormat>
}

type Hash = GenericArray<u8, <Sha512 as OutputSizeUser>::OutputSize>;

#[derive(Debug, Deserialize, Serialize)]
struct HashUrl {
	hash: Hash,
	url: String
}

pub struct Sticker {
	file_hash: Hash,
	pub mxc_url: String,
	pub file_id: String,

	pub emoji: String,
	pub width: u32,
	pub height: u32,
	pub file_size: usize,
	pub mimetype: String
}

pub fn run(mut opt: Opt) -> anyhow::Result<()> {
	let config = load_config_file()?;

	if !opt.noupload {
		matrix::whoami(&config.matrix).expect("Error connecting to Matrix homeserver");
	}
	let mut packs: Vec<String> = Vec::new();
	while let Some(pack) = opt.packs.pop() {
		let mut id = pack.strip_prefix("https://t.me/addstickers/");
		if id.is_none() {
			id = pack.strip_prefix("tg://addstickers?set=");
		};
		match id {
			None => {
				eprintln!("{pack:?} does not look like a Telegram StickerPack");
				exit(1);
			},
			Some(id) => packs.push(id.into())
		};
	}
	for pack in packs {
		import_pack(&pack, &config, &opt)?;
	}
	Ok(())
}

fn import_pack(pack: &String, config: &Config, opt: &Opt) -> anyhow::Result<()> {
	let stickerpack = tg::get_stickerpack(&config.telegram, &pack)?;
	println!("found Telegram stickerpack {}({})", stickerpack.title, stickerpack.name);
	if opt.save {
		fs::create_dir_all(format!("./stickers/{}", stickerpack.name))?;
	}
	let mut database_tree = BTreeMap::<GenericArray<u8, <Sha512 as OutputSizeUser>::OutputSize>, String>::new();
	let database_file = PROJECT_DIRS.data_dir().join(DATABASE_FILE);
	match File::open(&database_file) {
		Ok(file) => {
			let bufreader = std::io::BufReader::new(file);
			for (i, line) in bufreader.lines().enumerate() {
				let hashurl: Result<HashUrl, serde_json::Error> = serde_json::from_str(&line?);
				match hashurl {
					Ok(value) => {
						database_tree.insert(value.hash, value.url);
					},
					Err(error) => eprintln!(
						"Warning: Line {} of Database({}) can not be read: {:?}",
						i + 1,
						database_file.as_path().display(),
						error
					)
				};
			}
		},
		Err(error) if error.kind() == io::ErrorKind::NotFound => {
			print!("database not found, creating a new one");
		},
		Err(error) => {
			return Err(error.into());
		}
	};
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
				let animation = Animation::from_file(tmp.path()).ok_or_else(|| anyhow!("Failed to load sticker"))?;
				let size = animation.size();
				if !opt.noformat {
					pb.println(format!(" convert sticker {:02} {}", i, tg_sticker.emoji));
					sticker_image.clear();
					sticker_image_name.truncate(sticker_image_name.len() - 3);
					let converter = Converter::new(animation);
					match opt.animation_format.unwrap_or(config.sticker.animation_format) {
						AnimationFormat::Gif => {
							converter
								.gif(config.sticker.transparent_color, &mut sticker_image)?
								.convert()?;
							sticker_image_name += "gif";
						},
						AnimationFormat::Webp => {
							sticker_image = match converter.webp().and_then(Converter::convert) {
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

#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]

use anyhow::{anyhow, Context};
use directories::ProjectDirs;
use flate2::write::GzDecoder;
use generic_array::GenericArray;
use indicatif::{ProgressBar, ProgressStyle};
use lottie2gif::{Animation, Color};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::{
	collections::BTreeMap,
	fs::{self, File},
	io::{self, BufRead, Write},
	path::Path,
	process::exit
};
use structopt::StructOpt;
use tempfile::NamedTempFile;

mod config;
use config::*;

mod matrix;
use matrix::upload_to_matrix;

mod tg;

const CONFIG_FILE: &str = "config.toml";
const DATABASE_FILE: &str = "uploads";
static PROJECT_DIRS: Lazy<ProjectDirs> =
	Lazy::new(|| ProjectDirs::from("de", "lukas1818", "mstickereditor").expect("failed to get project dirs"));

#[derive(Debug, StructOpt)]
struct OptImport {
	/// Pack url
	pack: String,

	/// Save sticker local
	#[structopt(short, long)]
	download: bool,

	/// Does not upload the sticker to Matrix
	#[structopt(short = "U", long)]
	noupload: bool,

	/// Do not format the stickers;
	/// The stickers can may not be shown by a matrix client
	#[structopt(short = "F", long)]
	noformat: bool
}

#[derive(Debug, StructOpt)]
enum Opt {
	/// import Stickerpack from telegram
	Import(OptImport)
}

#[derive(Debug, Deserialize, Serialize)]
struct HashUrl {
	hash: GenericArray<u8, <Sha512 as Digest>::OutputSize>,
	url: String
}

fn import(opt: OptImport) -> anyhow::Result<()> {
	let config: Config = toml::from_str(&fs::read_to_string(PROJECT_DIRS.config_dir().join(CONFIG_FILE)).with_context(
		|| {
			format!(
				"Failed to open {}",
				PROJECT_DIRS.config_dir().join(CONFIG_FILE).to_str().unwrap()
			)
		}
	)?)?;

	let stickerpack = tg::get_stickerpack(&config.telegram, &opt.pack)?;
	println!("found Telegram stickerpack {}({})", stickerpack.title, stickerpack.name);
	if opt.download {
		fs::create_dir_all(format!("./stickers/{}", stickerpack.name))?;
	}
	let mut database_tree = BTreeMap::<GenericArray<u8, <Sha512 as Digest>::OutputSize>, String>::new();
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
	let hashes: Vec<HashUrl> = stickerpack
		.stickers
		.par_iter()
		.enumerate()
		.map(|(i, sticker)| {
			pb.println(format!("download sticker {:02} {}", i + 1, sticker.emoji));

			// get sticker from telegram
			let mut sticker_file = tg::get_sticker_file(&config.telegram, sticker)?;
			let mut sticker_image = sticker_file.download(&config.telegram)?;

			// convert sticker from lottie to gif if neccessary
			if !opt.noformat && sticker_file.file_path.ends_with(".tgs") {
				pb.println(format!(" convert sticker {:02} {}", i, sticker.emoji));
				let mut tmp = NamedTempFile::new()?;
				{
					let mut out = GzDecoder::new(&mut tmp);
					out.write_all(&sticker_image)?;
				}
				tmp.flush()?;
				let sticker = Animation::from_file(tmp.path()).ok_or_else(|| anyhow!("Failed to load sticker"))?;
				sticker_image.clear();
				lottie2gif::convert(
					sticker,
					Color {
						r: 0,
						g: 0,
						b: 0,
						alpha: true
					},
					&mut sticker_image
				)?;
				sticker_file.file_path += ".gif";
			}

			// store file on disk if desired
			if opt.download {
				pb.println(format!("    save sticker {:02} {}", i + 1, sticker.emoji));
				let file_path: &Path = sticker_file.file_path.as_ref();
				fs::write(
					Path::new(&format!("./stickers/{}", stickerpack.name)).join(file_path.file_name().unwrap()),
					&sticker_image
				)?;
			}

			let mut hashurl = None;
			if !opt.noupload && database.is_some() {
				let mut hasher = Sha512::new();
				hasher.update(&sticker_image);
				let hash = hasher.finalize();

				let url = if let Some(value) = database_tree.get(&hash) {
					value.clone()
				} else {
					pb.println(format!("  upload sticker {:02} {}", i + 1, sticker.emoji));
					let url = upload_to_matrix(&config.matrix, sticker_file.file_path, sticker_image, None)?;
					hashurl = Some(HashUrl { hash, url: url.clone() });
					url
				};
				// TODO set the url somewhere???
			}

			pb.println(format!("  finish sticker {:02} {}", i + 1, sticker.emoji));
			pb.inc(1);
			Ok(hashurl)
		})
		.filter_map(|res: anyhow::Result<Option<HashUrl>>| match res {
			Ok(hash) => hash,
			Err(err) => {
				eprintln!("ERROR: {:?}", err);
				None
			}
		})
		.collect();

	// write new entries into the database
	for hash in hashes {
		let db = database.as_mut().unwrap();
		writeln!(db, "{}", serde_json::to_string(&hash)?)?;
		// TODO write into database_tree
	}

	pb.finish();
	println!();
	if database.is_some() {
		database.unwrap().sync_data()?;
	}
	Ok(())
}

fn main() {
	let data_dir = PROJECT_DIRS.data_dir();
	if let Err(err) = fs::create_dir_all(&data_dir) {
		eprintln!("Cannot create data dir {}: {}", data_dir.display(), err);
		exit(1);
	}
	let result = match Opt::from_args() {
		Opt::Import(opt) => import(opt)
	};
	if let Err(error) = result {
		eprintln!("{:?}", error);
		exit(1);
	}
}

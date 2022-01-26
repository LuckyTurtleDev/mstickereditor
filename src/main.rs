#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]

use anyhow::{anyhow, Context};
use clap::{IntoApp, Parser};
use directories::ProjectDirs;
use flate2::write::GzDecoder;
use generic_array::GenericArray;
use indicatif::{ProgressBar, ProgressStyle};
use libwebp::WebPGetInfo as webp_get_info;
use lottie2gif::{Animation, Color};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::{
	collections::BTreeMap,
	fs::{self, File},
	io::{self, stdout, BufRead, Write},
	path::Path,
	process::exit
};
use tempfile::NamedTempFile;

mod config;
use config::*;

mod matrix;
use matrix::upload_to_matrix;

mod stickerpicker;

mod tg;

const CONFIG_FILE: &str = "config.toml";
const DATABASE_FILE: &str = "uploads";
const CARGO_PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
static PROJECT_DIRS: Lazy<ProjectDirs> =
	Lazy::new(|| ProjectDirs::from("de", "lukas1818", CARGO_PKG_NAME).expect("failed to get project dirs"));

#[derive(Debug, Parser)]
struct OptImport {
	/// Pack url
	pack: String,

	/// Save stickers to disk
	#[clap(short, long)]
	save: bool,

	/// Does not upload the sticker to Matrix
	#[clap(short = 'U', long)]
	noupload: bool,

	/// Do not format the stickers;
	/// The stickers can may not be shown by a matrix client
	#[clap(short = 'F', long)]
	noformat: bool
}

#[derive(Debug, Parser)]
struct OptSetWidget {
	/// The url of your sticker picker
	widgeturl: String
}

#[derive(Debug, Parser)]
struct OptShellCompletion {
	shell: clap_complete::Shell
}

#[derive(Debug, Parser)]
enum Opt {
	/// import Stickerpack from telegram
	Import(OptImport),

	/// enable a custom sticker picker widget in a supported Matirx client
	SetWidget(OptSetWidget),

	/// print shell completion for a given shell
	ShellCompletion(OptShellCompletion)
}

type Hash = GenericArray<u8, <Sha512 as Digest>::OutputSize>;

#[derive(Debug, Deserialize, Serialize)]
struct HashUrl {
	hash: Hash,
	url: String
}

struct Sticker {
	file_hash: Hash,
	mxc_url: String,
	file_id: String,

	emoji: String,
	width: u32,
	height: u32,
	file_size: usize,
	mimetype: String
}

fn print_shell_completion(opt: OptShellCompletion) -> anyhow::Result<()> {
	clap_complete::generate(opt.shell, &mut Opt::into_app(), CARGO_PKG_NAME, &mut stdout());
	Ok(())
}

fn load_config_file() -> anyhow::Result<Config> {
	let config: Config = toml::from_str(&fs::read_to_string(PROJECT_DIRS.config_dir().join(CONFIG_FILE)).with_context(
		|| {
			format!(
				"Failed to open {}",
				PROJECT_DIRS.config_dir().join(CONFIG_FILE).to_str().unwrap()
			)
		}
	)?)?;
	Ok(config)
}

fn set_widget(opt: OptSetWidget) -> anyhow::Result<()> {
	let config = load_config_file()?;
	matrix::set_widget(&config.matrix, config.matrix.user.clone(), opt.widgeturl).expect("Error setting widget");
	Ok(())
}

fn import(opt: OptImport) -> anyhow::Result<()> {
	let config = load_config_file()?;

	if !opt.noupload {
		matrix::whoami(&config.matrix).expect("Error connecting to Matrix homeserver");
	}
	let stickerpack = tg::get_stickerpack(&config.telegram, &opt.pack)?;
	println!("found Telegram stickerpack {}({})", stickerpack.title, stickerpack.name);
	if opt.save {
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
	let stickers: Vec<Sticker> = stickerpack
		.stickers
		.par_iter()
		.enumerate()
		.map(|(i, tg_sticker)| {
			pb.println(format!("download sticker {:02} {}", i + 1, tg_sticker.emoji));

			// get sticker from telegram
			let mut sticker_file = tg::get_sticker_file(&config.telegram, &tg_sticker)?;
			let mut sticker_image = sticker_file.download(&config.telegram)?;

			// convert sticker from lottie to gif if neccessary
			let (width, height) = if sticker_file.file_path.ends_with(".tgs") {
				let mut tmp = NamedTempFile::new()?;
				{
					let mut out = GzDecoder::new(&mut tmp);
					out.write_all(&sticker_image)?;
				}
				tmp.flush()?;
				let sticker = Animation::from_file(tmp.path()).ok_or_else(|| anyhow!("Failed to load sticker"))?;
				let size = sticker.size();
				if !opt.noformat {
					pb.println(format!(" convert sticker {:02} {}", i, tg_sticker.emoji));
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
				(size.width as u32, size.height as u32)
			} else {
				webp_get_info(&sticker_image)?
			};

			// store file on disk if desired
			if opt.save {
				pb.println(format!("    save sticker {:02} {}", i + 1, tg_sticker.emoji));
				let file_path: &Path = sticker_file.file_path.as_ref();
				fs::write(
					Path::new(&format!("./stickers/{}", stickerpack.name)).join(file_path.file_name().unwrap()),
					&sticker_image
				)?;
			}

			let mut sticker = None;
			if !opt.noupload && database.is_some() {
				let mut hasher = Sha512::new();
				hasher.update(&sticker_image);
				let hash = hasher.finalize();

				let mimetype = format!(
					"image/{}",
					Path::new(&sticker_file.file_path)
						.extension()
						.ok_or_else(|| anyhow!("ERROR: extracting mimetype from path {}", sticker_file.file_path))?
						.to_str()
						.ok_or_else(|| anyhow!("ERROR: converting mimetype to string"))?
				);

				let mxc_url = if let Some(value) = database_tree.get(&hash) {
					value.clone()
				} else {
					pb.println(format!("  upload sticker {:02} {}", i + 1, tg_sticker.emoji));
					let url = upload_to_matrix(&config.matrix, sticker_file.file_path, &sticker_image, &mimetype)?;
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
				pb.println(format!("ERROR: {:?}", err));
				None
			}
		})
		.collect();
	pb.finish();

	// write new entries into the database
	for sticker in &stickers {
		let db = database.as_mut().unwrap();
		let hash_url = HashUrl {
			hash: sticker.file_hash,
			url: sticker.mxc_url.clone()
		};
		writeln!(db, "{}", serde_json::to_string(&hash_url)?)?;
		// TODO write into database_tree
	}
	if database.is_some() {
		database.unwrap().sync_data()?;
	}

	// save the stickerpack to file
	if !stickers.is_empty() {
		println!("save stickerpack {} to {}.json", stickerpack.title, stickerpack.name);
		let pack_json = stickerpicker::StickerPack::new(&stickerpack, &stickers);
		fs::write(
			Path::new(&format!("./{}.json", stickerpack.name)),
			serde_json::to_string(&pack_json)?
		)?;
	}
	Ok(())
}

fn main() {
	let data_dir = PROJECT_DIRS.data_dir();
	if let Err(err) = fs::create_dir_all(&data_dir) {
		eprintln!("Cannot create data dir {}: {}", data_dir.display(), err);
		exit(1);
	}
	let result = match Opt::parse() {
		Opt::Import(opt) => import(opt),
		Opt::SetWidget(opt) => set_widget(opt),
		Opt::ShellCompletion(opt) => print_shell_completion(opt)
	};
	if let Err(error) = result {
		eprintln!("{:?}", error);
		exit(1);
	}
}

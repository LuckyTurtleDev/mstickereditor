use anyhow::{anyhow, bail, Context};
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

#[derive(Deserialize)]
struct Matrix {
	homeserver_url: String,
	user: String,
	access_token: String
}

#[derive(Deserialize)]
struct TTelegram {
	bot_key: String
}

#[derive(Deserialize)]
struct TomlFile {
	telegram: TTelegram,
	matrix: Matrix
}

// TODO rename to Status
#[derive(Debug, Deserialize)]
struct TJsonState {
	ok: bool,

	error_code: Option<u32>,
	description: Option<String>
}

#[derive(Debug)]
struct MSticker {
	filename: String,
	mimetype: String,
	uri: String
}

#[derive(Debug, Deserialize)]
struct TJsonSticker {
	emoji: String,
	file_id: String
}

#[derive(Debug, Deserialize)]
struct TJsonStickerPack {
	name: String,
	title: String,
	is_animated: bool,
	stickers: Vec<TJsonSticker>
}

#[derive(Debug, Deserialize)]
struct TJsonFile {
	file_path: String
}

#[derive(Debug, Deserialize, Serialize)]
struct HashUrl {
	hash: GenericArray<u8, <Sha512 as Digest>::OutputSize>,
	url: String
}

#[derive(Debug, Deserialize)]
struct MatrixError {
	errcode: String,
	error: String,
	_retry_after_ms: Option<u32>
}

fn check_telegram_resp(mut resp: serde_json::Value) -> anyhow::Result<serde_json::Value> {
	let resp_state: TJsonState = serde_json::from_value(resp.clone())?;
	if !resp_state.ok {
		bail!(
			"Telegram request was not successful: {} {}",
			resp_state.error_code.unwrap(),
			resp_state.description.unwrap()
		);
	}
	let resp = resp.get_mut("result").ok_or_else(|| anyhow!("Missing 'result'"))?.take();
	Ok(serde_json::from_value(resp)?)
}

fn upload_to_matrix(
	matrix: &Matrix,
	filename: String,
	image_data: Vec<u8>,
	mimetype: Option<String>
) -> anyhow::Result<String> {
	let url = format!("{}/_matrix/media/r0/upload", matrix.homeserver_url);
	let mimetype = match mimetype {
		Some(value) => value,
		None => format!(
			"image/{}",
			Path::new(&filename)
				.extension()
				.ok_or_else(|| anyhow!("ERROR: extracting mimetype from path {}", filename))?
				.to_str()
				.ok_or_else(|| anyhow!("ERROR: converting mimetype to string"))?
		)
	};
	let answer = attohttpc::put(url)
		.params([("access_token", &matrix.access_token), ("filename", &filename)])
		.header("Content-Type", mimetype)
		.bytes(image_data)
		.send()?; //TODO
	if answer.status() != 200 {
		let status = answer.status();
		let error: Result<String, anyhow::Error> = (|| {
			let matrix_error: MatrixError = serde_json::from_value(answer.json()?)?;
			Ok(format!(": {} {}", matrix_error.errcode, matrix_error.error))
		})();
		bail!(
			"failed to upload sticker {}: {}{}",
			filename,
			status,
			error.unwrap_or(String::new())
		);
	}
	// TODO return the real url here
	Ok(String::new())
}

fn import(opt: OptImport) -> anyhow::Result<()> {
	let toml_file: TomlFile = toml::from_str(
		&fs::read_to_string(PROJECT_DIRS.config_dir().join(CONFIG_FILE)).with_context(|| {
			format!(
				"Failed to open {}",
				PROJECT_DIRS.config_dir().join(CONFIG_FILE).to_str().unwrap()
			)
		})?
	)?;
	let telegram_api_base_url = format!("https://api.telegram.org/bot{}", toml_file.telegram.bot_key);
	check_telegram_resp(attohttpc::get(format!("{}/getMe", telegram_api_base_url)).send()?.json()?)?;

	let stickerpack: TJsonStickerPack = serde_json::from_value(check_telegram_resp(
		attohttpc::get(format!("{}/getStickerSet", telegram_api_base_url))
			.param("name", &opt.pack)
			.send()?
			.json()?
	)?)?;
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

			// get sticker metadata from telegram
			let mut sticker_file: TJsonFile = serde_json::from_value(check_telegram_resp(
				attohttpc::get(format!("{}/getFile", telegram_api_base_url))
					.param("file_id", &sticker.file_id)
					.send()?
					.json()?
			)?)?;

			// get sticker from telegram
			let mut sticker_image = attohttpc::get(format!(
				"https://api.telegram.org/file/bot{}/{}",
				toml_file.telegram.bot_key, sticker_file.file_path
			))
			.send()?
			.bytes()?;

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
					let url = upload_to_matrix(&toml_file.matrix, sticker_file.file_path, sticker_image, None)?;
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

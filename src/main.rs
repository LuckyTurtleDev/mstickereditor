use anyhow::{anyhow, bail, Context};
use directories::ProjectDirs;
use generic_array::GenericArray;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::{collections::BTreeMap, fs, fs::File, io, io::Write, path::Path, process::exit};
use structopt::StructOpt;

const CONFIG_FILE: &str = "config.toml";
const DATABASE_FILE: &str = "uploads";
static PROJECT_DIRS: Lazy<ProjectDirs> =
	Lazy::new(|| ProjectDirs::from("de", "lukas1818", "mstickereditor").expect("failed to get project dirs"));

#[derive(Debug, StructOpt)]
struct OptImport {
	///Pack url
	pack: String,

	///Save sticker local
	#[structopt(short, long)]
	download: bool,

	///Does not upload the sticker to Matrix
	#[structopt(short = "U", long)]
	noupload: bool,

	///Do not format the stickers;
	///The stickers can may not be shown by a matrix client
	#[structopt(short = "F", long)]
	noformat: bool,
}

#[derive(Debug, StructOpt)]
enum Opt {
	///import Stickerpack from telegram
	Import(OptImport),
}

#[derive(Deserialize)]
struct Matrix {
	homeserver_url: String,
	user: String,
	access_token: String,
}

#[derive(Deserialize)]
struct TTelegram {
	bot_key: String,
}

#[derive(Deserialize)]
struct TomlFile {
	telegram: TTelegram,
	matrix: Matrix,
}

// TODO rename to Status
#[derive(Debug, Deserialize)]
struct TJsonSatet {
	ok: bool,

	error_code: Option<u32>,
	description: Option<String>,
}

#[derive(Debug)]
struct MSticker {
	filename: String,
	mimetype: String,
	uri: String,
}

#[derive(Debug, Deserialize)]
struct TJsonSticker {
	emoji: String,
	file_id: String,
}

#[derive(Debug, Deserialize)]
struct TJsonStickerPack {
	name: String,
	title: String,
	is_animated: bool,
	stickers: Vec<TJsonSticker>,
}

#[derive(Debug, Deserialize)]
struct TJsonFile {
	file_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct HashUrl {
	hash: GenericArray<u8, <Sha512 as Digest>::OutputSize>,
	url: String,
}

fn check_telegram_resp(mut resp: serde_json::Value) -> anyhow::Result<serde_json::Value> {
	let resp_state: TJsonSatet = serde_json::from_value(resp.clone())?;
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

fn upload_to_matrix(matrix: &Matrix, filename: String, image_data: Vec<u8>, mimetype: Option<String>) -> anyhow::Result<()> {
	let url = format!("{}/_matrix/media/r0/upload", matrix.homeserver_url);
	let mimetype = match mimetype {
		Some(value) => value,
		None => format!(
			"image/{}",
			Path::new(&filename)
				.extension()
				.ok_or_else(|| anyhow!("ERROR: extrcating mimetype from path {}", filename))?
				.to_str()
				.ok_or_else(|| anyhow!("ERROR: converting mimetype to string"))?
		),
	};
	attohttpc::put(url)
		.params([("access_token", &matrix.access_token), ("filename", &filename)])
		.header("Content-Type", mimetype)
		.bytes(image_data)
		.send(); //TODO
	Ok(())
}

fn import(opt: OptImport) -> anyhow::Result<()> {
	let toml_file: TomlFile = toml::from_str(
		&fs::read_to_string(PROJECT_DIRS.config_dir().join(CONFIG_FILE)).with_context(|| {
			format!(
				"Failed to open {}",
				PROJECT_DIRS.config_dir().join(CONFIG_FILE).to_str().unwrap()
			)
		})?,
	)?;
	let telegram_api_base_url = format!("https://api.telegram.org/bot{}", toml_file.telegram.bot_key);
	check_telegram_resp(attohttpc::get(format!("{}/getMe", telegram_api_base_url)).send()?.json()?)?;

	let stickerpack: TJsonStickerPack = serde_json::from_value(check_telegram_resp(
		attohttpc::get(format!("{}/getStickerSet", telegram_api_base_url))
			.param("name", opt.pack)
			.send()?
			.json()?,
	)?)?;
	println!("found Telegram stickerpack {}({})", stickerpack.title, stickerpack.name);
	if opt.download {
		fs::create_dir_all(format!("./stickers/{}", stickerpack.name))?;
	}
	let mut database_tree = BTreeMap::<GenericArray<u8, <Sha512 as Digest>::OutputSize>, String>::new();
	let database_file = PROJECT_DIRS.data_dir().join(DATABASE_FILE);
	match File::open(&database_file) {
		Ok(file) => {
			use std::io::BufRead;
			let bufreader = std::io::BufReader::new(file);
			for (i, line) in bufreader.lines().enumerate() {
				let hashurl: Result<HashUrl, serde_json::Error> = serde_json::from_str(&line?);
				match hashurl {
					Ok(value) => {
						database_tree.insert(value.hash, value.url);
					}
					Err(error) => eprintln!(
						"Warning: Line {} of Database({}) can not be read: {:?}",
						i + 1,
						database_file.as_path().display(),
						error
					),
				};
			}
		}
		Err(error) if error.kind() == io::ErrorKind::NotFound => {
			print!("database not found, creating a new one");
		}
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
	let database = match database {
		Ok(value) => Some(value),
		Err(error) => {
			eprintln!("{:?}", error);
			None
		}
	};
	for (i, sticker) in stickerpack.stickers.iter().enumerate() {
		print!(
			"   download Sticker [{:02}/{:02}] {}   \r",
			i + 1,
			stickerpack.stickers.len(),
			sticker.emoji
		);
		io::stdout().flush()?;
		let sticker_file: TJsonFile = serde_json::from_value(check_telegram_resp(
			attohttpc::get(format!("{}/getFile", telegram_api_base_url))
				.param("file_id", &sticker.file_id)
				.send()?
				.json()?,
		)?)?;
		let sticker_image = attohttpc::get(format!(
			"https://api.telegram.org/file/bot{}/{}",
			toml_file.telegram.bot_key, sticker_file.file_path
		))
		.send()?
		.bytes()?;
		if !opt.noformat {
			print!("    convert Sticker\r");
			io::stdout().flush()?;
			//todo
		}
		if opt.download {
			print!("       save Sticker\r");
			io::stdout().flush()?;
			fs::write(
				Path::new(&format!("./stickers/{}", stickerpack.name))
					.join(std::path::Path::new(&sticker_file.file_path).file_name().unwrap()),
				&sticker_image,
			)?;
		}
		if !opt.noupload && database.is_some() {
			let mut hasher = Sha512::new();
			hasher.update(&sticker_image);
			let mut hashurl = HashUrl {
				hash: hasher.finalize(),
				url: "TODO:matirx_upload_url".to_owned(),
			};
			match database_tree.get(&hashurl.hash) {
				Some(value) => hashurl.url = value.clone(),
				None => {
					print!("     upload Sticker\r");
					io::stdout().flush()?;
					upload_to_matrix(&toml_file.matrix, sticker_file.file_path, sticker_image, None)?;
					database
						.as_ref()
						.unwrap()
						.write_all(format!("{}\n", serde_json::to_string(&hashurl)?).as_bytes())?;
					database_tree.insert(hashurl.hash, hashurl.url);
				}
			}
		}
	}
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
		Opt::Import(opt) => import(opt),
	};
	if let Err(error) = result {
		eprintln!("{:?}", error);
		exit(1);
	}
}

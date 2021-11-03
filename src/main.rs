use anyhow::anyhow;
use anyhow::Context;
use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use structopt::StructOpt;

const CONFIG_FILE: &str = "config.toml";
const DATABASE_FILE: &str = "uploads";
static PROJECT_DIRS: once_cell::sync::Lazy<directories::ProjectDirs> = once_cell::sync::Lazy::new(|| {
	directories::ProjectDirs::from("de", "lukas1818", "mstickereditor").expect("failde to get project dirs")
});

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
struct TMatrix {
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
	matrix: TMatrix,
}

#[derive(Debug, Deserialize)]
struct TJsonSatet {
	ok: bool,

	error_code: Option<u32>,
	description: Option<String>,
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

fn check_telegram_resp(mut resp: serde_json::Value) -> anyhow::Result<serde_json::Value> {
	let resp_state: TJsonSatet = serde_json::from_value(resp.clone())?;
	if !resp_state.ok {
		anyhow::bail!(
			"Telegram request was not successful: {} {}",
			resp_state.error_code.unwrap(),
			resp_state.description.unwrap()
		)
	}
	let resp = resp.get_mut("result").ok_or_else(|| anyhow!("Missing 'result'"))?.take();
	Ok(serde_json::from_value(resp)?)
}

fn upload_to_matrix(_sticker_image: &Vec<u8>) -> anyhow::Result<()> {
	Ok(())
}

fn import(opt: OptImport) -> anyhow::Result<()> {
	let toml_file: TomlFile = toml::from_str(&fs::read_to_string(PROJECT_DIRS.config_dir().join(CONFIG_FILE)).context(
		format!(
			"Failed to open {}",
			PROJECT_DIRS.config_dir().join(CONFIG_FILE).to_str().unwrap()
		),
	)?)?;
	let telegram_api_base_url: String = format!("https://api.telegram.org/bot{}", toml_file.telegram.bot_key);
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
	match File::open(PROJECT_DIRS.data_dir().join(DATABASE_FILE)) {
		Ok(file) => {
			use std::io::BufRead;
			let bufreader = std::io::BufReader::new(file);
			for line in bufreader.lines() {
				println!("{}", line?);
			}
			Ok(())
		}
		Err(error) => {
			if error.kind() == std::io::ErrorKind::NotFound {
				print!("databes not found, creat new one");
				Ok(())
			} else {
				Err(error)
			}
		}
	}?;
	let database = match std::fs::OpenOptions::new()
		.write(true)
		.append(true)
		.create(true)
		.open(PROJECT_DIRS.data_dir().join(DATABASE_FILE))
		.context(format!(
			"WARNING: Failed to open or create database {}",
			PROJECT_DIRS.data_dir().join(DATABASE_FILE).to_str().unwrap()
		)) {
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
		std::io::stdout().flush()?;
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
		if opt.download {
			fs::write(
				std::path::Path::new(&format!("./stickers/{}", stickerpack.name))
					.join(std::path::Path::new(&sticker_file.file_path).file_name().unwrap()),
				&sticker_image,
			)?;
		}
		if !opt.noupload && database.is_some() {
			print!("     upload Sticker\r");
			std::io::stdout().flush()?;
			let image_checksum = adler::adler32_slice(&sticker_image);
			upload_to_matrix(&sticker_image)?;
			database
				.as_ref()
				.unwrap()
				.write_all(format!("{:010} TODO:matirx_upload_url \n", image_checksum).as_bytes())?;
		}
	}
	println!("");
	if database.is_some() {
		database.unwrap().sync_data()?
	}
	Ok(())
}

fn main() {
	std::fs::create_dir_all(PROJECT_DIRS.data_dir()).expect(&format!(
		"can not create data_dir: {}",
		PROJECT_DIRS.data_dir().to_str().unwrap()
	));
	let result = match Opt::from_args() {
		Opt::Import(opt) => import(opt),
	};
	match result {
		Ok(_) => (),
		Err(error) => {
			eprintln!("{:?}", error);
			exit(1)
		}
	};
}

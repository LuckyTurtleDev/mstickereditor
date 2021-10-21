use anyhow::Context;
use serde::Deserialize;
use std::fs;
use std::process::exit;
use structopt::StructOpt;

const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, StructOpt)]
struct OptImport {
	///pack url
	pack: String,

	///show debug messages
	#[structopt(short, long)]
	debug: bool,
}

#[derive(Debug, StructOpt)]
enum Opt {
	///import Stickerpack from telegram
	Import(OptImport),
}

#[derive(Deserialize)]
struct TomlFile {
	telegram_bot_key: String,
}

#[derive(Debug, Deserialize)]
struct JsonGetMe {
	ok: bool,

	error_code: Option<u32>,
	description: Option<String>
}

fn import(opt: OptImport) -> anyhow::Result<()> {
	let toml_file: TomlFile =
		toml::from_str(&fs::read_to_string(CONFIG_FILE).context(format!("Failed to open {}", CONFIG_FILE))?)?;
	let telegram_api_base_url: String = format!("https://api.telegram.org/bot{}", toml_file.telegram_bot_key);
	let resp: JsonGetMe = attohttpc::get(dbg!(format!("{}/getMe", telegram_api_base_url)))
		.send()?
		.json()?;
	println!("{:?}", resp);
	if !resp.ok {
		anyhow::bail!(
			"Request was not successful: {} {}",
			resp.error_code.unwrap(),
			resp.description.unwrap()
		);
	}
	Ok(())
}

fn main() {
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

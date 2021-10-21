use serde::Deserialize;
use std::fs;
use structopt::StructOpt;
use std::process::exit;

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

fn import(opt: OptImport) {
	let toml_file: TomlFile =
		match toml::from_str(&fs::read_to_string(CONFIG_FILE).expect(&format!("Failed to open file {}", CONFIG_FILE))) {
			Ok(value) => value,
			Err(error) => { eprintln!("{}", error);
			exit(1)},
		};
}

fn main() {
	match Opt::from_args() {
		Opt::Import(opt) => import(opt),
	}
	println!("Hello, world!");
}

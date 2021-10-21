use serde::Deserialize;
use std::fs;
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

fn import(opt: OptImport) {
	let toml_file: TomlFile =
		toml::from_str(&fs::read_to_string(CONFIG_FILE).expect(&format!("Failed to open file {}", CONFIG_FILE)))
			.expect(&format!("Invalid Syntax of {}", CONFIG_FILE));
}

fn main() {
	match Opt::from_args() {
		Opt::Import(opt) => import(opt),
	}
	println!("Hello, world!");
}

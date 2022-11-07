#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]

use clap::Parser;
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use std::{fs, process::exit};
use sub_commands::*;

mod config;
mod matrix;
mod stickerpicker;
mod sub_commands;
mod tg;

const CONFIG_FILE: &str = "config.toml";
const DATABASE_FILE: &str = "uploads";
const CARGO_PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
static PROJECT_DIRS: Lazy<ProjectDirs> =
	Lazy::new(|| ProjectDirs::from("dev", "luckyturtle", CARGO_PKG_NAME).expect("failed to get project dirs"));

pub fn load_config_file() -> anyhow::Result<Config> {
	let config: Config = toml::from_str(&fs::read_to_string(PROJECT_DIRS.config_dir().join(CONFIG_FILE)).with_context(
		|| {
			format!(
				"Failed to open {}",
				PROJECT_DIRS.config_dir().join(CONFIG_FILE).to_str().unwrap()
			)
		},
	)?)?;
	Ok(config)
}

#[derive(Deserialize)]
pub struct Config {
	pub telegram: TelegramConfig,
	pub matrix: MatrixConfig,
	#[serde(default)]
	pub sticker: Sticker
}

#[derive(Debug, Parser)]
enum Opt {
	/// import Stickerpack from telegram
	Import(import::Opt),

	/// enable a custom sticker picker widget in a supported Matirx client
	SetWidget(set_widget::Opt),

	/// print shell completion for a given shell
	ShellCompletion(print_shell_completion::Opt),

	/// create the `index.json` from the local stickerpacks for maunium/stickerpicker.
	/// not need for msrd0/docker-stickerpicker (do not upload a `index.json` to the s3 bucket!)
	CreateIndex(create_index::Opt),
}

fn main() {
	let data_dir = PROJECT_DIRS.data_dir();
	if let Err(err) = fs::create_dir_all(&data_dir) {
		eprintln!("Cannot create data dir {}: {}", data_dir.display(), err);
		exit(1);
	}
	let result = match Opt::parse() {
		Opt::Import(opt) => import::run(opt),
		Opt::SetWidget(opt) => set_widget::run(opt),
		Opt::ShellCompletion(opt) => print_shell_completion::run(opt),
		Opt::CreateIndex(opt) => create_index::run(opt),
	};
	if let Err(error) = result {
		eprintln!("{:?}", error);
		exit(1);
	}
}

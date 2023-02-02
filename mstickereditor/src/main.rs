#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]

use anyhow::Context;
use clap::Parser;
use directories::ProjectDirs;
use mstickerlib::{image::AnimationFormat, matrix, tg};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{fs, path::PathBuf, process::exit};

mod sub_commands;
use sub_commands::*;

const CONFIG_FILE: &str = "config.toml";
const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
static PROJECT_DIRS: Lazy<ProjectDirs> =
	Lazy::new(|| ProjectDirs::from("dev", "luckyturtle", CARGO_PKG_NAME).expect("failed to get project dirs"));
static DATABASE_FILE: Lazy<PathBuf> = Lazy::new(|| PROJECT_DIRS.data_dir().join("uploads"));

pub fn new_current_thread_runtime() -> Result<tokio::runtime::Runtime, std::io::Error> {
	tokio::runtime::Builder::new_current_thread()
		.worker_threads(1)
		.enable_all()
		.build()
}

pub fn load_config_file() -> anyhow::Result<Config> {
	let config: Config = toml::from_str(&fs::read_to_string(PROJECT_DIRS.config_dir().join(CONFIG_FILE)).with_context(
		|| {
			format!(
				"Failed to open config file {:?}",
				PROJECT_DIRS.config_dir().join(CONFIG_FILE).to_string_lossy()
			)
		}
	)?)
	.with_context(|| {
		format!(
			"Failed to prase config file {:?}",
			PROJECT_DIRS.config_dir().join(CONFIG_FILE).to_string_lossy()
		)
	})?;
	Ok(config)
}

#[derive(Deserialize)]
pub struct Config {
	pub telegram: tg::Config,
	pub matrix: matrix::Config,
	#[serde(default)]
	pub sticker: AnimationFormat
}

#[derive(Debug, Parser)]
enum Opt {
	/// import Stickerpack from telegram
	Import(import::Opt),
	/// enable a custom sticker picker widget in a supported Matirx client
	/// ⚠️_Warning: make sure you have send a sticker (since creating your matrix account) using the Element stickerpicker, before excuting this subcommand or the widget will not work.
	SetWidget(set_widget::Opt),
	/// print shell completion for a given shell
	ShellCompletion(print_shell_completion::Opt),
	/// create the `index.json` from the local stickerpacks for maunium/stickerpicker.
	/// not need for msrd0/docker-stickerpicker (do not upload a `index.json` to the s3 bucket!)
	CreateIndex(create_index::Opt)
}

fn main() {
	let data_dir = PROJECT_DIRS.data_dir();
	if let Err(err) = fs::create_dir_all(data_dir) {
		eprintln!("Cannot create data dir {}: {}", data_dir.display(), err);
		exit(1);
	}
	let result = match Opt::parse() {
		Opt::Import(opt) => import::run(opt),
		Opt::SetWidget(opt) => set_widget::run(opt),
		Opt::ShellCompletion(opt) => print_shell_completion::run(opt),
		Opt::CreateIndex(opt) => create_index::run(opt)
	};
	if let Err(error) = result {
		eprintln!("{error:?}");
		exit(1);
	}
}

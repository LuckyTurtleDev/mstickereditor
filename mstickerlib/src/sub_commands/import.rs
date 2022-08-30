use crate::{
	config::{load_config_file, AnimationFormat, Config},
	matrix,
	matrix::upload_to_matrix,
	stickerpicker, tg, DATABASE_FILE, PROJECT_DIRS
};
use anyhow::{anyhow, bail, Context};
use clap::Parser;
use colored::*;
use flate2::write::GzDecoder;
use generic_array::GenericArray;
use indicatif::{ProgressBar, ProgressStyle};
use libwebp::WebPGetInfo as webp_get_info;
use lottie2gif::Color;
use lottie2webp;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{digest::OutputSizeUser, Digest, Sha512};
use std::{
	collections::BTreeMap,
	fs::{self, File},
	io::{self, BufRead, Write},
	path::Path,
	process::exit
};
use tempfile::NamedTempFile;

#[derive(Debug, Parser)]
pub struct Opt {
	/// Pack url
	#[clap(required = true)]
	packs: Vec<String>,

	/// Save stickers to disk
	#[clap(short, long)]
	save: bool,

	/// Does not upload the sticker to Matrix
	#[clap(short = 'U', long)]
	noupload: bool,

	/// Do not format the stickers;
	/// The stickers can may not be shown by a matrix client
	#[clap(short = 'F', long)]
	noformat: bool,

	/// format to which the stickers well be converted
	#[clap(long)]
	animation_format: Option<AnimationFormat>
}



#[derive(Debug, Deserialize, Serialize)]
struct HashUrl {
	hash: Hash,
	url: String
}

pub struct Sticker {
	file_hash: Hash,
	pub mxc_url: String,
	pub file_id: String,

	pub emoji: String,
	pub width: u32,
	pub height: u32,
	pub file_size: usize,
	pub mimetype: String
}

pub fn run(mut opt: Opt) -> anyhow::Result<()> {
	let config = load_config_file()?;

	if !opt.noupload {
		matrix::whoami(&config.matrix).expect("Error connecting to Matrix homeserver");
	}
	let mut packs: Vec<String> = Vec::new();
	while let Some(pack) = opt.packs.pop() {
		let mut id = pack.strip_prefix("https://t.me/addstickers/");
		if id.is_none() {
			id = pack.strip_prefix("tg://addstickers?set=");
		};
		match id {
			None => {
				eprintln!("{pack:?} does not look like a Telegram StickerPack");
				exit(1);
			},
			Some(id) => packs.push(id.into())
		};
	}
	for pack in packs {
		import_pack(&pack, &config, &opt)?;
	}
	Ok(())
}

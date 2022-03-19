use crate::config;
use anyhow::{bail, Context};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::read_dir;

#[derive(Debug, Parser)]
pub struct Opt {
	/// pretty human readable index.json
	#[clap(short, long)]
	pretty: bool,

	/// Matrix homeserver, with does render the preview thumbs
	#[clap(short = 's', long)]
	homeserver: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
struct INDEX {
	packs: Vec<String>,
	homeserver_url: String
}

pub fn run(opt: Opt) -> anyhow::Result<()> {
	let mut packs: Vec<String> = Vec::new();
	let paths = read_dir("./")?;
	for path in paths {
		let path = path?.path();
		let file = path.file_name();
		if let Some(file) = file {
			let file = file.to_str().unwrap();
			if file.ends_with(".json") && !file.ends_with("index.json") {
				println!("add pack {}", &file[..file.len() - 5]);
				packs.push(file.into())
			}
		}
	}
	if packs.is_empty() {
		bail!("Error: no stickerpacks found at working directory")
	}
	let homeserver_url = match opt.homeserver {
		Some(value) => value,
		None => config::load_config_file()?.matrix.homeserver_url
	};
	let index = INDEX { packs, homeserver_url };
	let string = match opt.pretty {
		true => serde_json::to_string_pretty(&index).unwrap(),
		false => serde_json::to_string(&index).unwrap()
	};
	std::fs::write("index.json", string).context("Error: could not save `index.json`")?;
	Ok(())
}

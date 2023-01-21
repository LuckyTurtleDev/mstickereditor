use crate::{load_config_file, DATABASE_FILE};
use anyhow::Context;
use clap::Parser;
use mstickerlib::{database::simple_file::FileDatabase, matrix, matrix::stickerpack::StickerPack};
use std::{path::Path, process::exit};
use tokio::fs;

#[derive(Debug, Parser)]
pub struct Opt {
	/// Pack url
	#[clap(required = true)]
	packs: Vec<String>,

	/// Save stickers to disk
	#[clap(short, long)]
	save: bool,

	/// Does not upload the sticker to Matrix
	#[clap(short = 'd', long)]
	dryrun: bool,

	/// Do not format the stickers;
	/// The stickers can may not be shown by a matrix client
	#[clap(short = 'F', long)]
	noformat: bool
}

#[tokio::main]
pub async fn run(mut opt: Opt) -> anyhow::Result<()> {
	let config = load_config_file()?;

	if !opt.dryrun {
		matrix::whoami(&config.matrix)
			.await
			.expect("Error connecting to Matrix homeserver");
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
				eprintln!("Pack url should start with \"https://t.me/addstickers/\" or \"tg://addstickers?set=\"");
				exit(1);
			},
			Some(id) => packs.push(id.into())
		};
	}
	let database = FileDatabase::new(&*DATABASE_FILE)?;
	for pack in packs {
		let matrix_pack = StickerPack::import_pack(
			&pack,
			Some(&database),
			&config.telegram,
			opt.dryrun,
			opt.save,
			&config.matrix,
			&config.sticker
		)
		.await
		.with_context(|| format!("failed to import pack {pack}"))?;
		fs::write(
			Path::new(&format!("./{}.json", matrix_pack.tg_pack.short_name)),
			serde_json::to_string(&matrix_pack)?
		)
		.await?;
	}
	Ok(())
}

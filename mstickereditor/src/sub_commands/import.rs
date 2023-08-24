use crate::{load_config_file, DATABASE_FILE};
use anyhow::Context;
use clap::Parser;
use mstickerlib::{
	database::simple_file::FileDatabase,
	matrix,
	matrix::{sticker_formats::maunium, stickerpack::StickerPack},
	tg::pack_url_to_name
};
use std::{path::PathBuf, process::exit};
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
	let animation_fromat = if opt.dryrun { None } else { Some(&config.sticker) };
	let mut packs: Vec<String> = Vec::new();
	while let Some(pack) = opt.packs.pop() {
		let name = pack_url_to_name(&pack).unwrap_or_else(|err| {
			eprintln!("{err}");
			exit(1)
		});
		packs.push(name.to_owned());
	}
	let database = FileDatabase::new(&*DATABASE_FILE).await?;

	for pack in packs {
		let matrix_pack = StickerPack::import_pack(
			&pack,
			Some(&database),
			&config.telegram,
			opt.dryrun,
			opt.save,
			&config.matrix,
			animation_fromat
		)
		.await
		.with_context(|| format!("failed to import pack {pack}"))?;
		let path: PathBuf = format!(
			"./{}.json",
			matrix_pack
				.tg_pack
				.as_ref()
				.map(|f| f.name.clone())
				.unwrap_or_else(|| matrix_pack.title.to_owned())
		)
		.into();
		let matrix_pack: maunium::StickerPack = matrix_pack.into();
		fs::write(path, serde_json::to_string(&matrix_pack)?).await?;
	}
	Ok(())
}

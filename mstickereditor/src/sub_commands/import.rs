use crate::{load_config_file, DATABASE_FILE};
use anyhow::{bail, Context};
use clap::Parser;
use log::info;
use mstickerlib::{
	database::FileDatabase,
	matrix,
	matrix::sticker_formats::maunium,
	tg::{self, pack_url_to_name, ImportConfig}
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
	let mut packs: Vec<String> = Vec::new();
	while let Some(pack) = opt.packs.pop() {
		let name = pack_url_to_name(&pack).unwrap_or_else(|err| {
			eprintln!("{err}");
			exit(1)
		});
		packs.push(name.to_owned());
	}
	let database = FileDatabase::new(&*DATABASE_FILE).await?;
	let mut import_config = ImportConfig::default();
	import_config.database = Some(&database);
	import_config.dry_run = opt.dryrun;
	let import_config = import_config;

	for pack in packs {
		let tg_pack = tg::StickerPack::get(&pack, &config.telegram)
			.await
			.with_context(|| format!("failed to get telegram sticker pack {pack:?}"))?;
		let matrix_pack = tg_pack.import(&config.telegram, &config.matrix, &import_config).await;
		let matrix_pack = match matrix_pack {
			Ok(pack) => pack,
			Err((matrix_pack, errors)) => {
				if let Some((_index, error)) = errors.into_iter().next() {
					return Err(error.context(format!("failed to import pack {pack:?}")));
				}
				matrix_pack
			}
		};
		if matrix_pack.stickers.is_empty() {
			bail!("imported pack {pack:?} is empty")
		}
		if opt.save {
			info!("save sticker of stickerpack {} to disk", matrix_pack.title);
			let dir = format!("./stickers/{}/", matrix_pack.tg_pack.as_ref().unwrap().name);
			std::fs::create_dir_all(&dir).with_context(|| format!("failed to create dir {dir:?}"))?;
			for sticker in &matrix_pack.stickers {
				{
					let index = sticker.tg_sticker.as_ref().unwrap().index.unwrap(); //should exist, since we have import the sticker from telegram right now
					let extension = sticker.image.meta_data.mimetype.split('/').last().unwrap();
					let path = format!("{dir}/{index:03}.{extension}");
					fs::write(&path, sticker.image.url.data().as_ref().unwrap().as_ref())
						.await
						.with_context(|| format!("failed to save sticker to {path:?}"))?;
				}
			}
		}
		let matrix_pack: maunium::StickerPack = matrix_pack.into();
		let path: PathBuf = format!(
			"./{}.json",
			matrix_pack
				.tg_pack
				.as_ref()
				.map(|f| f.short_name.clone()) // this should be never None, however sure is sure
				.unwrap_or_else(|| matrix_pack.title.to_owned())
		)
		.into();
		fs::write(path, serde_json::to_string(&matrix_pack)?).await?;
	}
	Ok(())
}

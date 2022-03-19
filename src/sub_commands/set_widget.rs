use crate::{config::load_config_file, matrix::set_widget};
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Opt {
	/// The url of your sticker picker
	widgeturl: String
}

pub fn run(opt: Opt) -> anyhow::Result<()> {
	let config = load_config_file()?;
	set_widget(&config.matrix, config.matrix.user.clone(), opt.widgeturl).expect("Error setting widget");
	Ok(())
}

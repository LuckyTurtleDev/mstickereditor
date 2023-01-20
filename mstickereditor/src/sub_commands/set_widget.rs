use crate::{load_config_file, matrix::set_widget, new_current_thread_runtime};
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Opt {
	/// The url of your sticker picker
	widgeturl: String
}

pub fn run(opt: Opt) -> anyhow::Result<()> {
	let config = load_config_file()?;
	new_current_thread_runtime()
		.expect("failed to starte tokio runtime")
		.block_on(set_widget(&config.matrix, config.matrix.user.clone(), opt.widgeturl))
		.expect("Error enabling widget");

	Ok(())
}

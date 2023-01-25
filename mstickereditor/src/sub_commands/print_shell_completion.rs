use clap::{IntoApp, Parser};
use std::io::stdout;

#[derive(Debug, Parser)]
pub struct Opt {
	shell: clap_complete::Shell
}

pub fn run(opt: Opt) -> anyhow::Result<()> {
	clap_complete::generate(opt.shell, &mut Opt::command(), crate::CARGO_PKG_NAME, &mut stdout());
	Ok(())
}

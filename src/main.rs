use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Opt {
	///import Stickerpack from telegram
	Import {
		///import file
		pack: String,

		///shew debug messages
		#[structopt(short, long)]
		debug: bool,
	},
}

fn main() {
	let opt = Opt::from_args();
	println!("Hello, world!");
}

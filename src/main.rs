use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct OptImport {
	///pack url
	pack: String,

	///show debug messages
	#[structopt(short, long)]
	debug: bool,
}

#[derive(Debug, StructOpt)]
enum Opt {
	///import Stickerpack from telegram
	Import(OptImport),
}

fn import(opt: OptImport) {
	println!("import {:?}", opt);
}

fn main() {
	match Opt::from_args() {
		Opt::Import(opt) => import(opt),
	}
	println!("Hello, world!");
}

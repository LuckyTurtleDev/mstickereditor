use super::{Database, Hash};

use anyhow::{self, Context};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use serde_json;
use std::{
	collections::BTreeMap,
	fs,
	fs::File,
	io,
	io::{BufRead, Write},
	path::Path
};

#[derive(Debug, Deserialize, Serialize)]
struct HashUrl {
	#[serde(with = "BigArray")]
	hash: Hash,
	url: String
}

pub struct FileDatabase {
	tree: BTreeMap<Hash, String>,
	file: fs::File
}

impl FileDatabase {
	fn new<P>(path: P) -> io::Result<FileDatabase>
	where
		P: AsRef<Path>
	{
		let path = path.as_ref();
		let mut tree = BTreeMap::<Hash, String>::new();
		match File::open(path) {
			Ok(file) => {
				let bufreader = std::io::BufReader::new(file);
				for (i, line) in bufreader.lines().enumerate() {
					let hashurl: Result<HashUrl, serde_json::Error> = serde_json::from_str(&line?);
					match hashurl {
						Ok(value) => {
							tree.insert(value.hash, value.url);
						},
						Err(error) => eprintln!(
							"Warning: Line {} of Database({}) can not be read: {:?}",
							i + 1,
							path.display(),
							error
						)
					};
				}
			},
			Err(error) if error.kind() == io::ErrorKind::NotFound => {
				print!("database not found, creating a new one");
			},
			Err(error) => {
				return Err(error);
			}
		};
		let file = fs::OpenOptions::new().write(true).append(true).create(true).open(path)?;
		Ok(FileDatabase { tree, file })
	}
}

impl Database for FileDatabase {
	fn get(&self, hash: &Hash) -> Option<&String> {
		self.tree.get(hash)
	}
	fn add(&self, hash: Hash, url: String) -> anyhow::Result<()> {
		let hash_url = HashUrl { hash, url };
		writeln!(self.file, "{}", serde_json::to_string(&hash_url)?)?;
		self.tree.insert(hash_url.hash, hash_url.url);
		Ok(())
	}
}

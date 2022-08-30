use super::{Database, Hash};

use anyhow;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::{collections::BTreeMap, fs::File, io, io::BufRead, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
struct HashUrl {
	#[serde(with = "BigArray")]
	hash: Hash,
	url: String
}

pub struct FileDatabase {
	path: PathBuf,
	tree: BTreeMap<Hash, String>
}

impl FileDatabase {
	fn new(path: PathBuf) -> io::Result<FileDatabase> {
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
		Ok(FileDatabase { path, tree })
	}
}

impl Database for FileDatabase {
	fn check(&self) -> bool {
		unimplemented!()
	}
	fn add(&self) -> anyhow::Result<()> {
		unimplemented!()
	}
}

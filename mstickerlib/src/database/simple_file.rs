use super::{Database, Hash};

use anyhow;
use futures_util::stream::StreamExt as _;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use serde_json;
use std::{collections::BTreeMap, io, path::Path};
use tokio::{
	fs::{self, File},
	io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufReader},
	sync::{Mutex, RwLock}
};
use tokio_stream::wrappers::LinesStream;

#[derive(Debug, Deserialize, Serialize)]
struct HashUrl {
	#[serde(with = "BigArray")]
	hash: Hash,
	url: String
}

/// simple implemtation of the `Database` traid,
/// with does save data to a file
pub struct FileDatabase {
	tree: RwLock<BTreeMap<Hash, String>>,
	file: Mutex<fs::File>
}

impl FileDatabase {
	pub async fn new<P>(path: P) -> io::Result<FileDatabase>
	where
		P: AsRef<Path>
	{
		let path = path.as_ref();
		let mut tree = BTreeMap::<Hash, String>::new();
		match File::open(path).await {
			Ok(file) => {
				let bufreader = BufReader::new(file);
				let mut lines = LinesStream::new(bufreader.lines()).enumerate();
				while let Some((i, line)) = lines.next().await {
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
		let file = fs::OpenOptions::new()
			.write(true)
			.append(true)
			.create(true)
			.open(path)
			.await?;
		Ok(FileDatabase {
			tree: RwLock::new(tree),
			file: Mutex::new(file)
		})
	}
}

impl Database for FileDatabase {
	async fn get(&self, hash: &Hash) -> anyhow::Result<Option<String>> {
		let lock = self.tree.read().await;
		let ret = lock.get(hash);
		Ok(ret.cloned())
	}

	async fn add(&self, hash: Hash, url: String) -> anyhow::Result<()> {
		let hash_url = HashUrl { hash, url };

		let mut file = self.file.lock().await;
		file.write_all(&serde_json::to_vec(&hash_url)?).await?;
		file.write_all(b"\n").await?;
		drop(file);

		let mut tree = self.tree.write().await;
		tree.insert(hash_url.hash, hash_url.url);
		Ok(())
	}
}

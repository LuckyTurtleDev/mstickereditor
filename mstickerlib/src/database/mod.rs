use anyhow;
use async_trait::async_trait;
use sha2::{Digest, Sha512};

pub mod simple_file;

pub type Hash = [u8; 64];

/// Database which stores mappings from hashes to matrix media urls,
/// to avoid duplicate uploads of the same file.
#[async_trait]
pub trait Database {
	async fn get(&self, hash: &Hash) -> Option<String>;
	async fn add(&self, hash: Hash, url: String) -> anyhow::Result<()>;
}

pub fn hash(value: &[u8]) -> Hash {
	let mut hasher = Sha512::new();
	hasher.update(value);
	hasher.finalize().into()
}

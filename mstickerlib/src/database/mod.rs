use anyhow;
use generic_array::GenericArray;
use sha2::{digest::OutputSizeUser, Digest, Sha512};

mod simple_file;

pub type Hash = [u8; 64];

pub trait Database {
	fn check(&self) -> bool;
	fn add(&self) -> anyhow::Result<()>;
}

pub fn hash(value: Vec<u8>) -> Hash {
	let mut hasher = Sha512::new();
	hasher.update(value);
	hasher.finalize().into()
}

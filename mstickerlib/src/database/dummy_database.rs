use super::{Database, Hash};
use async_trait::async_trait;

/// Dummy database to be used as default generic.
/// This database should be never constructed or used.
/// It should only be used to create the `None` variant of `Option<D> where D: Database`.
#[non_exhaustive]
pub struct DummyDatabase {}

#[async_trait]
impl Database for DummyDatabase {
	async fn get(&self, _: &Hash) -> Option<String> {
		None
	}

	async fn add(&self, _: Hash, _url: String) -> anyhow::Result<()> {
		{
			Ok(())
		}
	}
}

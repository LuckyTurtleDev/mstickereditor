#![allow(clippy::tabs_in_doc_comments)]
#![warn(unreachable_pub)]
#![cfg_attr(all(doc, nightly), feature(doc_auto_cfg))]

//! **WARINING: this crate is unstable und still have many anti-patterns**

pub mod database;
pub mod image;
pub mod matrix;
pub mod tg;
mod error;
pub use error::Error;

//mod sub_commands;

use std::cell::UnsafeCell;
use tokio::sync::RwLock;

static CLIENT: Client = Client {
	lock: RwLock::const_new(()),
	client: UnsafeCell::new(None)
};

unsafe impl Sync for Client {}

struct Client {
	lock: RwLock<()>,
	client: UnsafeCell<Option<reqwest::Client>>
}

impl Client {
	pub(crate) async fn get(&self) -> &reqwest::Client {
		// safety: this method ensures that the client is set from None to Some exactly once, and the
		// value is never altered thereafter. Once a value was set, all references to that value are
		// valid for the lifetime of self.

		let guard = self.lock.read().await;
		let client = unsafe { self.client.get().as_ref().unwrap() };
		if let Some(client) = client {
			return client;
		}
		drop(guard);

		#[allow(unused_variables)]
		let guard = self.lock.write().await;
		let client = unsafe { self.client.get().as_mut().unwrap() };
		if client.is_none() {
			*client = Some(reqwest::Client::new());
		}
		client.as_ref().unwrap()
	}
}
pub async fn set_client(client: reqwest::Client) {
	#[allow(unused_variables)]
	let guard = CLIENT.lock.read();
	let lib_client = unsafe { CLIENT.client.get().as_mut().unwrap() };
	if lib_client.is_some() {
		panic!("reqwest client was already set")
	}
	*lib_client = Some(client);
}

pub async fn get_client() -> &'static reqwest::Client {
	CLIENT.get().await
}

#![allow(clippy::tabs_in_doc_comments)]
#![warn(unreachable_pub)]
#![cfg_attr(all(doc, nightly), feature(doc_auto_cfg))]

//! **WARINING: this crate is unstable und still have many anti-patterns**

pub mod database;
pub mod error;
pub mod image;
pub mod matrix;
pub mod tg;
#[cfg(feature = "ffmpeg")]
mod video;

use std::sync::OnceLock;

struct Client(OnceLock<reqwest::Client>);
static CLIENT: Client = Client(OnceLock::new());

impl Client {
	fn get(&self) -> &'static reqwest::Client {
		if let Some(value) = CLIENT.0.get() {
			return value;
		};
		set_client(reqwest::Client::default()).ok();
		CLIENT.0.get().unwrap() //now it must be set
	}
}

/// set the crate wide [reqwest::Client].
/// This function should be called before performing any other interaction with this create.
/// Otherwise the client can not be set anymore and an error will be return.
/// If this function is not called, the client will be automaticly initialize with [reqwest::Client::default]
pub fn set_client(client: reqwest::Client) -> Result<(), ()> {
	init();
	CLIENT.0.set(client).map_err(|_| ())
}

pub fn get_client() -> &'static reqwest::Client {
	CLIENT.get()
}

// XXX Hacky: We abuse the fact that HTTP client will always be needed before ffmpeg.
fn init() {
	#[cfg(feature = "ffmpeg")]
	{
		static GUARD: OnceLock<()> = OnceLock::new();
		// from doc: "Returns Ok(()) if the cell’s value was set by this call."
		// so init will only be called once
		if GUARD.set(()).is_ok() {
			ffmpeg::init().expect("Failed to initialise ffmpeg");
		}
	}
}

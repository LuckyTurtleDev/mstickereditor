//! This module deals with translating telegram's video stickers to webp animations.

use crate::error::Error;
use ffmpeg::{
	codec::Context as CodecContext,
	decoder,
	format::{self, Pixel},
	media::Type,
	software::scaling::{context::Context as ScalingContext, flag::Flags},
	util::frame::video::Video
};
use std::path::Path;
use webp_animation::{Encoder, WebPData};

pub(crate) fn webm2webp<P: AsRef<Path>>(file: &P) -> Result<(WebPData, u32, u32), Error> {
	// heavily inspired by
	// https://github.com/zmwangx/rust-ffmpeg/blob/master/examples/dump-frames.rs

	let mut ictx = format::input(file)?;
	let input = ictx.streams().best(Type::Video).ok_or(ffmpeg::Error::StreamNotFound)?;

	let video_stream_index = input.index();
	let ctx_decoder = CodecContext::from_parameters(input.parameters())?;
	let mut decoder = ctx_decoder.decoder().video()?;

	let mut scaler = ScalingContext::get(
		decoder.format(),
		decoder.width(),
		decoder.height(),
		Pixel::RGBA,
		decoder.width(),
		decoder.height(),
		Flags::BILINEAR
	)?;

	let mut encoder = Encoder::new((decoder.width(), decoder.height()))?;
	let mut timestamp = 0;
	let frame_rate = input.rate();
	let time_per_frame = frame_rate.1 * 1000 / frame_rate.0;
	let mut receive_and_process_decoded_frames = |decoder: &mut decoder::Video| -> Result<(), Error> {
		let mut decoded = Video::empty();
		while decoder.receive_frame(&mut decoded).is_ok() {
			let mut rgba_frame = Video::empty();
			scaler.run(&decoded, &mut rgba_frame)?;

			encoder.add_frame(rgba_frame.data(0), timestamp)?;
			timestamp += time_per_frame;
		}
		Ok(())
	};

	for (stream, packet) in ictx.packets() {
		if stream.index() == video_stream_index {
			decoder.send_packet(&packet)?;
			receive_and_process_decoded_frames(&mut decoder)?;
		}
	}
	decoder.send_eof()?;
	receive_and_process_decoded_frames(&mut decoder)?;

	let webp = encoder.finalize(timestamp)?;
	Ok((webp, decoder.width(), decoder.height()))
}

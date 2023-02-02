//! Current different StickerPack formats exists for Matrix.
//! This mod store all current supported formats.

macro_rules! impl_from {
	($t_sticker:ty, $t_pack:ty) => {
		impl From<super::maunium::Sticker> for $t_sticker {
			fn from(value: super::maunium::Sticker) -> Self {
				let sticker: crate::matrix::sticker::Sticker = value.into();
				sticker.into()
			}
		}
		impl From<super::maunium::StickerPack> for $t_pack {
			fn from(value: super::maunium::StickerPack) -> Self {
				let sticker_pack: crate::matrix::stickerpack::StickerPack = value.into();
				sticker_pack.into()
			}
		}
	};
}

pub mod maunium;
pub mod ponies;

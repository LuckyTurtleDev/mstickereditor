use serde::Serialize;

#[derive(Serialize)]
pub struct StickerWidget {
	pub content: Content,
	pub sender: String,
	pub state_key: String,
	#[serde(rename = "stype")]
	pub stype: String,
	pub id: String
}

#[derive(Serialize)]
pub struct Content {
	#[serde(rename = "stype")]
	pub stype: String,
	pub url: String,
	pub name: String,
	pub data: String
}

impl StickerWidget {
	pub(crate) fn new(url: String, sender: String) -> Self {
		let content = Content {
			stype: String::from("m.stickerpicker"),
			url,
			name: String::from("Stickerpicker"),
			data: String::from("")
		};
		StickerWidget {
			content,
			sender,
			state_key: String::from("stickerpicker"),
			stype: String::from("m.widget"),
			id: String::from("stickerpicker")
		}
	}
}

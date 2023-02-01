use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct StickerWidget {
	content: Content,
	sender: String,
	state_key: String,
	#[serde(rename = "stype")]
	stype: String,
	id: String
}

#[derive(Serialize)]
struct Content {
	#[serde(rename = "stype")]
	stype: String,
	url: String,
	name: String,
	data: String
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

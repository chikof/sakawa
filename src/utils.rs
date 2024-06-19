use crate::defines::APP_ICON;

/// Load icon from memory and return it
pub fn load_icon() -> eframe::egui::IconData {
	let (icon_rgba, icon_width, icon_height) = {
		let image = image::load_from_memory(APP_ICON)
			.expect("Failed to open icon path")
			.into_rgba8();

		let (width, height) = image.dimensions();
		let rgba = image.into_raw();
		(rgba, width, height)
	};

	eframe::egui::IconData {
		rgba: icon_rgba,
		width: icon_width,
		height: icon_height,
	}
}

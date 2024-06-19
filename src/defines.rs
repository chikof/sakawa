use eframe::egui::FontFamily;

pub const APP_NAME: &str = "Sakawa Wuwa Modloader";
pub const APP_ICON: &[u8] = include_bytes!("../assets/icon-64x64.ico");

// Font
pub const FONT_SIZE: f32 = 12.0;
pub const FONT_FAMILY: FontFamily = FontFamily::Monospace;

// dimensions of main window
pub const WINDOW_WIDTH: f32 = 650.0;
pub const WINDOW_HEIGHT: f32 = 341.0;

// Default paths
pub const EPIC_PATH: &str =
	"C:\\Program Files\\Epic Games\\WutheringWavesj3oFh\\Wuthering Waves Game";
pub const KURO_PATH: &str = "Kuro"; // TODO: Default path for Kuro

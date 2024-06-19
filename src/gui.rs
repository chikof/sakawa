use eframe::egui::{self, vec2, Color32, FontId, Id};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use crate::defines::{EPIC_PATH, FONT_FAMILY, FONT_SIZE, KURO_PATH};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct Config {
	pub mods_path: String,
	pub game_path: String,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct SakawaApp {
	pub config: Config,
	pub available_mods: Option<Vec<String>>,
	pub installed_mods: Vec<String>,
	pub columns: Vec<Vec<String>>,
}

/// What is being dragged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
	col: usize,
	row: usize,
}

impl Default for SakawaApp {
	fn default() -> Self {
		let game_path = if cfg!(feature = "epic") {
			EPIC_PATH
		} else if cfg!(feature = "kuro") {
			KURO_PATH
		} else {
			EPIC_PATH
		};

		Self {
			config: Config {
				mods_path: String::from("mods"),
				game_path: String::from(game_path),
			},
			available_mods: None,
			installed_mods: Vec::new(),
			columns: Vec::new(),
		}
	}
}

impl SakawaApp {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let ctx = &cc.egui_ctx;

		let mut style = (*ctx.style()).clone();
		let font = FontId {
			size: FONT_SIZE,
			family: FONT_FAMILY,
		};
		style.override_font_id = Some(font);
		ctx.set_style(style);

		// Load previous app state (if any).
		// Note that you must enable the `persistence` feature for this to work.
		#[cfg(feature = "persistence")]
		if let Some(storage) = cc.storage {
			return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
		}

		Default::default()
	}

	pub fn load_available_mods(&mut self) -> Option<Vec<String>> {
		// Load all mods from mods_path
		let mod_path = Path::new(&self.config.mods_path);

		if !mod_path.exists() {
			return None;
		}

		let mods = mod_path
			.read_dir()
			.expect("Failed to read mods directory")
			.map(|entry| {
				entry
					.expect("Failed to read entry")
					.file_name()
					.into_string()
					.expect("Failed to convert OsString to String")
			})
			.collect::<Vec<String>>();

		self.available_mods = Some(mods.clone());

		Some(mods)
	}

	#[allow(dead_code)]
	fn load_installed_mods(&mut self) -> Option<Vec<String>> {
		self.installed_mods.clear();

		let mods_directory = Path::new(&self.config.game_path)
			.join("Client")
			.join("Content")
			.join("Paks")
			.join("~mod");

		if mods_directory.exists() {
			for entry in fs::read_dir(mods_directory).unwrap() {
				let entry = entry.unwrap();
				let mod_file = entry.path();
				if mod_file.is_file() {
					self.installed_mods
						.push(mod_file.file_name().unwrap().to_str().unwrap().to_string());
				}
			}
		}

		Some(self.installed_mods.clone())
	}

	#[allow(dead_code)]
	fn install_mod(&mut self, selected_mod: &str) {
		let source_file = std::env::current_dir()
			.unwrap()
			.join("mods")
			.join(selected_mod);

		let target_directory = Path::new(&self.config.game_path)
			.join("Client")
			.join("Content")
			.join("Paks")
			.join("~mod");

		if !target_directory.exists() {
			fs::create_dir_all(&target_directory).unwrap();
		}

		let target_file = target_directory.join(selected_mod);
		fs::copy(source_file, target_file).unwrap();
		self.load_installed_mods();
	}

	#[allow(dead_code)]
	fn uninstall_mod(&mut self, selected_mod: &str) {
		let target_directory = Path::new(&self.config.game_path)
			.join("Client")
			.join("Content")
			.join("Paks")
			.join("~mod");

		let target_file = target_directory.join(selected_mod);

		if target_file.exists() {
			fs::remove_file(target_file).unwrap();
			self.load_installed_mods();
		}
	}

	#[allow(dead_code)]
	fn launch_game(&self, with_mods: bool) {
		let executable_name = "Wuthering Waves.exe";
		let executable_path = Path::new(&self.config.game_path).join(executable_name);
		if executable_path.exists() {
			let mut cmd = Command::new(executable_path);
			if with_mods {
				cmd.arg("-fileopenlog");
			}
			cmd.spawn().unwrap();
		} else {
			// Error handling to be implemented
		}
	}
}

impl eframe::App for SakawaApp {
	fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
		egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
	}

	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading("Sakawa");

			ui.horizontal(|ui| {
				if ui.button("Select game path").clicked() {
					if let Some(path) = rfd::FileDialog::new().pick_folder() {
						self.config.game_path = path.display().to_string();
					}
				}

				if ui.button("Select mods path").clicked() {
					if let Some(path) = rfd::FileDialog::new().pick_folder() {
						self.config.mods_path = path.display().to_string();
						self.available_mods = None;
						self.columns.clear();
					}
				}
			});

			if self.columns.is_empty() {
				let mut mods = self.load_available_mods().unwrap();
				let installed_mods = self.load_installed_mods().unwrap();

				for r#mod in mods.clone().iter() {
					if installed_mods.contains(r#mod) {
						mods.retain(|x| x != r#mod);
					}
				}

				self.columns = vec![mods, installed_mods];
			}

			// If there is a drop, store the location of the item being dragged, and the destination for the drop.
			let mut from: Option<Arc<Location>> = None;
			let mut to: Option<Location> = None;

			ui.columns(2, |uis| {
				for (index, column) in self.columns.clone().into_iter().enumerate() {
					let ui = &mut uis[index];
					let frame = egui::Frame::default().inner_margin(4.0);

					let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
						ui.set_min_size(vec2(200.0, 100.0));

						for (row_idx, item) in column.iter().enumerate() {
							let item_id = Id::new(("mod", index, row_idx));
							let item_location = Location {
								col: index,
								row: row_idx,
							};

							let response = ui
								.dnd_drag_source(item_id, item_location, |ui| {
									ui.label(format!("#{} {}", row_idx + 1, item));
									ui.separator()
								})
								.response;

							if let (Some(pointer), Some(hovered_payload)) = (
								ui.input(|i| i.pointer.interact_pos()),
								response.dnd_hover_payload::<Location>(),
							) {
								let rect = response.rect;

								let stroke = egui::Stroke::new(1.0, Color32::WHITE);
								let insert_row_idx = if *hovered_payload == item_location {
									ui.painter().hline(rect.x_range(), rect.center().y, stroke);
									row_idx
								} else if pointer.y < rect.center().y {
									ui.painter().hline(rect.x_range(), rect.top(), stroke);
									row_idx
								} else {
									ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
									row_idx + 1
								};

								if let Some(dragged_payload) = response.dnd_release_payload() {
									from = Some(dragged_payload);
									to = Some(Location {
										col: index,
										row: insert_row_idx,
									});
								}
							}
						}
					});

					if let Some(dragged_payload) = dropped_payload {
						// The user dropped onto the column, but not on any one item.
						from = Some(dragged_payload);
						to = Some(Location {
							col: index,
							row: usize::MAX, // Inset last
						});
					}
				}

				if let (Some(from), Some(mut to)) = (from, to) {
					if from.col == to.col {
						// Dragging within the same column.
						// Adjust row index if we are re-ordering:
						to.row -= (from.row < to.row) as usize;
					}

					let item = self.columns[from.col].remove(from.row);

					let column = &mut self.columns[to.col];
					to.row = to.row.min(column.len());
					column.insert(to.row, item);
				}
			});
		});
	}
}

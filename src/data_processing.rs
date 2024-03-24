use std::io::Read;

use egui::{CentralPanel, Color32, Rounding, SidePanel, Ui};
use ndarray::Array1;
use rfd::FileDialog;

const DATASET_SEPARATOR: &str = "-=-=-=-=-=-=-";

#[derive(Debug)]
pub struct DataSet {
    raw_data: Vec<String>,
    processed_data: Option<Vec<Array1<f32>>>,
    name: String,
}

#[derive(Debug)]
pub struct DataProcessingUI {
    datasets: Vec<DataSet>,
    shown_dataset_index: Option<usize>, 
}

impl Default for DataProcessingUI {
    fn default() -> Self {
        Self { datasets: vec![], shown_dataset_index: None }
    }
}

impl DataProcessingUI {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.painter().rect_filled(ui.max_rect(), Rounding::ZERO, Color32::WHITE);
        if let Some(ind) = self.shown_dataset_index {
            SidePanel::right("tooltip_data").resizable(false).show_inside(ui, |ui| {
                ui.label(&self.datasets[ind].name);
            });
        }

        ui.vertical_centered(|ui| {
            if self.datasets.len() == 0 {
                ui.label("No datasets loaded");
            }
            else {
                for dataset in &self.datasets {
                    ui.label(dataset.name.as_str());
                }
            }
    
            if ui.button("Load a new dataset from file").clicked() {
                let files = FileDialog::new()
                    .add_filter("text", &["txt", "rs"])
                    .add_filter("rust", &["rs", "toml"])
                    .set_directory(".")
                    .pick_file();

                if let Some(path) = files {
                    // ui.ctx().set_cursor_icon(egui::CursorIcon::Wait);
                    if let Ok(mut open_file) = std::fs::File::open(&path) {
                        let mut file_contents = String::new();
                        let res = open_file.read_to_string(&mut file_contents);
                        if res.is_ok() {
                            let raw_data = file_contents.split(DATASET_SEPARATOR).map(|val| val.to_string()).collect();
                            self.datasets.push(DataSet {
                                raw_data,
                                processed_data: None,
                                name: path.file_name().unwrap().to_os_string().into_string().unwrap(),
                            });
                            self.shown_dataset_index = Some(self.datasets.len() - 1);
                        }
                    }
                    // ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                }

            }
        });
        
    }
}

use std::{io::Read, vec};

use egui::{include_image, CentralPanel, Color32, ComboBox, Frame, Grid, Image, Rounding, ScrollArea, Sense, SidePanel, Stroke, Style, Ui, Vec2};
use ndarray::Array1;
use rfd::FileDialog;

const DATASET_SEPARATOR: &str = "-=-=-=-=-=-=-";

#[derive(Debug, PartialEq)]
enum ProcessingType {
    Word2Vec
}

#[derive(Debug, PartialEq, Clone)]
pub struct DataSet {
    raw_data: Vec<String>,
    processed_data: Option<Vec<Array1<f32>>>,
    pub name: String,
}

impl DataSet {
    pub fn is_processed(&self) -> bool {
        self.processed_data.is_some()
    }
}

#[derive(Debug)]
pub struct DataProcessingUI {
    pub datasets: Vec<DataSet>,
    shown_dataset_index: Option<usize>, 
}

impl Default for DataProcessingUI {
    fn default() -> Self {
        Self { datasets: vec![], shown_dataset_index: None }
    }
}

impl DataProcessingUI {
    fn dataset_list(&mut self, ui: &mut Ui) {
        for (index, dataset) in self.datasets.iter().enumerate() {
            let frame_style = Style::default();
            let is_current = Some(index) == self.shown_dataset_index;
            let stroke_color = if is_current {
                Color32::DARK_GRAY
            }
            else {
                Color32::LIGHT_GRAY
            };

            let mut frame = Frame::group(&frame_style)
                .rounding(Rounding::same(3.0))
                .stroke(Stroke::new(1.5, stroke_color))
                .inner_margin(2.5)
                .outer_margin(2.5)
                .fill(Color32::LIGHT_GRAY)
                .begin(ui);
            frame.content_ui.horizontal(|ui|{
                if dataset.is_processed() {
                    ui.add(
                        Image::new(include_image!("../resources/dataset_processed.svg"))
                            .rounding(5.0).fit_to_exact_size(Vec2 { x: 30.0, y: 30.0 })
                        );
                }
                else {
                    ui.add(
                    Image::new(include_image!("../resources/dataset_raw.svg"))
                        .rounding(5.0).fit_to_exact_size(Vec2 { x: 30.0, y: 30.0 })
                    );
                }

                ui.centered_and_justified(|ui| {
                    ui.label(dataset.name.as_str());
                });
            });

            let response = frame.allocate_space(ui).on_hover_cursor(egui::CursorIcon::PointingHand).interact(Sense::click());
            if response.clicked() {
                self.shown_dataset_index = Some(index);
            }

            if response.hovered() {
                frame.frame.fill = Color32::WHITE;
            }
            frame.paint(ui);
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.painter().rect_filled(ui.max_rect(), Rounding::ZERO, Color32::WHITE);
        if let Some(ind) = self.shown_dataset_index {
            SidePanel::right("tooltip_data")
            .resizable(true)
            .show_inside(ui, |ui| {
                let chosen_dataset = &mut self.datasets[ind];
                ui.text_edit_singleline(&mut chosen_dataset.name);
                
                Grid::new("Parameters").show(ui, |ui| {
                    ui.label("Processing Type: ");
                    if chosen_dataset.is_processed() {
                        ui.label("Word2Vec 100");
                    }
                    else {
                        ui.label("Unprocessed (Raw)");
                    }
                    ui.end_row();
                });

                let mut processing_type = ProcessingType::Word2Vec;
                ComboBox::from_label("Type of text processing use")
                .selected_text(format!("{:?}", processing_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut processing_type, ProcessingType::Word2Vec, "Word2Vec");
                });

                if ui.button("Apply chosen processing").clicked() {
                    // ToDo: Add the actual processing and maybe add processing types to dataset struct
                    chosen_dataset.processed_data = Some(vec![]);
                }

                ui.separator();
            });
        }

        ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                if self.datasets.len() == 0 {
                    ui.label("No datasets loaded");
                }
                else {
                    self.dataset_list(ui);
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
        });
        
    }
}

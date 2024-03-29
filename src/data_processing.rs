use std::{fs::File, io::{BufReader, Read}, sync::{Arc, Mutex}, thread, time::Duration, vec};
use finalfusion::prelude::*;

use egui::{include_image, CentralPanel, Color32, ComboBox, Frame, Grid, Image, Layout, Rounding, ScrollArea, Sense, SidePanel, Stroke, Style, Ui, Vec2};
use ndarray::{concatenate, Array1, Axis};
use rfd::FileDialog;

const DATASET_SEPARATOR: &str = "-=-=-=-=-=-=-";
// const DATASET_SEPARATOR: &str = "\n";

#[derive(Debug, PartialEq)]
enum ProcessingType {
    Word2Vec
}

// Mutex gives interior mutability!
// This finally makes sense
pub fn process_dataset(dataset: Arc<Mutex<DataSet>>) {
    dataset.lock().unwrap().is_being_processed = true;

    // ToDo: Make this shared, maybe store in Data Processing UI struct?
    // Also, maybe use some other method for representing text
    // I kinda don't like how you need to distribute weights with the app
    let mut reader =
        BufReader::new(File::open("C:/All/glove-twitter-25/glove-twitter-25.txt").unwrap());
    let embeddings = Embeddings::read_text_dims(&mut reader).unwrap();

    let mut result = vec![];
    let lines = dataset.lock().unwrap().raw_data.clone();

    for sample in lines {
        let stripped_contents = sample
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>();

        let words: Vec<&str> = stripped_contents.split_whitespace().into_iter().collect();
        if words.len() > 0 {
            let vecs: Vec<_> = words
                .iter()
                .filter_map(|word| embeddings.embedding(word))
                .collect();

            let view_vec = vecs.iter().map(|a| a.view()).collect::<Vec<_>>();
            let final_vec = concatenate(Axis(0), view_vec.as_slice())
                .unwrap()
                .map(|x| *x as f32);

            result.push(final_vec);
        }
    }

    dataset.lock().unwrap().is_being_processed = false;
    println!("{result:?}");
    dataset.lock().unwrap().processed_data = Some(result);
}

#[derive(Debug, PartialEq, Clone)]
pub struct DataSet {
    raw_data: Vec<String>,
    pub processed_data: Option<Vec<Array1<f32>>>,
    pub name: String,
    is_being_processed: bool,
}

impl DataSet {
    pub fn is_processed(&self) -> bool {
        self.processed_data.is_some()
    }
}

#[derive(Debug)]
pub struct DataProcessingUI {
    pub datasets: Vec<Arc<Mutex<DataSet>>>,
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
            let dataset = dataset.lock().unwrap();

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

                ui.horizontal_centered(|ui| {
                    ui.label(dataset.name.as_str());
                    
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui|{
                        if dataset.is_being_processed {
                            ui.spinner();
                        }
                    });
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
                let chosen_dataset = &mut self.datasets[ind].lock().unwrap();
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
                    let cloned_dataset = self.datasets[ind].clone();
                    thread::spawn(|| {
                        process_dataset(cloned_dataset);
                    });
                }

                ui.separator();
            });
        }

        ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                if self.datasets.len() == 0 {
                    ui.label("No datasets loaded");
                }
                self.dataset_list(ui);
        
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
                                self.datasets.push(Arc::new(Mutex::new(DataSet {
                                    raw_data,
                                    processed_data: None,
                                    name: path.file_name().unwrap().to_os_string().into_string().unwrap(),
                                    is_being_processed: false,
                                })));
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

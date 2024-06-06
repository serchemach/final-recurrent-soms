use std::{clone, collections::HashSet, fs::File, io::{BufReader, Read}, path::PathBuf, sync::{Arc, Mutex}, thread, time::Duration, vec};
use finalfusion::prelude::*;

use egui::{include_image, CentralPanel, Color32, ComboBox, DragValue, Frame, Grid, Image, Layout, Rounding, ScrollArea, Sense, SidePanel, Stroke, Style, Ui, Vec2};
use ndarray::{concatenate, Array1, Axis};
use rfd::FileDialog;

use tqdm::tqdm;
use crate::{msom::MSOM, SOMParams};
use serde::{Serialize, Deserialize};

const DATASET_SEPARATOR: &str = "-=-=-=-=-=-=-";
// const DATASET_SEPARATOR: &str = "\n";

#[derive(Debug, PartialEq, Clone)]
enum ProcessingType {
    Word2Vec,
    DatasetContext
}

fn process_word2vec(dataset: Arc<Mutex<DataSet>>, params: SOMParams) -> Vec<Array1<f32>> {
    // let n = 10;
    // let m = 10;
    // let map_input_size = 25;
    // let a = 1.0; 
    // let b = 1.0;
    // let gamma = 0.5;
    // let train_iterations = 200; 
    // let learning_rate_base = 0.1; 
    // let gauss_width_squared_base = 10000.0; 
    // let time_constant = 200.0;

    let mut reader =
        BufReader::new(File::open("./resources/glove-twitter-25.txt").unwrap());
    let embeddings = Embeddings::read_text_dims(&mut reader).unwrap();

    let lines = dataset.lock().unwrap().raw_data.clone();
    let mut processed_texts = vec![];

    let mut dictionary: HashSet<String> = HashSet::new();

    for sample in lines {
        let stripped_contents = sample
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>();

        let words: Vec<&str> = stripped_contents.split_whitespace().into_iter().collect();
        for word in words {
            if embeddings.embedding(word).is_some() {
                dictionary.insert(word.to_owned());
            }
        }
        
        processed_texts.push(stripped_contents);
    }
    
    let words: Vec<String> = dictionary.drain().collect();
    let word_vecs: Vec<_> = words.iter()
        .map(|word| embeddings.embedding(word).unwrap())
        .collect();

    println!("{:?}", words);

    let mut word_map = MSOM::new(params.n, params.m, params.map_input_size, params.a, params.b, params.gamma);
    word_map.fit(&word_vecs.iter().map(|sample| sample.view()).collect(), 
        params.train_iterations, params.learning_rate_base, params.gauss_width_squared_base, params.time_constant);

    println!("Word map, text vec sizes {}", words.len());
    

    let mut text_vecs = vec![];
    for text in tqdm(processed_texts.iter()) {
        let text_words: Vec<&str> = text.split_whitespace().into_iter().collect();
        let mut cur_vec: Vec<f32> = vec![0.0; params.n * params.m];

        for word in text_words {
            if let Some(word_vec) = embeddings.embedding(word) {
                let map_pos = word_map.evaluate(word_vec.view());
                cur_vec[map_pos.0 * params.m + map_pos.1] += 1.0;
            }
        }
        let vec_sum  = cur_vec.iter().sum::<f32>();
        if vec_sum != 0.0 {
            text_vecs.push(Array1::from_vec(cur_vec) / vec_sum);
        }
        else {
            text_vecs.push(Array1::from_vec(cur_vec));
        }
    }

    text_vecs
}

fn process_dataset_context(dataset: Arc<Mutex<DataSet>>) -> Vec<Array1<f32>> {
    let mut reader =
        BufReader::new(File::open("./resources/glove-twitter-25.txt").unwrap());
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
    vec![]
}

// Mutex gives interior mutability!
// This finally makes sense
pub fn process_dataset(dataset: Arc<Mutex<DataSet>>, processing_type: ProcessingType, params: SOMParams) {
    dataset.lock().unwrap().is_being_processed = true;
    let result = match processing_type {
        ProcessingType::Word2Vec => process_word2vec(dataset.clone(), params),
        ProcessingType::DatasetContext => process_dataset_context(dataset.clone()),
    };

    // ToDo: Make this shared, maybe store in Data Processing UI struct?
    // Also, maybe use some other method for representing text
    // I kinda don't like how you need to distribute weights with the app
    

    dataset.lock().unwrap().is_being_processed = false;
    println!("{result:?}");
    dataset.lock().unwrap().processed_data = Some(result);
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DataSet {
    pub raw_data: Vec<String>,
    pub processed_data: Option<Vec<Array1<f32>>>,
    pub name: String,
    is_being_processed: bool,
}

impl DataSet {
    pub fn is_processed(&self) -> bool {
        self.processed_data.is_some()
    }

    fn from_file(filename: &PathBuf) -> Result<Self, &str> {
        let writer = File::options().read(true).open(filename);
        if writer.is_err() {
            return Err("Error while opening the file");
        }

        let res: Result<Self, serde_json::Error> = serde_json::from_reader(writer.unwrap());

        if res.is_err() {
            Err("Error parsing the dataset")
        }
        else {
            Ok(res.unwrap())
        }
    }

    fn to_file(&self, filename: &PathBuf) -> Result<(), &str> {
        // let json = serde_json::to_vec(&self.map_weights)?;
        let writer = File::options().write(true).create(true).open(filename);
        if writer.is_err() {
            return Err("Error while opening the file");
        }

        let res = serde_json::to_writer_pretty(writer.unwrap(), self);

        if res.is_err() {
            Err("Error while serializing the dataset")
        }
        else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct DataProcessingUI {
    pub datasets: Vec<Arc<Mutex<DataSet>>>,
    shown_dataset_index: Option<usize>, 
    current_processing_type: ProcessingType,
    current_params: SOMParams,
}

impl Default for DataProcessingUI {
    fn default() -> Self {
        Self { datasets: vec![], shown_dataset_index: None, current_processing_type: ProcessingType::Word2Vec, 
            current_params: SOMParams::default() }
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

            response.context_menu(|ui| {
                if ui.button("Save to file").clicked() {
                    let files = FileDialog::new()
                        .add_filter("Serde json file with dataset structure", &["json_set"])
                        .set_directory(".")
                        .save_file();
                    
                    if let Some(path) = files {
                        let res = dataset.to_file(&path);
                        if res.is_err() {
                            println!("{}", res.err().unwrap());
                        }
                    }

                    ui.close_menu();
                }
            });

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

                    let n = 10;
                    ui.label("n:");
                    ui.add(DragValue::new(&mut self.current_params.n));
                    ui.end_row();

                    ui.label("m:");
                    ui.add(DragValue::new(&mut self.current_params.m));
                    ui.end_row();

                    ui.label("map input size:");
                    ui.add(DragValue::new(&mut self.current_params.map_input_size));
                    ui.end_row();

                    ui.label("a:");
                    ui.add(DragValue::new(&mut self.current_params.a));
                    ui.end_row();

                    ui.label("b:");
                    ui.add(DragValue::new(&mut self.current_params.b));
                    ui.end_row();

                    ui.label("gamma:");
                    ui.add(DragValue::new(&mut self.current_params.gamma));
                    ui.end_row();

                    ui.label("train_iterations:");
                    ui.add(DragValue::new(&mut self.current_params.train_iterations));
                    ui.end_row();

                    ui.label("learning_rate_base:");
                    ui.add(DragValue::new(&mut self.current_params.learning_rate_base));
                    ui.end_row();

                    ui.label("gauss_width_squared_base:");
                    ui.add(DragValue::new(&mut self.current_params.gauss_width_squared_base));
                    ui.end_row();

                    ui.label("time_constant:");
                    ui.add(DragValue::new(&mut self.current_params.time_constant));
                    ui.end_row();
                });

                ComboBox::from_label("Type of text processing use")
                .selected_text(format!("{:?}", self.current_processing_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.current_processing_type, ProcessingType::Word2Vec, "Word2Vec");
                    ui.selectable_value(&mut self.current_processing_type, ProcessingType::DatasetContext, "DatasetContext");
                });

                if ui.button("Apply chosen processing").clicked() {
                    // ToDo: Add the actual processing and maybe add processing types to dataset struct
                    let cloned_dataset = self.datasets[ind].clone();
                    let cloned_processing_type = self.current_processing_type.clone();
                    let cloned_params = self.current_params.clone();
                    thread::spawn(|| {
                        process_dataset(cloned_dataset, cloned_processing_type, cloned_params);
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
        
                if ui.button("Create a new dataset from file").clicked() {
                    let files = FileDialog::new()
                        .add_filter("text", &["txt", "rs"])
                        .add_filter("rust", &["rs", "toml"])
                        .set_directory(".")
                        .pick_file();

                    if let Some(path) = files {
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
                    }

                }

                if ui.button("Load a processed dataset from file").clicked() {
                    self.current_params = SOMParams::default();

                    let files = FileDialog::new()
                            .add_filter("Serde json file with dataset structure", &["json_set"])
                            .set_directory(".")
                            .pick_file();
                        
                    if let Some(path) = files {
                        let res = DataSet::from_file(&path);
                        if res.is_err() {
                            println!("{}", res.err().unwrap());
                        }
                        else {
                            self.datasets.push(Arc::new(Mutex::new(res.unwrap())));
                        }
                    }
                }
            });
        });
        
    }
}

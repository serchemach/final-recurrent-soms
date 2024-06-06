use egui::{epaint::RectShape, include_image, Color32, ComboBox, DragValue, Frame, Grid, Image, Label, Layout, Pos2, Rect, Rounding, ScrollArea, Sense, Shape, SidePanel, Stroke, Style, Ui, Vec2};
use egui_modal::Modal;
use ndarray::{Array2, ArrayView1};
use rfd::FileDialog;
use tqdm::tqdm;

use crate::{data_processing::DataSet, msom::{get_vec_std, MSOM}, SOMParams};
use std::{cmp::{max, min}, fs::File, path::PathBuf, sync::{Arc, Mutex}};
use serde::{Serialize, Deserialize};

const TEXT_PREVIEW_CUTOFF: usize = 20;

pub fn calculate_visualization_data(visualization: Arc<Mutex<Visualization>>, map: MSOM, dataset: DataSet) {
    if let Some(samples) = dataset.processed_data {
        // println!("{}, {}", samples.len(), dataset.raw_data.len());
        
        let mut word_occurences: Vec<Vec<Vec<String>>> =
            vec![vec![vec![]; map.m]; map.n];

        let mut vector_occurences: Vec<Vec<Vec<ArrayView1<f32>>>> =
                vec![vec![vec![]; map.m]; map.n];

        for (index, sample) in tqdm(samples.iter().enumerate()) {
            let prediction = map.evaluate(sample.view());
            vector_occurences[prediction.0][prediction.1].push(sample.view());
            
            word_occurences[prediction.0][prediction.1].push(dataset.raw_data[index].replace("\n", " "));

            // println!("{index}");
        }

        visualization.lock().unwrap().word_clusters = word_occurences;

        let mut counts: Vec<Vec<f32>> = vec![vec![0.0; map.m]; map.n];
        for row_i in 0..map.n {
            for col_i in 0..map.m {
                counts[row_i][col_i] += vector_occurences[row_i][col_i].len() as f32;
            }
            // println!("{row_i}");
        }
        
        visualization.lock().unwrap().data = counts;
    }


}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visualization {
    name: String,
    data: Vec<Vec<f32>>,
    word_clusters: Vec<Vec<Vec<String>>>,

    is_calculating: bool,
}

impl Default for Visualization {
    fn default() -> Self {
        Self { name: "Name".to_owned(), data: vec![], word_clusters: vec![], is_calculating: false }
    }
}

impl Visualization {
    fn from_file(filename: &PathBuf) -> Result<Self, &str> {
        let writer = File::options().read(true).open(filename);
        if writer.is_err() {
            return Err("Error while opening the file");
        }

        let res: Result<Visualization, serde_json::Error> = serde_json::from_reader(writer.unwrap());

        if res.is_err() {
            Err("Error parsing the file")
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
            Err("Error while serializing the map")
        }
        else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct VisualizationsUI {
    visualizations: Vec<Arc<Mutex<Visualization>>>,
    shown_visualization_index: Option<usize>,
    current_visualization: Visualization,

    chosen_dataset_index: Option<usize>,
    chosen_map_index: Option<usize>,
    
    current_shown_square: (usize, usize),
}

impl Default for VisualizationsUI {
    fn default() -> Self {
        Self { visualizations: vec![], shown_visualization_index: None, current_visualization: Visualization::default(), 
            chosen_dataset_index: None, chosen_map_index: None, current_shown_square: (0, 0) }
    }
}

impl VisualizationsUI {
    fn visualization_list(&mut self, ui: &mut Ui) {
        for (index, visualization) in self.visualizations.iter().enumerate() {
            let frame_style = Style::default();
            let is_current = Some(index) == self.shown_visualization_index;
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
                ui.add(
                Image::new(include_image!("../resources/visualization.svg"))
                    .rounding(5.0).fit_to_exact_size(Vec2 { x: 30.0, y: 30.0 })
                );

                ui.horizontal_centered(|ui| {
                    ui.label(visualization.lock().unwrap().name.as_str());
                    
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui|{
                        if visualization.lock().unwrap().is_calculating {
                            ui.spinner();
                        }
                    });
                });;
            });

            let response = frame.allocate_space(ui).on_hover_cursor(egui::CursorIcon::PointingHand).interact(Sense::click());
            if response.clicked() {
                self.shown_visualization_index = Some(index);
            }

            if response.hovered() {
                frame.frame.fill = Color32::WHITE;
            }

            response.context_menu(|ui| {
                if ui.button("Save to file").clicked() {
                    let files = FileDialog::new()
                        .add_filter("Serde json file with visualization structure", &["json_vis"])
                        .set_directory(".")
                        .save_file();
                    
                    if let Some(path) = files {
                        let vis = visualization.lock().unwrap();
                        let res = vis.to_file(&path);
                        if res.is_err() {
                            println!("{}", res.err().unwrap());
                        }

                    }

                    ui.close_menu();
                }
            });

            frame.paint(ui);
        }
    }

    pub fn show(&mut self, ui: &mut Ui, maps: Vec<SOMParams>, datasets: &Vec<Arc<Mutex<DataSet>>>) {
        ui.painter().rect_filled(ui.max_rect(), Rounding::ZERO, Color32::WHITE);
        let modal = Modal::new(ui.ctx(), "visualization modal");
        
        if let Some(index) = self.shown_visualization_index {
            SidePanel::right("visualization_preview")
            .show_inside(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    let shown_visualization = &self.visualizations[index];
                    if shown_visualization.lock().unwrap().data.len() == 0 {
                        return ;
                    }

                    if shown_visualization.lock().unwrap().data[0].len() == 0 {
                        return ;
                    } 

                    let available_size = ui.available_size();
                    let (response, painter) = ui.allocate_painter(
                        egui::Vec2::new(
                            available_size.x - 10.0,
                            available_size.x - 10.0,
                        ),
                        egui::Sense::hover(),
                    );
                    let available_rect = response.rect;
                    let mut rects = vec![];
        
                    let n = shown_visualization.lock().unwrap().data.len();
                    let m = shown_visualization.lock().unwrap().data[0].len();
                    let i_step = available_rect.width() / (n as f32);
                    let j_step = available_rect.height() / (m as f32);
        
                    let max_val = shown_visualization.lock().unwrap()
                        .data.iter().flatten().max_by(|a, b| a.total_cmp(b)).unwrap().clone();

                    let brightness = shown_visualization.lock().unwrap().data.clone();
                    let mut lines_to_display = vec![];
        
                    for i in 0..n {
                        for j in 0..m {
                            let i_f = i as f32;
                            let j_f = j as f32;
                            let cur_rect = Rect {
                                min: Pos2 {x: i_f * i_step, y: j_f * j_step}, 
                                max: Pos2 { x: (i_f + 1.0) * i_step, y: (j_f + 1.0) * j_step }
                            }.translate(response.rect.min.to_vec2());
        
                            if self.current_shown_square == (i, j) {
                                lines_to_display = shown_visualization.lock().unwrap().word_clusters[i][j].clone();
                                rects.push(Shape::Rect(RectShape::new(cur_rect, Rounding::ZERO, Color32::RED, Stroke::new(1.0, Color32::BLACK))));
                            }
                            else if ui.rect_contains_pointer(cur_rect) {

                                if response.interact(Sense::click()).clicked() {
                                    self.current_shown_square = (i, j);
                                }

                                rects.push(Shape::Rect(RectShape::new(cur_rect, Rounding::ZERO, Color32::DARK_GREEN, Stroke::new(1.0, Color32::BLACK))));
                            }
                            else {
                                if brightness[i][j] == 0.0 {
                                    rects.push(Shape::Rect(RectShape::new(cur_rect, Rounding::ZERO, 
                                        Color32::GRAY, 
                                        Stroke::new(1.0, Color32::BLACK))));
                                }
                                else {
                                    rects.push(Shape::Rect(RectShape::new(cur_rect, Rounding::ZERO, 
                                        Color32::BLUE.gamma_multiply(brightness[i][j] / max_val), 
                                        Stroke::new(1.0, Color32::BLACK))));
                                }
                            }

                        }
                    }
        
                    painter.extend(rects);
                    if lines_to_display.len() == 0 {
                        ui.label("No texts in the cluster");
                    }
                    else {
                        ui.label(format!("{} Texts in chosen cluster: ", lines_to_display.len()));
                        for line in lines_to_display {
                            // println!("{:?}, {:?}", ui.available_size(), available_size);
                            
                            let response = ui.add(Label::new(&line).truncate(true));
                            // response.on_hover_text(&line); 
                            // println!("{:?}", response.rect);
                            
                        }
                    }

                    ui.separator();
                });
            });
        }


        ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                if self.visualizations.len() == 0 {
                    ui.label("No visualizations loaded");
                    // ui.label("No datasets loaded");
                }
                self.visualization_list(ui);
                
                modal.show(|ui| {
                    modal.title(ui, "Choose the parameters for the Visualization");
                    modal.frame(ui, |ui| {
                        Grid::new("Params")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .with_row_color(|row_index, _style| {
                            if row_index % 2 == 0 {
                                Some(Color32::from_rgb(200, 200, 200))
                            } else {
                                None 
                            }
                        })
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.current_visualization.name);
                            ui.end_row();

                            let mut cur_dataset_label = "".to_owned();
                            if let Some(dataset_index) = self.chosen_dataset_index {
                                cur_dataset_label = datasets[dataset_index].lock().unwrap().name.clone();
                            }
                            ui.label("Dataset for evaluation:");
                            ComboBox::from_id_source("Dataset")
                            .selected_text(cur_dataset_label)
                            .show_ui(ui, |ui| {
                                for (index, dataset) in datasets.iter().enumerate() {
                                    let locked_dataset = dataset.lock().unwrap();
                                    if locked_dataset.is_processed() {
                                        ui.selectable_value(&mut self.chosen_dataset_index, 
                                            Some(index), locked_dataset.name.as_str());
                                    }
                                }
                            });
                            ui.end_row();
                            
                            let mut cur_map_label = "".to_owned();
                            if let Some(map_index) = self.chosen_map_index {
                                cur_map_label = maps[map_index].name.clone();
                            }
                            ui.label("Map to evaluate:");
                            ComboBox::from_id_source("Map")
                            .selected_text(cur_map_label)
                            .show_ui(ui, |ui| {
                                for (index, map) in maps.iter().enumerate() {
                                    if map.map_weights.is_some() {
                                        ui.selectable_value(&mut self.chosen_map_index, 
                                            Some(index), map.name.as_str());
                                    }
                                }
                            });
                            ui.end_row();
                        });
                    });

                    modal.buttons(ui, |ui| {
                        modal.button(ui, "Cancel");

                        // ToDo: Implement Visualization creation and finally decide how to share Vecs' elements across tabs
                        if modal.button(ui, "Create").clicked() {
                            let visualization = Arc::new(Mutex::new(self.current_visualization.clone()));
                            self.visualizations.push(visualization.clone());
                            println!("The stuff with stuff: {:?} {:?}", self.chosen_dataset_index, self.chosen_map_index);

                            let map = maps[self.chosen_map_index.unwrap()].map_weights.as_ref().unwrap().lock().unwrap().clone();
                            let dataset = datasets[self.chosen_dataset_index.unwrap()].lock().unwrap().clone();

                            // ToDo: Add progress tracking and maybe thread termination
                            let handle = std::thread::spawn(move || {
                                visualization.lock().unwrap().is_calculating = true;
                                calculate_visualization_data(visualization.clone(), map, dataset);

                                visualization.lock().unwrap().is_calculating = false;
                                
                            });
                            self.shown_visualization_index = Some(self.visualizations.len() - 1);
                        }
                    }); 
                });
                
                if ui.button("Create a new visualization").clicked() {
                    self.current_visualization = Visualization::default();

                    modal.open();
                }

                if ui.button("Load a visualization from file").clicked() {
                    self.current_visualization = Visualization::default();

                    let files = FileDialog::new()
                            .add_filter("Serde json file with visualization structure", &["json_vis"])
                            .set_directory(".")
                            .pick_file();
                        
                    if let Some(path) = files {
                        let res = Visualization::from_file(&path);
                        if res.is_err() {
                            println!("{}", res.err().unwrap());
                        }
                        else {
                            self.visualizations.push(Arc::new(Mutex::new(res.unwrap())));
                        }
                    }
                }
            });
        });

    }

}


use std::sync::{Arc, Mutex};

use egui::{include_image, Color32, ComboBox, DragValue, Frame, Grid, Image, Rounding, ScrollArea, Sense, SidePanel, Stroke, Style, Ui, Vec2};

use crate::DataSet;
use egui_modal::{Modal};

#[derive(Debug, Clone)]
pub struct SOMParams {
    name: String,
    n: usize,
    m: usize,
    map_input_size: usize,
    a: f32,
    b: f32,
    gamma: f32,

    train_iterations: usize,
    learning_rate_base: f32,
    gauss_width_squared_base: f32,
    time_constant: f32,
}

impl Default for SOMParams {
    fn default() -> Self {
        Self {
            name: "Name".to_owned(),
            n: 10,
            m: 10,
            map_input_size: 1,

            a: 1.0,
            b: 1.0,
            gamma: 0.5,

            train_iterations: 100,
            learning_rate_base: 0.1,
            gauss_width_squared_base: 10000.0,
            time_constant: 200.0,
        }
    }
}

#[derive(Debug)]
pub struct MapsUI {
    current_params: SOMParams,
    maps: Vec<SOMParams>,
    shown_map_index: Option<usize>,
}

impl Default for MapsUI {
    fn default() -> Self {
        Self { current_params: SOMParams::default(), maps: vec![], shown_map_index: None }
    }
}

impl MapsUI {
    fn map_list(&mut self, ui: &mut Ui) {
        for (index, map) in self.maps.iter().enumerate() {
            let frame_style = Style::default();
            let is_current = Some(index) == self.shown_map_index;
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
                Image::new(include_image!("../resources/map.svg"))
                    .rounding(5.0).fit_to_exact_size(Vec2 { x: 30.0, y: 30.0 })
                );

                ui.centered_and_justified(|ui| {
                    ui.label(map.name.as_str());
                });
            });

            let response = frame.allocate_space(ui).on_hover_cursor(egui::CursorIcon::PointingHand).interact(Sense::click());
            if response.clicked() {
                self.shown_map_index = Some(index);
            }

            if response.hovered() {
                frame.frame.fill = Color32::WHITE;
            }
            frame.paint(ui);
        }
    }

    pub fn show(&mut self, ui: &mut Ui, datasets: &Vec<Arc<Mutex<DataSet>>>) {
        ui.painter().rect_filled(ui.max_rect(), Rounding::ZERO, Color32::WHITE);
        let modal = Modal::new(ui.ctx(), "map modal");

        if let Some(ind) = self.shown_map_index {
            SidePanel::right("tooltip_maps")
            .show_inside(ui, |ui| {
                let chosen_map = &mut self.maps[ind];
                ui.text_edit_singleline(&mut chosen_map.name);
                
                Grid::new("Parameters").show(ui, |ui| {
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

                    ui.label("Dataset to fit:");
                    ComboBox::from_id_source("Dataset selection").show_ui(ui, |ui| {
                        // ToDo: decide how to pass the dataset to this function
                        // Maybe a Box or Cell? It's kinda infuriating to work with references stored in structs
                        // Or I could switch to Rc<RefCell>
                    });
                    ui.end_row();
                });

                if ui.button("Fit the map").clicked() {
                    // ToDo: the actual training
                    
                }

                ui.separator();
            });
        }

        ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                if self.maps.len() == 0 {
                    ui.label("MAP ALL THE MAPS");
                    // ui.label("No datasets loaded");
                }
                self.map_list(ui);
                
                modal.show(|ui| {
                    modal.title(ui, "Choose the parameters for the Map");
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
                            ui.label("Map name:");
                            ui.text_edit_singleline(&mut self.current_params.name);
                            ui.end_row();

                            ui.label("n:");
                            ui.add(DragValue::new(&mut self.current_params.n));
                            ui.end_row();

                            ui.label("m:");
                            ui.add(DragValue::new(&mut self.current_params.m));
                            ui.end_row();

                            ui.label("Map input length:");
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
                        });
                    });

                    modal.buttons(ui, |ui| {
                        // After clicking, the modal is automatically closed
                        modal.button(ui, "Cancel");

                        // ToDo: Implement Map creation
                        if modal.button(ui, "Create").clicked() {
                            self.maps.push(self.current_params.clone());
                        }
                    }); 
                });
                
                if ui.button("Create a new map").clicked() {
                    self.current_params = SOMParams::default();

                    modal.open();
                }
            });
        });

    }
}

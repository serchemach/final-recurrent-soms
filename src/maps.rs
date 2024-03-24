use egui::{Color32, DragValue, Grid, Rounding, ScrollArea, Ui};

use crate::DataSet;
use egui_modal::{Modal};

#[derive(Debug)]
pub struct SOMParams {
    n: usize,
    m: usize,
    map_input_size: usize,
    a: f32,
    b: f32,
    gamma: f32,
}

impl Default for SOMParams {
    fn default() -> Self {
        Self {
            n: 10,
            m: 10,
            map_input_size: 1,
            a: 1.0,
            b: 1.0,
            gamma: 0.5,
        }
    }
}

#[derive(Debug)]
pub struct MapsUI {
    current_params: SOMParams
}

impl Default for MapsUI {
    fn default() -> Self {
        Self { current_params: SOMParams::default() }
    }
}

impl MapsUI {
    pub fn show(&mut self, ui: &mut Ui, datasets: &Vec<DataSet>) {
        ui.painter().rect_filled(ui.max_rect(), Rounding::ZERO, Color32::WHITE);
        let modal = Modal::new(ui.ctx(), "my_modal");

        ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("MAP ALL THE MAPS");
                
                modal.show(|ui| {
                    // these helper functions help set the ui based on the modal's
                    // set style, but they are not required and you can put whatever
                    // ui you want inside [`.show()`]
                    modal.title(ui, "Choose the parameters for the Map");
                    modal.frame(ui, |ui| {
                        Grid::new("Params")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .with_row_color(|row_index, _style| {
                            // Example: Color even rows differently
                            if row_index % 2 == 0 {
                                Some(Color32::from_rgb(200, 200, 200)) // Light gray for even rows
                            } else {
                                None // Default color for odd rows
                            }
                        })
                        .show(ui, |ui| {
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

                        }
                    }); 
                });
        
                if ui.button("Create a new map").clicked() {
                    // Show the modal
                    modal.open();
                }
            });
        });
        // What goes inside the modal

    }
}

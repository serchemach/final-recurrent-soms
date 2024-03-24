use egui::{Color32, Rounding, Ui};

use crate::DataSet;
use egui_modal::{Modal};


#[derive(Debug)]
pub struct MapsUI {

}

impl Default for MapsUI {
    fn default() -> Self {
        Self {  }
    }
}

impl MapsUI {
    pub fn show(&mut self, ui: &mut Ui, datasets: &Vec<DataSet>) {
        ui.painter().rect_filled(ui.max_rect(), Rounding::ZERO, Color32::WHITE);


        let modal = Modal::new(ui.ctx(), "my_modal");

        // What goes inside the modal
        modal.show(|ui| {
            // these helper functions help set the ui based on the modal's
            // set style, but they are not required and you can put whatever
            // ui you want inside [`.show()`]
            modal.title(ui, "Hello world!");
            modal.frame(ui, |ui| {
                modal.body(ui, "This is a modal.");
            });
            modal.buttons(ui, |ui| {
                // After clicking, the modal is automatically closed
                if modal.button(ui, "close").clicked() {
                    println!("Hello world!")
                };
            }); 
        });

        if ui.button("Open the modal").clicked() {
            // Show the modal
            modal.open();
        }



        ui.label("MAP ALL THE MAPS");

    }
}

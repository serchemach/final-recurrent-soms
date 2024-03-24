use egui::{Color32, Rounding, Ui};

use crate::data_processing::DataSet;



#[derive(Debug)]
pub struct VisualizationsUI {

}

impl Default for VisualizationsUI {
    fn default() -> Self {
        Self {  }
    }
}

impl VisualizationsUI {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.painter().rect_filled(ui.max_rect(), Rounding::ZERO, Color32::WHITE);

    }
}


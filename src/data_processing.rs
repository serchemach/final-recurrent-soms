use egui::{Color32, Rounding, Ui};


#[derive(Debug)]
pub struct DataProcessingUI {

}

impl Default for DataProcessingUI {
    fn default() -> Self {
        Self {  }
    }
}

impl DataProcessingUI {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.painter().rect_filled(ui.max_rect(), Rounding::ZERO, Color32::WHITE);
        ui.label("PROCESS ALL THE DATA");
    }
}

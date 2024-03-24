use egui::{Color32, Rounding, Ui};


#[derive(Debug)]
pub struct MapsUI {

}

impl Default for MapsUI {
    fn default() -> Self {
        Self {  }
    }
}

impl MapsUI {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.painter().rect_filled(ui.max_rect(), Rounding::ZERO, Color32::RED);
        ui.label("MAP ALL THE MAPS");
    }
}

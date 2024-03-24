use egui::{epaint::RectShape, include_image, Color32, Pos2, Rect, Rounding, Shape, SidePanel, Stroke, Ui};

use crate::data_processing::DataSet;
use std::cmp::{max, min};


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

        SidePanel::right("visualization_preview")
        .show_inside(ui, |ui| {
            // ui.image(include_image!("../resources/sample_image.png"));
            let available_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(
                egui::Vec2::new(
                    available_size.x - 10.0,
                    available_size.x - 10.0,
                ), // Allocate a space for the container
                egui::Sense::hover(),
            );
            let available_rect = ui.available_rect_before_wrap();
            let mut rects = vec![];

            let n = 100;
            let m = 100;
            let i_step = (available_size.x - 10.0) / (n as f32);
            let j_step = (available_size.x - 10.0) / (m as f32);

            for i in 0..n {
                for j in 0..m {
                    // rects.push(Rect::)
                    let i = i as f32;
                    let j = j as f32;
                    let cur_rect = Rect {
                        min: Pos2 {x: i * i_step, y: j * j_step}, 
                        max: Pos2 { x: (i + 1.0) * i_step, y: (j + 1.0) * j_step }
                    }.translate(response.rect.min.to_vec2());

                    if ui.rect_contains_pointer(cur_rect) {
                        rects.push(Shape::Rect(RectShape::new(cur_rect, Rounding::ZERO, Color32::RED, Stroke::NONE)));
                    }
                    else {
                        rects.push(Shape::Rect(RectShape::new(cur_rect, Rounding::ZERO, Color32::BLUE, Stroke::NONE)));
                    }
                }
            }

            painter.extend(rects);
            ui.separator();
        });
    }
}


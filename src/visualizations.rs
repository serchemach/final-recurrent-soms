use egui::{epaint::RectShape, include_image, Color32, ComboBox, DragValue, Frame, Grid, Image, Pos2, Rect, Rounding, ScrollArea, Sense, Shape, SidePanel, Stroke, Style, Ui, Vec2};
use egui_modal::Modal;

use crate::data_processing::DataSet;
use std::cmp::{max, min};

#[derive(Debug, Clone)]
pub struct Visualization {
    name: String,
}

impl Default for Visualization {
    fn default() -> Self {
        Self { name: "Name".to_owned() }
    }
}

#[derive(Debug)]
pub struct VisualizationsUI {
    visualizations: Vec<Visualization>,
    shown_visualization_index: Option<usize>,
    current_visualization: Visualization,
}

impl Default for VisualizationsUI {
    fn default() -> Self {
        Self { visualizations: vec![], shown_visualization_index: None, current_visualization: Visualization::default() }
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

                ui.centered_and_justified(|ui| {
                    ui.label(visualization.name.as_str());
                });
            });

            let response = frame.allocate_space(ui).on_hover_cursor(egui::CursorIcon::PointingHand).interact(Sense::click());
            if response.clicked() {
                self.shown_visualization_index = Some(index);
            }

            if response.hovered() {
                frame.frame.fill = Color32::WHITE;
            }
            frame.paint(ui);
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.painter().rect_filled(ui.max_rect(), Rounding::ZERO, Color32::WHITE);
        let modal = Modal::new(ui.ctx(), "visualization modal");

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
            let available_rect = response.rect;
            let mut rects = vec![];

            let n = 10;
            let m = 10;
            let i_step = available_rect.width() / (n as f32);
            let j_step = available_rect.height() / (m as f32);

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
                        rects.push(Shape::Rect(RectShape::new(cur_rect, Rounding::ZERO, Color32::RED, Stroke::new(1.0, Color32::BLACK))));
                    }
                    else {
                        rects.push(Shape::Rect(RectShape::new(cur_rect, Rounding::ZERO, Color32::BLUE, Stroke::new(1.0, Color32::BLACK))));
                    }
                }
            }

            painter.extend(rects);
            ui.separator();
        });


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

                            ui.label("Dataset for evaluation:");
                            ComboBox::from_id_source("Dataset").show_ui(ui, |ui| {

                            });
                            ui.end_row();
                            
                            ui.label("Map to evaluate:");
                            ComboBox::from_id_source("Map").show_ui(ui, |ui| {
                                
                            });
                            ui.end_row();
                        });
                    });

                    modal.buttons(ui, |ui| {
                        modal.button(ui, "Cancel");

                        // ToDo: Implement Visualization creation and finally decide how to share Vecs' elements across tabs
                        if modal.button(ui, "Create").clicked() {
                            self.visualizations.push(self.current_visualization.clone())
                        }
                    }); 
                });
                
                if ui.button("Create a new map").clicked() {
                    self.current_visualization = Visualization::default();

                    modal.open();
                }
            });
        });

    }

}


#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod data_processing;
mod maps;
mod visualizations;

use data_processing::{*};
use env_logger::fmt::style::Color;
use maps::{*};
use visualizations::{*};

use egui::{Button, Color32, Id, Sense};

const DATA_PROCESSING_SAVE_PATH: &str = "./data/dp.sv";
const MAPS_SAVE_PATH: &str = "./data/mp.sv";
const VISUALIZATIONS_SAVE_PATH: &str = "./data/vz.sv";

#[derive(Debug)]
enum PaneType {
    DataProcessing(DataProcessingState),
    Maps(MapsState),
    Visualizations(VisualizationsState)
}

struct Pane {
    p_type: PaneType,
}

struct TreeBehavior {}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        format!("Pane {:?}", pane.p_type).into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        // Give each pane a unique color:
        // let color = egui::epaint::Hsva::new(0.103 * pane.nr as f32, 0.5, 0.5, 1.0);
        let color = Color32::RED;
        ui.painter().rect_filled(ui.max_rect(), 0.0, color);

        ui.label(format!("The contents of pane {:?}.", pane.p_type));

        // You can make your pane draggable like so:
        if ui.interact(ui.available_rect_before_wrap(), egui::Id::new(_tile_id), Sense::drag())
        .dragged() {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([520.0, 240.0]),
        ..Default::default()
    };

    let mut tree = create_tree();

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut behavior = TreeBehavior {};
            tree.ui(&mut behavior, ui);
        });
    })
}

fn create_tree() -> egui_tiles::Tree<Pane> {

    let mut tiles = egui_tiles::Tiles::default();

    let mut tabs = vec![];
    tabs.push(tiles.insert_pane(Pane { p_type: PaneType::DataProcessing(DataProcessingState::default()) }));
    tabs.push(tiles.insert_pane(Pane { p_type: PaneType::Maps(MapsState::default()) }));
    tabs.push(tiles.insert_pane(Pane { p_type: PaneType::Visualizations(VisualizationsState::default()) }));

    let root = tiles.insert_tab_tile(tabs);

    egui_tiles::Tree::new("my_tree", root, tiles)
}

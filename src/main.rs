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
    DataProcessing(DataProcessingUI),
    Maps(MapsUI),
    Visualizations(VisualizationsUI)
}

struct Pane {
    p_type: PaneType,
}

struct TreeBehavior {}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane.p_type {
            PaneType::DataProcessing(_) => "Data Processing",
            PaneType::Maps(_) => "Maps",
            PaneType::Visualizations(_) => "Visualizations",
        }.into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {

        match &mut pane.p_type {
            PaneType::DataProcessing(state) => state.show(ui),
            PaneType::Maps(state) => state.show(ui),
            PaneType::Visualizations(state) => state.show(ui),
        }

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
        egui_extras::install_image_loaders(ctx);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut behavior = TreeBehavior {};
            tree.ui(&mut behavior, ui);
        });
    })
}

fn create_tree() -> egui_tiles::Tree<Pane> {

    let mut tiles = egui_tiles::Tiles::default();

    let mut tabs = vec![];
    tabs.push(tiles.insert_pane(Pane { p_type: PaneType::DataProcessing(DataProcessingUI::default()) }));
    tabs.push(tiles.insert_pane(Pane { p_type: PaneType::Maps(MapsUI::default()) }));
    tabs.push(tiles.insert_pane(Pane { p_type: PaneType::Visualizations(VisualizationsUI::default()) }));

    let root = tiles.insert_tab_tile(tabs);

    egui_tiles::Tree::new("my_tree", root, tiles)
}

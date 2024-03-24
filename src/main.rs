#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod data_processing;
mod maps;
mod visualizations;

use std::iter::Map;

use data_processing::{*};
use env_logger::fmt::style::Color;
use maps::{*};
use visualizations::{*};

use egui::{Button, Color32, Id, ScrollArea, Sense};

const DATA_PROCESSING_SAVE_PATH: &str = "./data/dp.sv";
const MAPS_SAVE_PATH: &str = "./data/mp.sv";
const VISUALIZATIONS_SAVE_PATH: &str = "./data/vz.sv";

#[derive(Debug)]
enum PaneType {
    DataProcessing,
    Maps,
    Visualizations
}

struct Pane {
    p_type: PaneType,
}

struct TreeBehavior {
    data_processing_state: DataProcessingUI,
    maps_state: MapsUI,
    visualizations_state: VisualizationsUI,
}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane.p_type {
            PaneType::DataProcessing => "Data Processing",
            PaneType::Maps => "Maps",
            PaneType::Visualizations => "Visualizations",
        }.into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        

        match &mut pane.p_type {
            PaneType::DataProcessing => self.data_processing_state.show(ui),
            PaneType::Maps => self.maps_state.show(ui, &self.data_processing_state.datasets),
            PaneType::Visualizations => self.visualizations_state.show(ui),
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

    let mut behavior = TreeBehavior {
        maps_state: MapsUI::default(),
        data_processing_state: DataProcessingUI::default(),
        visualizations_state: VisualizationsUI::default(),
    };
    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui_extras::install_image_loaders(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            tree.ui(&mut behavior, ui);
        });
    })
}

fn create_tree() -> egui_tiles::Tree<Pane> {

    let mut tiles = egui_tiles::Tiles::default();

    let mut tabs = vec![];
    tabs.push(tiles.insert_pane(Pane { p_type: PaneType::DataProcessing }));
    tabs.push(tiles.insert_pane(Pane { p_type: PaneType::Maps }));
    tabs.push(tiles.insert_pane(Pane { p_type: PaneType::Visualizations }));

    let root = tiles.insert_tab_tile(tabs);

    egui_tiles::Tree::new("my_tree", root, tiles)
}

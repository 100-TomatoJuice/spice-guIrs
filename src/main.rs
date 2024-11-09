use app::SpiceGuIrsApp;

mod app;
mod circuit;
mod element_pointer;
mod element_properties;
mod menu_bar;
mod terminal;
pub mod utils;

const GRID_SIZE: i32 = 16;

const GROUND_SOURCE: egui::ImageSource = egui::include_image!("../assets/ground.png");
const DC_VOLTAGE_SOURCE: egui::ImageSource =
    egui::include_image!("../assets/dc_voltage_source.png");
const DC_CURRENT_SOURCE: egui::ImageSource =
    egui::include_image!("../assets/dc_current_source.png");
const RESISTOR_SOURCE: egui::ImageSource = egui::include_image!("../assets/resistor.png");
const CAPACITOR_SOURCE: egui::ImageSource = egui::include_image!("../assets/capacitor.png");
const INDUCTOR_SOURCE: egui::ImageSource = egui::include_image!("../assets/inductor.png");

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_decorations(true),
        ..Default::default()
    };
    eframe::run_native(
        "Spice GuIrs",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(SpiceGuIrsApp::new()))
        }),
    )
}

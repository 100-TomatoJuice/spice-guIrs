use crate::{
    circuit::{GuiCircuit, ToPlaceElement},
    element_pointer::handle_elements,
    element_properties::handle_selected_object_properties,
    menu_bar::menu_bar,
    terminal::handle_terminal,
    utils::ipos2::IPos2,
};

pub struct SpiceGuIrsApp {
    pub gui_circuit: GuiCircuit,
    pub to_place_element: Option<ToPlaceElement>,
    pub selected_element: Option<u32>,
    pub selected_node: Option<IPos2>,
    pub drag_data: Option<DragData>,
    pub terminal_lines: Vec<String>,
}

impl SpiceGuIrsApp {
    pub fn new() -> Self {
        Self {
            gui_circuit: GuiCircuit::default(),
            to_place_element: None,
            selected_element: None,
            selected_node: None,
            drag_data: None,
            terminal_lines: vec![],
        }
    }
}

impl eframe::App for SpiceGuIrsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        menu_bar(self, ctx);
        handle_terminal(self, ctx);
        handle_elements(self, ctx);
        handle_selected_object_properties(self, ctx);
    }
}

#[derive(Clone, Copy)]
pub struct DragData {
    pub start_position: IPos2,
    pub end_position: Option<IPos2>,
    pub x_first: bool,
}

impl DragData {
    pub fn new(start_position: IPos2) -> Self {
        Self {
            start_position,
            end_position: None,
            x_first: false,
        }
    }
}

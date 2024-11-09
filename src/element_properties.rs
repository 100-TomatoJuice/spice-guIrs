use egui::{Align2, Context, DragValue, Vec2};

use crate::app::SpiceGuIrsApp;

pub fn handle_selected_object_properties(app: &mut SpiceGuIrsApp, ctx: &Context) {
    handle_selected_element_properties(app, ctx);
    handle_selected_wire_properties(app, ctx);
}

fn handle_selected_element_properties(app: &mut SpiceGuIrsApp, ctx: &Context) {
    let Some(selected_index) = app.selected_element else {
        return;
    };
    let Some(selected_element) = app
        .gui_circuit
        .gui_elements
        .get_mut(&selected_index)
        .map(|x| &mut x.element)
    else {
        return;
    };

    let name = format!("{} {}", selected_element.display_name(), "Properties");
    let Some(element_unit) = selected_element.display_unit_name().to_owned() else {
        return;
    };
    let Some(element_unit_symbol) = selected_element.display_unit_symbol().to_owned() else {
        return;
    };
    let Some(element_value) = selected_element.value_mut() else {
        return;
    };

    let mut open = true;

    egui::Window::new(name)
        .movable(false)
        .collapsible(false)
        .vscroll(false)
        .resizable(false)
        .constrain_to(ctx.available_rect())
        .anchor(Align2::RIGHT_BOTTOM, Vec2::ZERO)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(element_unit);
                ui.add(DragValue::new(element_value).range(0..=i32::MAX));
                ui.label(element_unit_symbol);
            })
        });

    if !open {
        app.selected_element = None;
    }
}

fn handle_selected_wire_properties(app: &mut SpiceGuIrsApp, ctx: &Context) {
    let Some(selected_position) = app.selected_node else {
        return;
    };

    let name = format!("Wire");

    let mut open = true;

    egui::Window::new(name)
        .movable(false)
        .collapsible(false)
        .vscroll(false)
        .resizable(false)
        .constrain_to(ctx.available_rect())
        .anchor(Align2::RIGHT_BOTTOM, Vec2::ZERO)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(selected_position.to_pos2().to_string());
            })
        });

    if !open {
        app.selected_node = None;
    }
}

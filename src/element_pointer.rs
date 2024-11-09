use core::f32;

use egui::{
    Button, Color32, Context, ImageButton, Key, PointerButton, Pos2, Rect, Sense, Shape, Stroke,
    Ui, Vec2,
};

use crate::{
    app::{DragData, SpiceGuIrsApp},
    circuit::GuiElement,
    utils::ipos2::{IPos2, Pos2Ext},
    GRID_SIZE,
};

pub fn handle_elements(app: &mut SpiceGuIrsApp, ctx: &Context) {
    egui::CentralPanel::default().show(&ctx, |ui| {
        display_circuit_elements(app, ui);
        display_to_place_element(app, ui);
        display_dragged_wire(app, ui);
        display_wire_drag_points(app, ui);
        display_placed_wire(app, ui);
        place_element(app, ui);
        place_wires(app);
    });

    rotate_to_place_element(app, ctx);
    deselect_element(app, ctx);
    reset_dragged_node(app, ctx);
    delete_selected_element(app, ctx);
    delete_selected_node(app, ctx);
}

fn display_to_place_element(app: &SpiceGuIrsApp, ui: &mut Ui) {
    let Some(element_image) = app.to_place_element.as_ref().map(|x| x.image.clone()) else {
        return;
    };

    let pointer_position = ui
        .ctx()
        .pointer_latest_pos()
        .unwrap_or_default()
        .to_ipos2(GRID_SIZE)
        .to_pos2();

    let rect = Rect::from_center_size(
        pointer_position,
        element_image.calc_size(Vec2::new(128.0, 62.5), element_image.size()),
    );

    ui.put(rect, element_image);
}

fn display_circuit_elements(app: &mut SpiceGuIrsApp, ui: &mut Ui) {
    for (index, gui_element) in app.gui_circuit.gui_elements.iter() {
        let response = ui.put(
            gui_element.rect,
            ImageButton::new(gui_element.image.clone()).frame(false),
        );

        if response.clicked_by(PointerButton::Primary) {
            app.selected_element = Some(*index);
            app.selected_node = None;
            println!("Selected {}", index);
        }

        for node_position in gui_element.nodes.iter() {
            let rect = Rect::from_center_size(node_position.to_pos2(), Vec2::splat(10.0));
            let response = ui
                .put(rect, Button::new("").fill(Color32::from_white_alpha(0)))
                .interact(Sense::click_and_drag());

            if response.drag_started_by(PointerButton::Primary) {
                app.drag_data = Some(DragData::new(rect.center().to_ipos2(GRID_SIZE)));
            }

            if response.contains_pointer() && !response.drag_stopped_by(PointerButton::Primary) {
                response.highlight();

                let mut drag_stopped = false;
                ui.input(|input| {
                    drag_stopped = input.pointer.button_released(PointerButton::Primary)
                });
                if drag_stopped {
                    if let Some(drag_data) = &mut app.drag_data {
                        drag_data.end_position = Some(rect.center().to_ipos2(GRID_SIZE));
                    }
                }
            }
        }
    }
}

fn display_dragged_wire(app: &mut SpiceGuIrsApp, ui: &mut Ui) {
    let Some(drag_data) = &mut app.drag_data else {
        return;
    };

    let start = drag_data.start_position.to_pos2();
    let end = ui
        .ctx()
        .pointer_latest_pos()
        .unwrap_or_default()
        .to_ipos2(GRID_SIZE)
        .to_pos2();
    let mut mouse_delta = Vec2::default();
    ui.input(|input| {
        mouse_delta = input.pointer.delta();
    });

    if start.distance_sq(end) < 1000.0 && mouse_delta.length() > 1.0 {
        drag_data.x_first = mouse_delta.y.abs() < mouse_delta.x.abs();
    }

    let middle_position = match drag_data.x_first {
        true => Pos2::new(end.x, start.y),
        false => Pos2::new(start.x, end.y),
    };
    let stroke = Stroke::new(2.0, Color32::BLUE);

    ui.painter()
        .add(Shape::line(vec![start, middle_position, end], stroke));
}

fn display_wire_drag_points(app: &mut SpiceGuIrsApp, ui: &mut Ui) {
    for group in app.gui_circuit.node_groups.iter() {
        for position in group.iter() {
            if *position % GRID_SIZE != IPos2::ZERO {
                continue;
            }

            let rect = Rect::from_center_size(position.to_pos2(), Vec2::splat(10.0));
            let response = ui
                .put(rect, Button::new("").fill(Color32::from_white_alpha(0)))
                .interact(Sense::click_and_drag());

            if response.clicked_by(PointerButton::Primary) {
                app.selected_node = Some(*position);
                app.selected_element = None;
            }

            if response.drag_started_by(PointerButton::Primary) {
                app.drag_data = Some(DragData::new(rect.center().to_ipos2(GRID_SIZE)));
            }

            if response.contains_pointer() && !response.drag_stopped_by(PointerButton::Primary) {
                response.highlight();

                let mut drag_stopped = false;
                ui.input(|input| {
                    drag_stopped = input.pointer.button_released(PointerButton::Primary)
                });
                if drag_stopped {
                    if let Some(drag_data) = &mut app.drag_data {
                        drag_data.end_position = Some(rect.center().to_ipos2(GRID_SIZE));
                    }
                }
            }
        }
    }
}

fn display_placed_wire(app: &mut SpiceGuIrsApp, ui: &mut Ui) {
    let stroke = Stroke::new(2.0, Color32::WHITE);

    for wire in app.gui_circuit.rendered_wires.iter() {
        ui.painter().add(Shape::line(wire.clone(), stroke));
    }
}

fn place_wires(app: &mut SpiceGuIrsApp) {
    let Some(drag_data) = app.drag_data else {
        return;
    };
    let Some(end_position) = drag_data.end_position else {
        return;
    };

    app.gui_circuit
        .add_orthogonal_wires(drag_data.start_position, end_position, drag_data.x_first);
}

fn place_element(app: &mut SpiceGuIrsApp, ui: &mut Ui) {
    let Some(selected_element) = &app.to_place_element else {
        return;
    };
    if !ui.ui_contains_pointer() {
        return;
    }

    let pointer_position = ui
        .ctx()
        .pointer_latest_pos()
        .unwrap_or_default()
        .to_ipos2(GRID_SIZE)
        .to_pos2();

    ui.input(|input| {
        if input.pointer.button_released(PointerButton::Primary) {
            let size = selected_element
                .image
                .calc_size(Vec2::new(128.0, 62.5), selected_element.image.size());
            let rect = Rect::from_center_size(pointer_position, size);

            app.gui_circuit.add_element(GuiElement::new(
                selected_element.element.clone(),
                rect,
                selected_element.image.clone(),
            ));
        }
    });
}

fn rotate_to_place_element(app: &mut SpiceGuIrsApp, ctx: &Context) {
    ctx.input(|input| {
        if input.key_pressed(Key::R) {
            if let Some(to_place_element) = &mut app.to_place_element {
                let current_angle = to_place_element
                    .image
                    .image_options()
                    .rotation
                    .unwrap_or_default()
                    .0
                    .angle();
                to_place_element.image = to_place_element
                    .image
                    .clone()
                    .rotate(current_angle + f32::consts::FRAC_PI_2, Vec2::splat(0.5));
            }
        }
    })
}

fn delete_selected_element(app: &mut SpiceGuIrsApp, ctx: &Context) {
    ctx.input(|input| {
        if input.key_pressed(Key::Delete) {
            if let Some(selected_element_index) = app.selected_element {
                app.gui_circuit.remove_element(selected_element_index);
                app.selected_element = None;
                println!("Delete Element");
            }
        }
    })
}

fn delete_selected_node(app: &mut SpiceGuIrsApp, ctx: &Context) {
    ctx.input(|input| {
        if input.key_pressed(Key::Delete) {
            if let Some(selected_element_position) = app.selected_node {
                app.gui_circuit.remove_node(selected_element_position);
                app.selected_node = None;
                println!("Delete Node");
            }
        }
    })
}

fn deselect_element(app: &mut SpiceGuIrsApp, ctx: &Context) {
    ctx.input(|input| {
        if input.pointer.button_released(PointerButton::Secondary) {
            app.to_place_element = None;
        }
    });
}

fn reset_dragged_node(app: &mut SpiceGuIrsApp, ctx: &Context) {
    ctx.input(|input| {
        if input.pointer.button_released(PointerButton::Primary) {
            app.drag_data = None;
        }
    });
}

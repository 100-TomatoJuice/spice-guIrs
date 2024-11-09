use egui::{Image, ImageButton, Ui};
use spice_rs::runners::dc_op::dc_op;

use crate::{
    app::SpiceGuIrsApp,
    circuit::{ElementType, ToPlaceElement},
    CAPACITOR_SOURCE, DC_CURRENT_SOURCE, DC_VOLTAGE_SOURCE, GROUND_SOURCE, INDUCTOR_SOURCE,
    RESISTOR_SOURCE,
};

pub fn menu_bar(app: &mut SpiceGuIrsApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar")
        .default_height(50.0)
        .min_height(0.0)
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("File").clicked() {}

                ui.menu_button("Runners", |ui| {
                    if ui.button("DC Operating Point").clicked() {
                        if let Some(circuit) = app.gui_circuit.construct_circuit() {
                            match dc_op(&circuit) {
                                Ok(values) => {
                                    app.terminal_lines.push("".to_string());
                                    for (i, value) in values.iter().enumerate() {
                                        let unit = if i < circuit.node_count().saturating_sub(1) {
                                            "V"
                                        } else {
                                            "A"
                                        };

                                        let name = if i < circuit.node_count().saturating_sub(1) {
                                            format!("V{}", i + 1)
                                        } else {
                                            format!(
                                                "I{}",
                                                i - circuit.node_count().saturating_sub(2)
                                            )
                                        };

                                        app.terminal_lines
                                            .push(format!("{}: {}{}", name, value, unit));
                                    }
                                }
                                Err(error) => app
                                    .terminal_lines
                                    .push(format!("Error: {}", error.to_string())),
                            }
                        }

                        ui.close_menu();
                    }
                });
            });
        });

    egui::SidePanel::left("element_bar")
        .default_width(75.0)
        .max_width(100.0)
        .min_width(60.0)
        .show(ctx, |ui| {
            element_bar(app, ui);
        });
}

fn element_bar(app: &mut SpiceGuIrsApp, ui: &mut Ui) {
    if ui
        .add(ImageButton::new(GROUND_SOURCE).rounding(5.0))
        .on_hover_text("Ground")
        .clicked()
    {
        let element_image = Image::new(GROUND_SOURCE);
        app.to_place_element = Some(ToPlaceElement::new(ElementType::Ground, element_image));
    }

    if ui
        .add(ImageButton::new(DC_VOLTAGE_SOURCE).rounding(5.0))
        .on_hover_text("DC Voltage Source")
        .clicked()
    {
        let element_image = Image::new(DC_VOLTAGE_SOURCE);
        app.to_place_element = Some(ToPlaceElement::new(
            ElementType::DCVoltageSource(10.0),
            element_image,
        ));
    }

    if ui
        .add(ImageButton::new(DC_CURRENT_SOURCE).rounding(5.0))
        .on_hover_text("DC Current Source")
        .clicked()
    {
        let element_image = Image::new(DC_CURRENT_SOURCE);
        app.to_place_element = Some(ToPlaceElement::new(
            ElementType::DCCurrentSource(10.0),
            element_image,
        ));
    }

    if ui
        .add(ImageButton::new(CAPACITOR_SOURCE).rounding(5.0))
        .on_hover_text("Capacitor")
        .clicked()
    {
        let element_image = Image::new(CAPACITOR_SOURCE);
        app.to_place_element = Some(ToPlaceElement::new(
            ElementType::Capacitor(10.0),
            element_image,
        ));
    }

    if ui
        .add(ImageButton::new(INDUCTOR_SOURCE).rounding(5.0))
        .on_hover_text("Inductor")
        .clicked()
    {
        let element_image = Image::new(INDUCTOR_SOURCE);
        app.to_place_element = Some(ToPlaceElement::new(
            ElementType::Inductor(10.0),
            element_image,
        ));
    }

    if ui
        .add(ImageButton::new(RESISTOR_SOURCE).rounding(5.0))
        .on_hover_text("Resistor")
        .clicked()
    {
        let element_image = Image::new(RESISTOR_SOURCE);
        app.to_place_element = Some(ToPlaceElement::new(
            ElementType::Resistor(10.0),
            element_image,
        ));
    }
}

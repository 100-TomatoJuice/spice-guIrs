use egui::Context;

use crate::app::SpiceGuIrsApp;

pub fn handle_terminal(app: &mut SpiceGuIrsApp, ctx: &Context) {
    egui::TopBottomPanel::bottom("terminal")
        .resizable(true)
        .default_height(90.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                        for line in app.terminal_lines.iter() {
                            ui.label(line);
                        }
                    })
                })
        });
}

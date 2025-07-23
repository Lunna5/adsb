use crate::viewer::{Viewer, ViewerMapInfo};
use egui::{Align2, ComboBox, Image, MenuBar, RichText, Ui, Window};
use walkers::{MapMemory, sources::Attribution};

pub fn controls(
    ui: &mut Ui,
    app: &mut ViewerMapInfo,
    http_stats: Vec<walkers::HttpStats>,
    fps: f32,
) {
    ui.collapsing("Map", |ui| {
        ComboBox::from_label("Tile Provider")
            .selected_text(format!("{:?}", app.selected_provider))
            .show_ui(ui, |ui| {
                for p in app.providers.keys() {
                    ui.selectable_value(&mut app.selected_provider, *p, format!("{p:?}"));
                }
            });

        ui.checkbox(&mut app.zoom_with_ctrl_wheel, "Zoom with Ctrl");
    });

    ui.collapsing("HTTP statistics", |ui| {
        for http_stats in http_stats {
            ui.label(format!(
                "{:?} requests in progress: {}",
                app.selected_provider, http_stats.in_progress
            ));
        }
    });

    ui.collapsing("Framerate statistics", |ui| {
        ui.label(format!("FPS: {}", fps));
    });
}

pub fn zoom(ui: &mut Ui, map_memory: &mut MapMemory) {
    ui.horizontal(|ui| {
        if ui.button(RichText::new("➕").heading()).clicked() {
            let _ = map_memory.zoom_in();
        }

        if ui.button(RichText::new("➖").heading()).clicked() {
            let _ = map_memory.zoom_out();
        }
    });
}

pub fn go_to_my_position(ui: &mut Ui, map_memory: &mut MapMemory) {
    if let Some(position) = map_memory.detached() {
        Window::new("Go to my position")
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(Align2::RIGHT_BOTTOM, [-10., -10.])
            .fixed_size([500., 50.])
            .show(ui.ctx(), |ui| {
                ui.label(format!(
                    "center at {:.04} {:.04}",
                    position.x(),
                    position.y()
                ));
                if ui
                    .button(RichText::new("Go to the starting point").heading())
                    .clicked()
                {
                    map_memory.follow_my_position();
                }
            });
    }
}

pub fn can_go_to_my_position(map_memory: &MapMemory) -> bool {
    map_memory.detached().is_some()
}

pub fn menu_bar(ui: &mut Ui, app: &mut Viewer) {
    MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Quit").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        ui.menu_button("View", |ui| {
            if ui.button("Map Options").clicked() {
                let app_state_write = app.app_state.write().unwrap();
                let mut store_write = app_state_write.store.write().unwrap();

                let map_controls_open =
                    store_write.get_as_bool_or_default("viewer.windows.map_controls_open", false);
                store_write.set("viewer.windows.map_controls_open", !map_controls_open);
            }
        });
    });
}

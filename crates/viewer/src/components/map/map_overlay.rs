use crate::components::Component;
use eframe::emath::Align2;
use egui::{Area, Context, Frame, Id, Image, RichText, Ui};
use std::sync::{Arc, RwLock, TryLockError};
use walkers::MapMemory;
use walkers::sources::Attribution;

pub struct MapOverlay {
    map_memory: Arc<RwLock<MapMemory>>,
    attributions: Vec<Attribution>,
}

impl MapOverlay {
    pub fn new(map_memory: Arc<RwLock<MapMemory>>, attributions: Vec<Attribution>) -> MapOverlay {
        MapOverlay {
            map_memory,
            attributions,
        }
    }
}

impl Component for MapOverlay {
    fn draw(&mut self, ctx: &Context) {
        Area::new(Id::new("map_overlay"))
            .anchor(Align2::LEFT_BOTTOM, egui::vec2(10.0, -10.0))
            .show(ctx, |ui| {
                Frame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("➕").heading()).clicked() {
                            let mut write_guard = self.map_memory.write().unwrap();
                            write_guard.zoom_in().expect("TODO: panic message");
                        }

                        if ui.button(RichText::new("➖").heading()).clicked() {
                            let mut write_guard = self.map_memory.write().unwrap();
                            write_guard.zoom_out().expect("TODO: panic message");
                        }
                    });
                });

                Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label("map provided by");
                    for attribution in &self.attributions {
                        ui.horizontal(|ui| {
                            if let Some(logo) = &attribution.logo_light {
                                ui.add(Image::new(logo.clone()).max_height(30.0).max_width(80.0));
                            }
                            ui.hyperlink_to(attribution.text, attribution.url);
                        });
                    }
                });
            });
    }
}

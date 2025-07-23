use crate::components::Component;
use crate::tiles::{Provider, TilesKind};
use crate::{AppState, airport_plugin};
use egui::{CentralPanel, DragPanButtons, Frame, Id, TopBottomPanel, Window};
use rayon::ThreadPool;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use walkers::{Map, MapMemory, lat_lon};

pub struct ViewerMapInfo {
    pub(crate) providers: BTreeMap<Provider, Vec<TilesKind>>,
    pub selected_provider: Provider,
    pub map_memory: Arc<RwLock<MapMemory>>,
    pub zoom_with_ctrl_wheel: bool,
}

impl ViewerMapInfo {
    pub fn new(egui_ctx: egui::Context) -> Self {
        let providers = crate::tiles::providers(egui_ctx);
        let selected_provider = *providers.keys().next().unwrap_or(&Provider::OpenStreetMap);
        let map_memory = Arc::new(RwLock::new(MapMemory::default()));
        let zoom_with_ctrl_wheel = false;

        Self {
            providers,
            selected_provider,
            map_memory,
            zoom_with_ctrl_wheel,
        }
    }
}

pub struct Viewer {
    pub map_info: ViewerMapInfo,
    pub app_state: Arc<RwLock<AppState>>,
    pub components: Vec<Box<dyn Component>>,
    pub thread_pool: ThreadPool,
    pub last_frame_time: std::time::Instant,
    pub last_fps: f32,
}

impl Viewer {
    pub fn new(
        egui_ctx: egui::Context,
        app_state: Arc<RwLock<AppState>>,
        components: Vec<Box<dyn Component>>,
    ) -> Self {
        let map_info = ViewerMapInfo::new(egui_ctx);

        Self {
            map_info,
            app_state,
            components,
            thread_pool: rayon::ThreadPoolBuilder::new()
                .num_threads(8) // You can adjust the number of threads as needed
                .build()
                .expect("Failed to create thread pool"),
            last_frame_time: Instant::now(),
            last_fps: 0.0,
        }
    }

    pub fn update_fps(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_frame_time;
        self.last_fps = 1.0 / dt.as_secs_f32();
        self.last_frame_time = now;
    }

    pub fn add_component<C: Component + 'static>(&mut self, component: C) {
        self.components.push(Box::new(component));
    }
}

#[profiling::all_functions]
impl eframe::App for Viewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        profiling::scope!("Viewer::update");
        self.update_fps();

        TopBottomPanel::top(Id::new("my_top_panel")).show(ctx, |ui| {
            crate::frames::menu_bar(ui, self);
        });

        {
            let app_state_write = self.app_state.write().unwrap();
            let mut store_write = app_state_write.store.write().unwrap();

            let mut map_controls_open =
                store_write.get_as_bool_or_default("viewer.windows.map_controls_open", false);

            Window::new("Map Controls")
                .resizable(true)
                .title_bar(true)
                .collapsible(true)
                .open(&mut map_controls_open)
                .show(ctx, |ui| {
                    let tiles = self
                        .map_info
                        .providers
                        .get_mut(&self.map_info.selected_provider)
                        .unwrap();

                    let http_stats = tiles
                        .iter()
                        .filter_map(|tiles| {
                            if let crate::tiles::TilesKind::Http(tiles) = tiles {
                                Some(tiles.stats())
                            } else {
                                None
                            }
                        })
                        .collect();

                    crate::frames::controls(ui, &mut self.map_info, http_stats, self.last_fps);
                });

            store_write.set("viewer.windows.map_controls_open", map_controls_open);
        }

        CentralPanel::default().frame(Frame::NONE).show(ctx, |ui| {
            profiling::scope!("Viewer::CentralPanel");
            let gps_position = lat_lon(36.67680681137922, -4.49391784930664);

            let tiles = self
                .map_info
                .providers
                .get_mut(&self.map_info.selected_provider)
                .unwrap();

            let attributions: Vec<_> = tiles
                .iter()
                .map(|tile| tile.as_ref().attribution())
                .collect();

            {
                let mut write_guard = self.map_info.map_memory.write().unwrap();
                let mut map = Map::new(None, &mut write_guard, gps_position, &mut self.thread_pool);

                let airports = &self.app_state.read().unwrap().airports;

                map = map
                    .zoom_with_ctrl(self.map_info.zoom_with_ctrl_wheel)
                    .drag_pan_buttons(DragPanButtons::PRIMARY | DragPanButtons::SECONDARY);

                {
                    let airport_plugin =
                        airport_plugin(Arc::clone(&self.app_state.read().unwrap().airports));
                    let write_guard = airport_plugin.write().unwrap();

                    for group in write_guard.iter().cloned() {
                        map = map.with_plugin(group);
                    }
                }

                for (n, tiles) in tiles.iter_mut().enumerate() {
                    let transparency = if n == 0 { 1.0 } else { 0.25 };
                    map = map.with_layer(tiles.as_mut(), transparency);
                }

                Frame::canvas(ui.style()).show(ui, |ui| {
                    profiling::scope!("Map::draw");
                    ui.add(map);
                });
            }

            {
                for component in &mut self.components {
                    component.draw(ctx);
                }
            }

            {
                let mut write_guard = self.map_info.map_memory.write().unwrap();
                crate::frames::go_to_my_position(ui, &mut write_guard);
            }
        });

        profiling::finish_frame!();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let app_state = self.app_state.read().unwrap();
        let store = app_state.store.read().unwrap();
        store.save();
    }
}

use sqlx::Row;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::{Arc, RwLock};
use utils::path::{config_path, variable_data_path};
use viewer::components::map::map_overlay::MapOverlay;
use viewer::viewer::Viewer;
use viewer::{Airport, AppState};

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let key_store = kv_sys::KVStore::new(config_path() + "settings.toml");

    start_puffin_server();
    profiling::register_thread!("Main Thread");

    let database_path = variable_data_path("adsb/adsb-viewer.db")
        .into_os_string()
        .into_string()
        .unwrap();
    println!("{}", database_path);

    let conn = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite://{}", database_path))
        .await
        .expect("Can't connect to DB");

    let tables = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
        .fetch_all(&conn)
        .await
        .expect("Failed to fetch tables");

    let airports_query =
        sqlx::query("SELECT iata, icao, name, city, country, latitude, longitude FROM airports")
            .fetch_all(&conn)
            .await
            .expect("Failed to fetch airports");

    let mut airports: Vec<Airport> = Vec::new();

    for row in airports_query {
        let lat_str: &str = row.get("latitude");
        let lat: f64 = lat_str.parse().unwrap_or(0.0);

        let lon_str: &str = row.get("longitude");
        let lon: f64 = lon_str.parse().unwrap_or(0.0);

        let airport = Airport {
            iata: row.get("iata"),
            icao: row.get("icao"),
            name: row.get("name"),
            city: row.get("city"),
            country: row.get("country"),
            lat,
            lon
        };

        airports.push(airport);
    }

    for table in tables {
        let name: &str = table.get("name");
        println!("Tabla: {}", name);
    }

    let app_state = AppState {
        store: Arc::new(RwLock::new(key_store)),
        airports: Arc::new(RwLock::new(airports)),
    };

    eframe::run_native(
        "ADS-B Viewer",
        Default::default(),
        Box::new(|cc| {
            let mut viewer = Viewer::new(
                cc.egui_ctx.clone(),
                Arc::new(RwLock::new(app_state)),
                vec![],
            );

            let map_memory = Arc::clone(&viewer.map_info.map_memory);

            viewer.add_component(MapOverlay::new(map_memory, Vec::new()));

            Ok(Box::new(viewer))
        }),
    )
}

fn start_puffin_server() {
    puffin::set_scopes_on(true); // tell puffin to collect data

    match puffin_http::Server::new("127.0.0.1:8585") {
        Ok(puffin_server) => {
            log::info!("Run:  cargo install puffin_viewer && puffin_viewer --url 127.0.0.1:8585");

            std::process::Command::new("puffin_viewer")
                .arg("--url")
                .arg("127.0.0.1:8585")
                .spawn()
                .ok();

            // We can store the server if we want, but in this case we just want
            // it to keep running. Dropping it closes the server, so let's not drop it!
            #[expect(clippy::mem_forget)]
            std::mem::forget(puffin_server);
        }
        Err(err) => {
            log::error!("Failed to start puffin server: {err}");
        }
    };
}
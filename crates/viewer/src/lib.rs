use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use walkers::extras::{
    GroupedPlaces, LabeledSymbol, LabeledSymbolGroup, LabeledSymbolGroupStyle, LabeledSymbolStyle,
    Symbol,
};
use walkers::{Plugin, lat_lon};

pub mod components;
pub mod frames;
pub mod tiles;
pub mod viewer;

type ArcRwLock<T> = std::sync::Arc<std::sync::RwLock<T>>;

pub struct Airport {
    pub iata: String,
    pub icao: String,
    pub name: String,
    pub city: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
}

lazy_static::lazy_static! {
    pub static ref AIRPORTS_PLUGIN: Arc<RwLock<Vec<GroupedPlaces<LabeledSymbol, LabeledSymbolGroup>>>> = Arc::new(RwLock::new(Vec::new()));
}

pub fn airport_plugin(
    airports_lock: Arc<RwLock<Vec<Airport>>>,
) -> Arc<RwLock<Vec<GroupedPlaces<LabeledSymbol, LabeledSymbolGroup>>>> {
    if !AIRPORTS_PLUGIN.read().unwrap().is_empty() {
        return Arc::clone(&AIRPORTS_PLUGIN);
    }

    let mut airports_group: Vec<GroupedPlaces<LabeledSymbol, LabeledSymbolGroup>> = Vec::new();

    let airports = airports_lock.read().unwrap();

    profiling::scope!("airport_plugin");
    let mut places_by_country: HashMap<String, Vec<LabeledSymbol>> = HashMap::new();

    for airport in &*airports {
        let label = format!("{} ({})", airport.name, airport.iata);

        let symbol = LabeledSymbol {
            position: walkers::lat_lon(airport.lat.clone(), airport.lon.clone()),
            label: airport.name.clone(),
            symbol: Some(Symbol::Circle("✈️".to_string())),
            style: LabeledSymbolStyle {
                symbol_size: 25.,
                ..Default::default()
            },
        };

        places_by_country
            .entry(airport.country.clone())
            .or_default()
            .push(symbol);
    }

    for (_, symbols) in places_by_country {
        airports_group.push(GroupedPlaces::new(
            symbols,
            LabeledSymbolGroup {
                style: LabeledSymbolGroupStyle::default(),
            }
        ));
    }

    {
        let mut write_guard = AIRPORTS_PLUGIN.write().unwrap();
        *write_guard = airports_group;
    }

    AIRPORTS_PLUGIN.clone()
}

pub struct AppState {
    pub store: ArcRwLock<kv_sys::KVStore>,
    pub airports: ArcRwLock<Vec<Airport>>,
}

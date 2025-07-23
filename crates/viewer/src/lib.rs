use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use walkers::extras::{
    GroupedPlaces, LabeledSymbol, LabeledSymbolGroup, LabeledSymbolGroupStyle, LabeledSymbolStyle,
    Places, Symbol,
};
use walkers::{MapMemory, Plugin, lat_lon};

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

pub fn airport_plugin(airports_lock: Arc<RwLock<Vec<Airport>>>) -> Vec<GroupedPlaces<LabeledSymbol, LabeledSymbolGroup>> {
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
            },
        ));
    }

    // airports_group.push(GroupedPlaces::new(
    //     places_by_country.get("\"Spain\"").unwrap().clone(),
    //     LabeledSymbolGroup {
    //         style: LabeledSymbolGroupStyle::default(),
    //     },
    // ));

    airports_group
}

pub fn airport_plugin_2(airports_lock: Arc<RwLock<Vec<Airport>>>) -> impl Plugin{
    let mut airports_group: Vec<GroupedPlaces<LabeledSymbol, LabeledSymbolGroup>> = Vec::new();

    let airports = airports_lock.read().unwrap();

    profiling::scope!("airport_plugin");
    let mut places: Vec<LabeledSymbol> = Vec::new();

    for airport in &*airports {

        let symbol = LabeledSymbol {
            position: walkers::lat_lon(airport.lat.clone(), airport.lon.clone()),
            label: airport.name.clone(),
            symbol: Some(Symbol::Circle("✈️".to_string())),
            style: LabeledSymbolStyle {
                symbol_size: 25.,
                ..Default::default()
            },
        };

        places.push(symbol);
    }

    GroupedPlaces::new(places,
        LabeledSymbolGroup {
            style: LabeledSymbolGroupStyle::default(),
        }
    )
}


pub struct AppState {
    pub store: ArcRwLock<kv_sys::KVStore>,
    pub airports: ArcRwLock<Vec<Airport>>,
}

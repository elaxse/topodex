use std::sync::Arc;

use geohash::Coord;
use ntex::web;
use rocksdb::{DBWithThreadMode, MultiThreaded};
use serde::{Deserialize, Serialize};

use crate::lookup_service::lookup_coordinate;

#[derive(Deserialize)]
pub struct Location {
    lat: f64,
    lng: f64,
}

#[derive(Deserialize)]
struct LocationsRequest {
    locations: Vec<Location>,
}

#[derive(Serialize)]
struct LocationsResponse {
    locations: Vec<String>,
}

pub struct AppState {
    pub db: Arc<DBWithThreadMode<MultiThreaded>>,
    pub max_geohash_level: usize,
}

#[web::get("/lookup")]
async fn lookup_single(
    location: web::types::Query<Location>,
    state: web::types::State<AppState>,
) -> impl web::Responder {
    let coord = Coord {
        x: location.lng,
        y: location.lat,
    };
    let res = lookup_coordinate(&state.db, coord, state.max_geohash_level)
        .await
        .unwrap();
    web::HttpResponse::Ok().body(res)
}

#[web::post("/lookup")]
async fn lookup_multiple(
    location_request: web::types::Json<LocationsRequest>,
    state: web::types::State<AppState>,
) -> impl web::Responder {
    let coordinates: Vec<_> = location_request
        .locations
        .iter()
        .map(|location| Coord {
            x: location.lng,
            y: location.lat,
        })
        .map(|coord| lookup_coordinate(&state.db, coord, state.max_geohash_level))
        .collect();

    let resolved_locations: Vec<_> = ntex::util::join_all(coordinates)
        .await
        .iter()
        .filter_map(|t| match t {
            Result::Ok(res) => Some(res.to_owned()),
            Result::Err(_) => None,
        })
        .collect();

    let location_response = LocationsResponse {
        locations: resolved_locations,
    };

    web::HttpResponse::Ok().json(&location_response)
}

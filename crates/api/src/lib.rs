use std::sync::Arc;

use geohash::GeohashError;
use geohash::{Coord, encode};
use ntex::web;
use rocksdb::DB;
use rocksdb::DBWithThreadMode;
use rocksdb::MultiThreaded;
use rocksdb::Options;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Location {
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

async fn lookup_coordinate(
    state: &web::types::State<AppState>,
    coord: Coord,
) -> Result<String, GeohashError> {
    let hash = encode(coord, state.max_geohash_level)?;

    for i in 1..=hash.len() {
        let substring = &hash[0..i];
        let data = state.db.get(substring.as_bytes()).unwrap();

        if let Some(out) = data {
            let res = String::from_utf8(out).unwrap();
            return Ok(res);
        }
    }

    Ok(String::from(""))
}

#[web::get("/lookup")]
async fn lookup(
    location: web::types::Query<Location>,
    state: web::types::State<AppState>,
) -> impl web::Responder {
    let coord = Coord {
        x: location.lng,
        y: location.lat,
    };
    let res = lookup_coordinate(&state, coord).await.unwrap();
    web::HttpResponse::Ok().body(res)
}

#[web::post("/lookup")]
async fn lookup_coordinates(
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
        .map(|coord| lookup_coordinate(&state, coord))
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

struct AppState {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
    max_geohash_level: usize,
}

pub async fn run_api(db_name: &str, max_geohash_level: usize) -> std::io::Result<()> {
    println!("Starting webserver on port 8090");
    let options = Options::default();
    let db = Arc::new(DB::open_for_read_only(&options, db_name, false).unwrap());

    web::HttpServer::new(move || {
        web::App::new()
            .state(AppState {
                db: db.clone(),
                max_geohash_level,
            })
            .service(lookup)
            .service(lookup_coordinates)
    })
    .bind(("127.0.0.1", 8090))?
    .run()
    .await
}

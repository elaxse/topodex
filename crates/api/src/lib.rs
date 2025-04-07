mod lookup_endpoint;
mod lookup_service;

use anyhow::{Ok, Result};
use lookup_endpoint::AppState;
use lookup_endpoint::{lookup_multiple, lookup_single};
use ntex::web;
use rocksdb::DB;
use std::sync::Arc;
use util::rocksdb_options;

pub async fn run_api(
    db_name: &str,
    max_geohash_level: usize,
    port: u16,
    workers: usize,
) -> Result<()> {
    println!("Starting webserver on port {}", port);

    let rockdb_options = rocksdb_options();
    let db = Arc::new(DB::open_for_read_only(&rockdb_options, db_name, false)?);

    web::HttpServer::new(move || {
        web::App::new()
            .state(AppState {
                db: db.clone(),
                max_geohash_level,
            })
            .service(lookup_single)
            .service(lookup_multiple)
    })
    .workers(workers)
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}

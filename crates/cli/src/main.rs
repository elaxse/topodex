use api::run_api;
use clap::{Parser, Subcommand};
use extract::extract;
use geo::Polygon;
use geojson::Feature;
use ntex;
use process::{extract_topologies, save_geohash_index};
use std::thread;
use std::{fs::read_to_string, str::FromStr};
use types::{GeohashIndex, TopodexConfig};

fn default_thread_count() -> String {
    thread::available_parallelism()
        .map(|p| p.get().to_string())
        .unwrap_or_else(|_| "1".to_string())
}

#[derive(Parser)]
#[command(version, about, long_about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(short, long)]
        geohash_db: String,

        #[arg(short, long, default_value_t = 5)]
        max_geohash_level: usize,

        #[arg(short, long, default_value_t = 8090)]
        port: u16,

        #[arg(short, long, default_value_os_t = default_thread_count())]
        threads: String,
    },
    Extract {
        #[arg(short, long)]
        osm_pbf_file: String,

        #[arg(short, long)]
        features_output_path: String,

        #[arg(short, long)]
        config_path: String,
    },
    Process {
        #[arg(short, long)]
        features_output_path: String,

        #[arg(short, long, default_value_t = 5)]
        max_geohash_level: usize,

        #[arg(short, long)]
        processed_features_output_path: Option<String>,

        #[arg(short, long)]
        geohash_db_output_path: String,

        #[arg(short, long)]
        config_path: String,
    },
}

#[ntex::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();

    match cli.command {
        Commands::Extract {
            osm_pbf_file,
            features_output_path,
            config_path,
        } => {
            let config = topodex_config(&config_path);
            println!("Read file {}", osm_pbf_file);
            let geometries = extract(&osm_pbf_file, &config);
            println!("Received {} geometries", geometries.len());

            let geojson_str = geometries
                .iter()
                .map(|feature| feature.to_string())
                .collect::<Vec<String>>()
                .join("\n");

            std::fs::write(features_output_path, geojson_str)?;
        }
        Commands::Process {
            features_output_path,
            max_geohash_level,
            processed_features_output_path,
            geohash_db_output_path,
            config_path,
        } => {
            let config = topodex_config(&config_path);

            let features_str = read_to_string(features_output_path)?;
            let geometries: Vec<Feature> = features_str
                .split("\n")
                .into_iter()
                .map(|feature_str| Feature::from_str(feature_str).unwrap())
                .collect();
            let geohash_indexes = extract_topologies(geometries, max_geohash_level, &config)?;
            println!("Geohash indexes count: {}", geohash_indexes.len());

            if let Some(output_path) = processed_features_output_path {
                let geojson_str = geohash_to_geojson(&geohash_indexes);
                std::fs::write(output_path, geojson_str)?;
            }

            save_geohash_index(geohash_indexes, &geohash_db_output_path)?;
        }
        Commands::Serve {
            geohash_db,
            max_geohash_level,
            port,
            threads,
        } => {
            let t: usize = threads.parse().unwrap();
            run_api(&geohash_db, max_geohash_level, port, t).await?;
        }
    }
    Ok(())
}

fn topodex_config(config_path: &str) -> TopodexConfig {
    let config_str = read_to_string(&config_path).unwrap();
    serde_json::from_str(&config_str)
        .inspect_err(|e| eprintln!("Error: {e}"))
        .unwrap()
}

fn geohash_to_geojson(geohash_indexes: &Vec<GeohashIndex>) -> String {
    let bboxes = geohash_indexes
        .iter()
        .filter_map(|geohash_index| match geohash_index {
            GeohashIndex::DirectValue { hash, value: _ } => {
                Some(geohash::decode_bbox(&hash).unwrap().to_polygon())
            }
            GeohashIndex::PartialValue {
                hash: _,
                value: _,
                shape,
            } => shape.into_iter().map(|t| t.clone()).next(),
        })
        .collect::<Vec<Polygon>>();

    let multi_polygon = geojson::Value::from(&geo::MultiPolygon(bboxes));

    let geometry = geojson::Geometry {
        value: multi_polygon,
        bbox: None,
        foreign_members: None,
    };

    let feature = Feature {
        id: None,
        properties: None,
        geometry: Some(geometry),
        foreign_members: None,
        bbox: None,
    };

    feature.to_string()
}

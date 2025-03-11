use api::run_api;
use clap::Parser;
use extract_osm::{extract_osm, extract_topologies, save_geohash_index};
use geo::Polygon;
use geojson::Feature;
use ntex;

#[derive(Parser)]
#[command(version, about, long_about)]
struct Args {
    #[arg(short, long)]
    input_pbf_file: String,

    #[arg(short, long, default_value_t = 5)]
    max_geohash_level: u8,

    #[arg(short, long)]
    raw_features_output_path: Option<String>,

    #[arg(short, long)]
    processed_features_output_path: Option<String>,

    #[arg(short, long)]
    geohash_db_output_path: String,
}

#[ntex::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Read file {}", args.input_pbf_file);
    let geometries = extract_osm(&args.input_pbf_file);
    println!("Received {} geometries", geometries.len());

    if let Some(output_path) = args.raw_features_output_path {
        let geojson_str = geometries
            .iter()
            .map(|feature| feature.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        std::fs::write(output_path, geojson_str)?;
    }

    let geohash_indexes = extract_topologies(geometries, args.max_geohash_level)?;
    println!("Geohash indexes count: {}", geohash_indexes.len());

    if let Some(output_path) = args.processed_features_output_path {
        let bboxes = geohash_indexes
            .iter()
            .map(|geohash_index| geohash::decode_bbox(&geohash_index.hash))
            .filter_map(Result::ok)
            .map(|bbox| bbox.to_polygon())
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

        let geojson_str = feature.to_string();
        std::fs::write(output_path, geojson_str)?;
    }

    save_geohash_index(geohash_indexes, &args.geohash_db_output_path)?;
    run_api(&args.geohash_db_output_path).await?;

    Ok(())
}

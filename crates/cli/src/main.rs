use clap::Parser;
use extract_osm::extract_osm;

#[derive(Parser)]
#[command(version, about, long_about)]
struct Args {
    #[arg(short, long)]
    input_pbf_file: String,

    #[arg(short, long, default_value_t = 5)]
    max_geohash_level: i8,

    #[arg(short, long)]
    geojson_output_path: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Read file {}", args.input_pbf_file);
    let geometries = extract_osm(&args.input_pbf_file);
    println!("Received {} geometries", geometries.len());

    if let Some(output_path) = args.geojson_output_path {
        let geojson_str = geometries
            .iter()
            .map(|feature| feature.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        std::fs::write(output_path, geojson_str)?;
    }

    Ok(())
}

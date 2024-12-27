use clap::Parser;
use extract_osm::extract_osm;

#[derive(Parser)]
#[command(version, about, long_about)]
struct Args {
    #[arg(short, long)]
    input_pbf_file: String,
    
    #[arg(short, long, default_value_t = 5)]
    max_geohash_level: i8
}

fn main() {
    let args = Args::parse();

    println!("Read file {}", args.input_pbf_file);
    extract_osm(&args.input_pbf_file);
}

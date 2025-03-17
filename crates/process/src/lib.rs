mod fill_polygon;

use std::{collections::HashMap, error::Error};

use fill_polygon::fill_polygon;
use geo::{MultiPolygon, Polygon};
use geojson::{Feature, Value, feature::Id};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rocksdb::DB;
use types::{GeohashIndex, GeohashValue, UndecidedValue};

pub fn extract_topologies(
    features: Vec<Feature>,
    max_geohash_level: usize,
) -> Result<Vec<GeohashIndex>, Box<dyn Error>> {
    let geohashes: Vec<GeohashIndex> = features
        .into_par_iter()
        .map(|feature| {
            if let Some(geometry) = feature.geometry {
                let feature_shape_option = match &geometry.value {
                    Value::MultiPolygon(_) => {
                        let geo_polygon: MultiPolygon<f64> = MultiPolygon::try_from(geometry)?;
                        Some(geo_polygon)
                    }
                    Value::Polygon(_) => {
                        let geo: Polygon<f64> = Polygon::try_from(geometry)?;
                        Some(MultiPolygon(vec![geo]))
                    }
                    _ => {
                        println!("Unsupported geometry");
                        None
                    }
                };
                if let (Some(feature_shape), Some(feature_id)) = (feature_shape_option, feature.id)
                {
                    let fid = match feature_id {
                        Id::String(s) => s,
                        Id::Number(n) => n.to_string(),
                    };
                    return fill_polygon(feature_shape, fid, max_geohash_level);
                }
            }
            Ok(Vec::new())
        })
        .filter_map(Result::ok)
        .flatten()
        .collect();

    Ok(geohashes)
}

pub fn save_geohash_index(
    geohashes: Vec<GeohashIndex>,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut map = HashMap::<String, GeohashValue>::new();

    for geohash_index in geohashes {
        match geohash_index {
            GeohashIndex::DirectValue { hash, value } => {
                map.insert(hash, GeohashValue::DirectValue { value });
            }
            GeohashIndex::PartialValue { hash, value, shape } => {
                if let Some(GeohashValue::Undecided { options }) = map.get_mut(&hash) {
                    options.push(UndecidedValue { value, shape });
                } else {
                    map.insert(
                        hash,
                        GeohashValue::Undecided {
                            options: vec![UndecidedValue { value, shape }],
                        },
                    );
                }
            }
        }
    }

    let db = DB::open_default(path).unwrap();
    for (hash, value) in map.iter() {
        db.put(hash.as_bytes(), bitcode::serialize(value).unwrap())?;
    }

    db.flush()?;

    Ok(())
}

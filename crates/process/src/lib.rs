mod fill_polygon;

use anyhow::Result;
use fill_polygon::fill_polygon;
use geo::{MultiPolygon, Polygon};
use geojson::{Feature, Value};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rocksdb::DB;
use std::collections::HashMap;
use util::rocksdb_options;
use util::{GeohashIndex, GeohashValue, TopodexConfig, UndecidedValue};

pub fn extract_topologies(
    features: Vec<Feature>,
    max_geohash_level: usize,
    config: &TopodexConfig,
) -> Result<Vec<GeohashIndex>> {
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

                if let (Some(feature_shape), Some(shape_value)) = (
                    feature_shape_option,
                    feature.properties.and_then(|properties| {
                        properties
                            .get(&config.process_property_name)
                            .and_then(|property_value| {
                                property_value
                                    .as_str()
                                    .map(|property_value_str| property_value_str.to_owned())
                            })
                    }),
                ) {
                    return fill_polygon(feature_shape, shape_value, max_geohash_level);
                }
            }
            Ok(Vec::new())
        })
        .filter_map(Result::ok)
        .flatten()
        .collect();

    Ok(geohashes)
}

pub fn save_geohash_index(geohashes: Vec<GeohashIndex>, path: &str) -> Result<()> {
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

    let rocksdb_options = rocksdb_options();
    let db = DB::open(&rocksdb_options, path)?;
    for (hash, value) in map.iter() {
        db.put(hash.as_bytes(), bitcode::serialize(value).unwrap())?;
    }

    db.flush()?;

    Ok(())
}

mod element_collection_reader;
mod read_osm_data;

use geo::{coord, BooleanOps, Contains, Coord, Intersects, MultiPolygon, Polygon};
use geohash::decode_bbox;
use geojson::{feature::Id, Feature, Geometry, Value};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use read_osm_data::read_osm_elements;
use std::error::Error;
use std::time::Instant;
use types::{RelationWithLocations, Way};

#[derive(Debug)]
pub enum GeohashValue {
    GeohashCountry(u16),
    PartialGeohash(Vec<MultiPolygon>),
}

#[derive(Debug, Clone)]
struct ShouldCheck {
    hash: String,
    area: MultiPolygon,
}

#[derive(Debug)]
pub struct GeohashIndex {
    pub hash: String,
    pub value: Id,
}

pub fn extract_topologies(
    features: Vec<Feature>,
    max_geohash_level: u8,
) -> Result<Vec<GeohashIndex>, Box<dyn Error>> {
    let geohashes: Vec<GeohashIndex> = features
        .par_iter()
        .map(|feature| {
            if let Some(geometry) = feature.clone().geometry {
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
                if let (Some(feature_shape), Some(feature_id)) =
                    (feature_shape_option, feature.id.clone())
                {
                    return process_geometry(feature_shape, feature_id, max_geohash_level);
                }
            }
            Ok(Vec::new())
        })
        .filter_map(Result::ok)
        .flatten()
        .collect();

    Ok(geohashes)
}

fn process_geometry(
    geo_polygon: MultiPolygon,
    country_id: Id,
    max_geohash_level: u8,
) -> Result<Vec<GeohashIndex>, Box<dyn Error + Send + Sync>> {
    let mut geohashes = Vec::<GeohashIndex>::new();
    let geohash_possibilities: Vec<String> = "0123456789bcdefghjkmnpqrstuvwxyz"
        .chars()
        .map(|c| c.to_string())
        .collect();

    let mut geohashes_to_check: Vec<ShouldCheck> = geohash_possibilities
        .iter()
        .map(|c| ShouldCheck {
            hash: c.to_string(),
            area: geo_polygon.clone(),
        })
        .collect();

    let start = std::time::Instant::now();
    for _ in 1..=max_geohash_level {
        let mut next_geohashes_check = Vec::<ShouldCheck>::new();

        for check in geohashes_to_check {
            let rect = decode_bbox(&check.hash)?;
            let mp = MultiPolygon(vec![rect.to_polygon()]);
            if check.area.contains(&rect) {
                geohashes.push(GeohashIndex {
                    hash: check.hash.clone(),
                    value: country_id.clone(),
                })
            } else if check.area.intersects(&rect) {
                let intersecting_polygon = check.area.intersection(&mp);
                let options_to_check: Vec<ShouldCheck> = geohash_possibilities
                    .iter()
                    .map(|h| ShouldCheck {
                        hash: format!("{}{}", check.hash, h),
                        area: intersecting_polygon.clone(),
                    })
                    .collect();
                next_geohashes_check.extend(options_to_check);
            }
        }

        geohashes_to_check = next_geohashes_check;
    }

    println!(
        "processed relation {:?} in {:.2?}",
        country_id,
        start.elapsed()
    );
    Ok(geohashes)
}

pub fn extract_osm(path: &str) -> Vec<Feature> {
    let (relations, ways, nodes) = read_osm_elements(path);
    let start = Instant::now();
    let mut countries = Vec::<RelationWithLocations>::new();

    for relation in relations {
        let mut relation_ways: Vec<Way> = vec![];
        let mut relation_data_complete = true;

        for way in relation.members.iter() {
            if let Some(node_ids) = ways.get(&way) {
                relation_ways.push(Way {
                    id: way.clone(),
                    node_ids: node_ids.to_vec(),
                })
            }
        }

        let mut parts: Vec<Vec<Coord>> = vec![];

        while !relation_ways.is_empty() {
            let start_node_id: &i64 = &relation_ways
                .get(0)
                .unwrap()
                .node_ids
                .get(0)
                .unwrap()
                .clone();
            let mut search_node_id = start_node_id.clone();

            let mut part: Vec<Coord<f64>> = vec![];
            let first_node_option = nodes.get(start_node_id);

            if first_node_option.is_none() {
                panic!("Node {start_node_id} not found");
            }

            let first_node = first_node_option.unwrap();

            part.push(coord! {x: first_node.0, y: first_node.1});

            loop {
                if let Some(way) = find_match(&search_node_id, &mut relation_ways) {
                    let mut locations: Vec<Coord> = way
                        .node_ids
                        .iter()
                        .skip(1)
                        .map(|node_id| nodes.get(node_id))
                        .flatten()
                        .map(|(lon, lat)| coord! {x: lon.clone(), y: lat.clone()})
                        .collect();

                    part.append(&mut locations);

                    search_node_id = way.node_ids.last().unwrap().clone();

                    if &search_node_id == start_node_id {
                        break;
                    }
                } else {
                    // in case not whole shape is present make connection to starting point to be valid polygon
                    relation_data_complete = false;
                    break;
                }
            }

            if !relation_data_complete {
                break;
            }
            parts.push(part);
        }

        if !relation_data_complete {
            continue;
        }

        countries.push(RelationWithLocations {
            id: relation.id,
            locations: parts,
        })
    }

    println!(
        "Countries combination: {} seconds",
        start.elapsed().as_secs()
    );

    countries
        .iter()
        .map(|country| {
            let geometry = Geometry::new(Value::Polygon(vec![country
                .locations
                .iter()
                .flatten()
                .map(|location| vec![location.x, location.y])
                .collect()]));

            Feature {
                bbox: None,
                geometry: Some(geometry),
                id: Some(Id::String(country.id.to_string())),
                properties: None,
                foreign_members: None,
            }
        })
        .collect::<Vec<Feature>>()
}

fn find_match(node_id: &i64, ways: &mut Vec<Way>) -> Option<Way> {
    for (i, way) in ways.iter().enumerate() {
        if let Some(node) = way.node_ids.first() {
            if node == node_id {
                return Some(ways.swap_remove(i));
            }
        }

        if let Some(node) = way.node_ids.last() {
            if node == node_id {
                let mut way = ways.swap_remove(i);
                way.node_ids.reverse();
                return Some(way);
            }
        }
    }

    None
}

mod element_collection_reader;
mod read_osm_data;

use geo::{coord, Coord};
use geojson::{feature::Id, Feature, Geometry, Value};
use read_osm_data::read_osm_elements;
use std::time::Instant;
use types::{RelationWithLocations, Way};

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

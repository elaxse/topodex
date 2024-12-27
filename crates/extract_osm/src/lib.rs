mod element_collection_reader;

use std::{collections::{HashMap, HashSet}, fs, time::Instant};
use osmpbf::{Element, Relation};
use types::{RelationWithLocations, RelationWithMembers, Way};
use element_collection_reader::ElementCollectReader;
use geojson::{feature::Id, Feature, GeoJson, Geometry, Value};


pub fn extract_osm(path: &str) {
    let required_tags: Vec<(&str, Option<&str>)> = vec![
        ("type", Some("boundary")),
        ("admin_level", Some("4")),
        ("boundary", Some("administrative")),
        ("ISO3166-2", None),
    ];

    let start = Instant::now();
    let relations = read_relations(&path, &required_tags).unwrap();
    println!("Relations extract: {} seconds", start.elapsed().as_secs());

    let start = Instant::now();
    let ways_set: HashSet<i64> = relations
        .iter()
        .map(|relation| relation.members.clone())
        .flatten()
        .collect();
    println!("Ways set: {} seconds", start.elapsed().as_secs());

    let start = Instant::now();
    let ways = read_ways(&path, &ways_set).unwrap();
    println!("Ways extract: {} seconds", start.elapsed().as_secs());

    let start = Instant::now();
    let nodes_set: HashSet<i64> = ways
        .iter()
        .map(|(_, node_ids)| node_ids.clone())
        .flatten()
        .collect();
    println!("Nodes set: {} seconds", start.elapsed().as_secs());

    let start = Instant::now();
    let nodes = read_nodes(&path, &nodes_set).unwrap();
    println!("Nodes extract: {} seconds", start.elapsed().as_secs());

    let start = Instant::now();
    let mut countries = Vec::<RelationWithLocations>::new();

    for relation in relations {
        let mut relation_ways: Vec<Way> = vec![];

        for way in relation.members.iter() {
            if let Some(node_ids) = ways.get(&way) {
                relation_ways.push(Way {
                    id: way.clone(),
                    node_ids: node_ids.to_vec(),
                })
            }
        }

        let mut parts: Vec<Vec<Vec<f64>>> = vec![];

        while !relation_ways.is_empty() {
            let start_node_id: &i64 = &relation_ways
                .get(0)
                .unwrap()
                .node_ids
                .get(0)
                .unwrap()
                .clone();
            let mut search_node_id = start_node_id.clone();

            let mut part: Vec<Vec<f64>> = vec![];
            let first_node_option = nodes.get(start_node_id);

            if first_node_option.is_none() {
                println!("Node {start_node_id} not found");
                break;
            }

            let first_node = first_node_option.unwrap();

            part.push(vec![first_node.0, first_node.1]);

            loop {
                if let Some(way) = find_match(&search_node_id, &mut relation_ways) {
                    let mut locations: Vec<Vec<f64>> = way
                        .node_ids
                        .iter()
                        .skip(1)
                        .map(|node_id| nodes.get(node_id))
                        .flatten()
                        .map(|(lon, lat)| vec![lon.clone(), lat.clone()])
                        .collect();

                    part.append(&mut locations);

                    search_node_id = way.node_ids.last().unwrap().clone();

                    if &search_node_id == start_node_id {
                        break;
                    }
                } else {
                    // in case not whole shape is present make connection to starting point to be valid polygon
                    part.push(vec![first_node.0, first_node.1]);
                    break;
                }
            }

            parts.push(part);
        }

        // println!("Country {} has {} polygons", relation.id, parts.len());
        countries.push(RelationWithLocations {
            id: relation.id,
            locations: parts,
        })
    }

    println!(
        "Countries combination: {} seconds",
        start.elapsed().as_secs()
    );

    // println!("Found {:?} countries", countries.len());

    let start = Instant::now();

    let mut geojson_output = Vec::<String>::new();

    for country in countries {
        let geometry = Geometry::new(Value::MultiPolygon(
            country
                .locations
                .into_iter()
                .map(|location| vec![location])
                .collect(),
        ));

        let geojson = GeoJson::Feature(Feature {
            bbox: None,
            geometry: Some(geometry),
            id: Some(Id::String(country.id.to_string())),
            properties: None,
            foreign_members: None,
        });

        let geojson_string = geojson.to_string();
        geojson_output.push(geojson_string)
    }
    fs::write("assets/countries.ndgeojson", geojson_output.join("\n")).unwrap();

    println!("Geosjon convert: {} seconds", start.elapsed().as_secs());

    ()
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

fn read_relations(
    path: &str,
    required_tags: &[(&str, Option<&str>)],
) -> Result<Vec<RelationWithMembers>, osmpbf::Error> {
    let relations = ElementCollectReader::from_path(path)?.elements(|element| match element {
        Element::Relation(relation) => {
            let match_tags = relation_filter(&relation, &required_tags);

            if !match_tags {
                return None;
            }

            let members: Vec<i64> = relation
                .members()
                .into_iter()
                .filter(|member| member.role().unwrap() == "outer")
                .map(|member| member.member_id)
                .collect();

            let out_rel = RelationWithMembers {
                id: relation.id(),
                members,
            };

            Some(out_rel)
        }
        _ => None,
    })?;

    Ok(relations)
}

fn relation_filter(relation: &Relation, required_tags: &[(&str, Option<&str>)]) -> bool {
    for required_tag in required_tags {
        let mut found_tag = false;
        for (key, val) in relation.tags() {
            if key == required_tag.0 && (required_tag.1.is_none() || val == required_tag.1.unwrap())
            {
                found_tag = true;
                break;
            }
        }

        if found_tag == false {
            return found_tag;
        }
    }

    true
}

fn read_ways(path: &str, ways_set: &HashSet<i64>) -> Result<HashMap<i64, Vec<i64>>, osmpbf::Error> {
    let ways = ElementCollectReader::from_path(path)?
        .elements(|element| match element {
            Element::Way(way) => {
                let id = way.id();
                if ways_set.contains(&id) {
                    return Some((id, way.refs().collect::<Vec<i64>>()));
                }
                None
            }
            _ => None,
        })?
        .into_iter()
        .collect();

    Ok(ways)
}

fn read_nodes(
    path: &str,
    nodes_set: &HashSet<i64>,
) -> Result<HashMap<i64, (f64, f64)>, osmpbf::Error> {
    let nodes: HashMap<i64, (f64, f64)> = ElementCollectReader::from_path(path)?
        .elements(|element| match element {
            Element::Node(node) => {
                if nodes_set.contains(&node.id()) {
                    return Some((node.id(), (node.lon(), node.lat())));
                }
                None
            }
            Element::DenseNode(node) => {
                if nodes_set.contains(&node.id()) {
                    return Some((node.id(), (node.lon(), node.lat())));
                }
                None
            }
            _ => None,
        })?
        .into_iter()
        .collect();
    Ok(nodes)
}

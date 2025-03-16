use osmpbf::{Element, Relation};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};
use types::{ExtractConfig, RelationWithMembers};

use crate::element_collection_reader::ElementCollectReader;

pub fn read_osm_elements(
    path: &str,
    extract_config: &ExtractConfig,
) -> (
    Vec<RelationWithMembers>,
    HashMap<i64, Vec<i64>>,
    HashMap<i64, (f64, f64)>,
) {
    let start = Instant::now();
    let relations = read_relations(
        &path,
        &extract_config.filters,
        &extract_config.extract_properties,
    )
    .unwrap();
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

    (relations, ways, nodes)
}

fn read_relations(
    path: &str,
    required_tags: &[(String, Option<String>)],
    property_filters: &[(String, Option<String>)],
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

            let tags: serde_json::Map<String, Value> = relation
                .tags()
                .filter_map(|(key, value)| {
                    for (fkey, rkey) in property_filters {
                        if fkey == key {
                            let nkey = (rkey.as_deref().unwrap_or(fkey)).to_owned();
                            let nval = serde_json::Value::String(value.to_owned());
                            return Some((nkey, nval));
                        }
                    }
                    None
                })
                .collect();

            let out_rel = RelationWithMembers {
                id: relation.id(),
                members,
                tags,
            };

            Some(out_rel)
        }
        _ => None,
    })?;

    Ok(relations)
}

fn relation_filter(relation: &Relation, required_tags: &[(String, Option<String>)]) -> bool {
    for required_tag in required_tags {
        let req_val = required_tag.1.as_deref();
        let mut found_tag = false;
        for (key, val) in relation.tags() {
            if key == required_tag.0 && (req_val.is_none() || val == req_val.unwrap()) {
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

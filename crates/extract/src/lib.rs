mod element_collection_reader;
mod read_osm_data;

use anyhow::Result;
use geo::{coord, Coord, LineString, MultiPolygon, Polygon, Within};
use geojson::{feature::Id, Feature, Geometry, Value};
use read_osm_data::read_osm_elements;
use std::{collections::HashMap, time::Instant};
use util::{RelationMember, RelationWithLocations, RelationWithMembers, TopodexConfig, Way};

pub fn extract(path: &str, extract_config: &TopodexConfig) -> Result<Vec<Feature>> {
    let (relations, ways, nodes) = read_osm_elements(path, extract_config)?;
    let start = Instant::now();
    println!(
        "Countries combination: {} seconds",
        start.elapsed().as_secs()
    );

    let countries = build_relations(relations, ways, nodes)?;

    Ok(countries
        .into_iter()
        .map(|country| {
            let geometry = Geometry::new(Value::from(&country.shape));

            Feature {
                bbox: None,
                geometry: Some(geometry),
                id: Some(Id::String(country.id.to_string())),
                properties: Some(country.tags),
                foreign_members: None,
            }
        })
        .collect::<Vec<Feature>>())
}

fn extract_ways(
    relation: &RelationWithMembers,
    ways: &HashMap<i64, Vec<i64>>,
) -> (Vec<Way>, Vec<Way>) {
    relation
        .members
        .iter()
        .filter_map(|way| {
            if let Some(node_ids) = ways.get(&way.to_i64()) {
                Some(Way {
                    id: way.to_i64(),
                    node_ids: node_ids.to_vec(),
                    outer: matches!(way, RelationMember::OuterMember { .. }),
                })
            } else {
                None
            }
        })
        .partition(|way| way.outer)
}

fn build_polygons(ways: &mut Vec<Way>, nodes: &HashMap<i64, (f64, f64)>) -> Result<Vec<Polygon>> {
    let mut polygons = Vec::<Polygon>::new();
    let mut relation_data_complete = true;

    while !ways.is_empty() {
        let start_node_id = ways.get(0).unwrap().node_ids.get(0).unwrap().clone();
        let mut polygon_parts = Vec::<Coord>::new();
        let mut search_node_id = start_node_id.clone();
        if let Some((lng, lat)) = nodes.get(&start_node_id) {
            polygon_parts.push(coord! {x: *lng, y: *lat});
        }

        loop {
            if let Some(way) = find_match(&search_node_id, ways) {
                let mut locations: Vec<Coord> = way
                    .node_ids
                    .iter()
                    .skip(1)
                    .map(|node_id| nodes.get(node_id))
                    .flatten()
                    .map(|(lon, lat)| coord! {x: lon.clone(), y: lat.clone()})
                    .collect();

                polygon_parts.append(&mut locations);

                search_node_id = way.node_ids.last().unwrap().clone();

                if search_node_id == start_node_id {
                    break;
                }
            } else {
                relation_data_complete = false;
                break;
            }
        }

        if !relation_data_complete {
            break;
        }
        let outline = LineString::new(polygon_parts);
        polygons.push(Polygon::new(outline, vec![]));
    }
    Ok(polygons)
}
fn assemble_polygons(outer_polygons: &Vec<Polygon>, inner_polygons: &Vec<Polygon>) -> MultiPolygon {
    let mut result_polygons = Vec::new();

    for outer_polygon in outer_polygons.iter() {
        let mut polygon = outer_polygon.clone();

        for inner_polygon in inner_polygons {
            if inner_polygon.is_within(outer_polygon) {
                polygon.interiors_push(inner_polygon.exterior().clone());
            }
        }

        result_polygons.push(polygon);
    }

    MultiPolygon::new(result_polygons)
}

fn build_relations(
    relations: Vec<RelationWithMembers>,
    ways: HashMap<i64, Vec<i64>>,
    nodes: HashMap<i64, (f64, f64)>,
) -> Result<Vec<RelationWithLocations>> {
    let mut processed_relations = Vec::<RelationWithLocations>::new();

    for relation in relations {
        let (mut outer_ways, mut inner_ways) = extract_ways(&relation, &ways);

        let outer_polygons = build_polygons(&mut outer_ways, &nodes)?;
        let inner_polygons = build_polygons(&mut inner_ways, &nodes)?;
        let multi_polygon = assemble_polygons(&outer_polygons, &inner_polygons);

        if outer_polygons.len() < 1 {
            continue;
        }

        processed_relations.push(RelationWithLocations {
            id: relation.id,
            shape: multi_polygon,
            tags: relation.tags,
        })
    }
    Ok(processed_relations)
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

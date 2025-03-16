use geo::{Coord, MultiPolygon};
use geojson::JsonObject;

#[derive(Debug, Clone)]
pub struct RelationWithMembers {
    pub id: i64,
    pub members: Vec<i64>,
    pub tags: JsonObject,
}

#[derive(Debug, Clone)]
pub struct RelationWithLocations {
    pub id: i64,
    pub locations: Vec<Vec<Coord>>,
    pub tags: JsonObject,
}

#[derive(Debug, Clone)]
pub struct Way {
    pub id: i64,
    pub node_ids: Vec<i64>,
}

#[derive(Debug)]
pub struct GeohashIndex {
    pub hash: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct ShouldCheck {
    pub hash: String,
    pub area: MultiPolygon,
}

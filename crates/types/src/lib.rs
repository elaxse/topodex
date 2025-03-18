use geo::{Coord, MultiPolygon};
use geojson::JsonObject;
use serde::{Deserialize, Serialize};

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
pub enum GeohashIndex {
    DirectValue {
        hash: String,
        value: String,
    },
    PartialValue {
        hash: String,
        value: String,
        shape: MultiPolygon,
    },
}

#[derive(Debug, Clone)]
pub struct ShouldCheck {
    pub hash: String,
    pub area: MultiPolygon,
}

#[derive(Deserialize)]
pub struct TopodexConfig {
    pub filters: Vec<(String, Option<String>)>,
    pub extract_properties: Vec<(String, Option<String>)>,
    pub process_property_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UndecidedValue {
    pub value: String,
    pub shape: MultiPolygon,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GeohashValue {
    DirectValue { value: String },
    Undecided { options: Vec<UndecidedValue> },
}

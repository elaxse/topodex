mod rocksdb_helper;

use geo::MultiPolygon;
use geojson::JsonObject;
pub use rocksdb_helper::rocksdb_options;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum RelationMember {
    OuterMember(i64),
    InnerMember(i64),
}

impl RelationMember {
    pub fn to_i64(&self) -> i64 {
        match self {
            RelationMember::OuterMember(id) => *id,
            RelationMember::InnerMember(id) => *id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RelationWithMembers {
    pub id: i64,
    pub members: Vec<RelationMember>,
    pub tags: JsonObject,
}

#[derive(Debug, Clone)]
pub struct RelationWithLocations {
    pub id: i64,
    pub shape: MultiPolygon,
    pub tags: JsonObject,
}

#[derive(Debug, Clone)]
pub struct Way {
    pub id: i64,
    pub node_ids: Vec<i64>,
    pub outer: bool,
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

#[derive(Debug, Clone)]
pub struct RelationWithMembers {
    pub id: i64,
    pub members: Vec<i64>,
}

#[derive(Debug, Clone)]
pub struct RelationWithLocations {
    pub id: i64,
    pub locations: Vec<Vec<Vec<f64>>>,
}

#[derive(Debug, Clone)]
pub struct Way {
    pub id: i64,
    pub node_ids: Vec<i64>,
}
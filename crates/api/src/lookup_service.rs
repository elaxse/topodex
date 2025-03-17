use geo::Contains;
use geohash::{Coord, GeohashError, encode};
use rocksdb::{DBWithThreadMode, MultiThreaded};
use types::GeohashValue;

pub async fn lookup_coordinate(
    db: &DBWithThreadMode<MultiThreaded>,
    coord: Coord,
    max_geohash_level: usize,
) -> Result<String, GeohashError> {
    let hash = encode(coord, max_geohash_level)?;

    for i in 1..=hash.len() {
        let substring = &hash[0..i];
        let data = db.get(substring.as_bytes()).unwrap();

        if let Some(out) = data {
            let res = bitcode::deserialize::<GeohashValue>(&out).unwrap();

            let contains_res = match res {
                GeohashValue::DirectValue { value } => Some(value),
                GeohashValue::Undecided { options } => options
                    .iter()
                    .find(|option| {
                        option.shape.contains(&geo::Coord {
                            x: coord.x,
                            y: coord.y,
                        })
                    })
                    .and_then(|option| Some(option.value.clone())),
            };

            if let Some(val) = contains_res {
                return Ok(val);
            }
        }
    }

    Ok(String::from(""))
}

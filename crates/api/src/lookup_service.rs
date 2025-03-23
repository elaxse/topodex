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

    let substrings = (1..=hash.len()).map(|i| &hash[0..i]).collect::<Vec<&str>>();
    let vals = db.multi_get(substrings);

    for lookup_val in vals {
        if let Result::Ok(Some(out)) = lookup_val {
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

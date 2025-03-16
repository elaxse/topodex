use geohash::{Coord, GeohashError, encode};
use rocksdb::{DBWithThreadMode, MultiThreaded};

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
            let res = String::from_utf8(out).unwrap();
            return Ok(res);
        }
    }

    Ok(String::from(""))
}

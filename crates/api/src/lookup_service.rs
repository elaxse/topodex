use geo::Contains;
use geohash::{encode, Coord, GeohashError};
use rocksdb::{DBWithThreadMode, MultiThreaded};
use util::GeohashValue;

pub fn lookup_coordinates(
    db: &DBWithThreadMode<MultiThreaded>,
    coords: Vec<Coord>,
    max_geohash_level: usize,
) -> Result<Vec<String>, GeohashError> {
    let hash_strings: Vec<String> = coords
        .iter()
        .map(|coord| encode(coord.clone(), max_geohash_level).unwrap())
        .flat_map(|hash| (1..=hash.len()).map(move |i| hash[0..i].to_string()))
        .collect();

    let hash_string_slices: Vec<&str> = hash_strings
        .iter()
        .map(|hash_string| hash_string.as_str())
        .collect();

    let lookup_res: Vec<_> = db.multi_get(hash_string_slices);
    let lookup_chunks: Vec<_> = lookup_res.chunks(max_geohash_level).collect();

    let mut resolved_locations = Vec::<String>::new();

    for (i, chunk) in lookup_chunks.into_iter().enumerate() {
        let mut found_loc = false;
        for lookup_val in chunk {
            if let Result::Ok(Some(out)) = lookup_val {
                let res = bitcode::deserialize::<GeohashValue>(&out).unwrap();

                let contains_res = match res {
                    GeohashValue::DirectValue { value } => Some(value),
                    GeohashValue::Undecided { options } => options
                        .iter()
                        .find(|option| {
                            let coord = coords.get(i).unwrap();
                            option.shape.contains(&geo::Coord {
                                x: coord.x,
                                y: coord.y,
                            })
                        })
                        .and_then(|option| Some(option.value.clone())),
                };

                if let Some(val) = contains_res {
                    resolved_locations.push(val);
                    found_loc = true;
                    break;
                }
            }
        }
        if !found_loc {
            resolved_locations.push(String::from(""));
        }
    }

    Ok(resolved_locations)
}

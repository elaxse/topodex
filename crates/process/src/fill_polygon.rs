use std::error::Error;

use geo::{BooleanOps, Contains, Intersects, MultiPolygon};
use geohash::decode_bbox;
use types::{GeohashIndex, ShouldCheck};

pub fn fill_polygon(
    geo_polygon: MultiPolygon,
    country_id: String,
    max_geohash_level: usize,
) -> Result<Vec<GeohashIndex>, Box<dyn Error + Send + Sync>> {
    let mut geohashes = Vec::<GeohashIndex>::new();
    let geohash_possibilities: Vec<String> = "0123456789bcdefghjkmnpqrstuvwxyz"
        .chars()
        .map(|c| c.to_string())
        .collect();

    let mut geohashes_to_check: Vec<ShouldCheck> = geohash_possibilities
        .iter()
        .map(|c| ShouldCheck {
            hash: c.to_string(),
            area: geo_polygon.clone(),
        })
        .collect();

    let start = std::time::Instant::now();
    for _ in 1..=max_geohash_level {
        let mut next_geohashes_check = Vec::<ShouldCheck>::new();

        for check in geohashes_to_check {
            let rect = decode_bbox(&check.hash)?;
            let mp = MultiPolygon(vec![rect.to_polygon()]);
            if check.area.contains(&rect) {
                geohashes.push(GeohashIndex {
                    hash: check.hash.clone(),
                    value: country_id.clone(),
                })
            } else if check.area.intersects(&rect) {
                let intersecting_polygon = check.area.intersection(&mp);
                let options_to_check: Vec<ShouldCheck> = geohash_possibilities
                    .iter()
                    .map(|h| ShouldCheck {
                        hash: format!("{}{}", check.hash, h),
                        area: intersecting_polygon.clone(),
                    })
                    .collect();
                next_geohashes_check.extend(options_to_check);
            }
        }

        geohashes_to_check = next_geohashes_check;
    }

    println!(
        "processed relation {:?} in {:.2?}",
        country_id,
        start.elapsed()
    );
    Ok(geohashes)
}

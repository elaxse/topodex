use std::{fs::File, io::{BufReader, Read}, path::Path};

use osmpbf::{BlobDecode, BlobReader, Element};
use rayon::iter::{ParallelBridge, ParallelIterator};

pub struct ElementCollectReader<R: Read + Send> {
    blob_iter: BlobReader<R>,
}

impl ElementCollectReader<BufReader<File>> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, osmpbf::Error> {
        Ok(ElementCollectReader {
            blob_iter: BlobReader::from_path(path)?,
        })
    }

    pub fn elements<T, FMO>(self, filter_map_op: FMO) -> Result<Vec<T>, osmpbf::Error>
    where
        T: Send,
        FMO: for<'a> Fn(Element<'a>) -> Option<T> + Send + Sync,
    {
        let result: Vec<T> = self
            .blob_iter
            .par_bridge()
            .filter_map(Result::ok)
            .flat_map(|blob| match blob.decode() {
                Ok(BlobDecode::OsmData(block)) => Some(
                    block
                        .elements()
                        .filter_map(&filter_map_op)
                        .collect::<Vec<T>>(),
                ),
                _ => None,
            })
            .flatten()
            .collect();
        Ok(result)
    }
}
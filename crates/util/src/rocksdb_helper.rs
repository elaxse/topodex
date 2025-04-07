use rocksdb::{BlockBasedOptions, Cache, Options};

pub fn rocksdb_options() -> Options {
    let cache = Cache::new_lru_cache(3 * 1024 * 1024 * 1024);
    let mut table_options = BlockBasedOptions::default();
    table_options.set_block_cache(&cache);
    table_options.set_bloom_filter(10.0, false);

    let mut options = Options::default();
    options.set_block_based_table_factory(&table_options);
    options.set_write_buffer_size(1024 * 1024 * 1024);
    options.create_if_missing(true);
    options
}

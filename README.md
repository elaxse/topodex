# Topodex

## Performance metrics

Performance tests were conducted on a Apple Mac Book pro with a M1 Max processor.
The webserver was limited to 2 threads.
For the load test k6 was used on the same system. See `load-test/script.js` for details.

Metrics achieved:
- (2025-03-17) 4444 req/s with a p95 duraiton of 2.74ms while each request looked up 200 locations => 888'800 location lookups per second
- (2025-03-22) states of the world 370 -> 420 req/s by doing requests to RocksDB with multi_get
- (2025-03-23) Use multi_get to fetch the info for all requested locations at the same time 420 req/s -> 450 req/s

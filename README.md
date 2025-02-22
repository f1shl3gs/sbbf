# Split Block Bloom Filter

Implementation of [Parquet bloom filter spec](https://github.com/apache/parquet-format/blob/master/BloomFilter.md)

## Features
* Optimized with AVX2, SSE2
* Only, focus on Bloom insert and contains

## Note
User have to hash your item, and interactive with the `BloomFilter`. 
This is the best part too, cause hash performance effect the `insert`,
and `contains` a lot.

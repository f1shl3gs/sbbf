# Split Block Bloom Filter

Implementation of [Parquet bloom filter spec](https://github.com/apache/parquet-format/blob/master/BloomFilter.md)

## Features
* Ultra Lightweight, and optimized with AVX2, SSE2
* Only, focus on `insert` and `contains`, not hash

## Note
User have to hash your item, and interactive with the `Filter`.
The different hashing algorithms vary greatly, and their performance
also varies greatly. Therefore, you can optimize your code by 
choosing different hashing algorithms.

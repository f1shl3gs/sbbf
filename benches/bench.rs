use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use sbbf::BloomFilter;

fn bloom(c: &mut Criterion) {
    let total = 1024 * 1024;

    c.bench_function("insert", |b| {
        let mut rng = rand::rng();
        let mut filter = BloomFilter::new(16, total);

        b.iter(|| {
            let hash = rng.random();
            filter.insert(hash);
        })
    });

    c.bench_function("contains", |b| {
        let mut rng = rand::rng();
        let mut filter = BloomFilter::new(16, total);

        for _ in 0..total {
            let hash = rng.random();
            filter.insert(hash);
        }

        b.iter(|| {
            let hash = rng.random();
            filter.contains(hash);
        })
    });
}

criterion_group!(benches, bloom);
criterion_main!(benches);

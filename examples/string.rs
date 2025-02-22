use sbbf::Filter;
use std::hash::{DefaultHasher, Hash, Hasher};

fn main() {
    let mut filter = Filter::new(16, 1024);
    let s = String::from("hello world");

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish();

    // insert
    filter.insert(hash);

    // check
    assert!(filter.contains(hash));
}

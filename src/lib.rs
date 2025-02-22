#![allow(unsafe_op_in_unsafe_fn)]

use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::ptr::NonNull;

#[cfg(feature = "avx")]
mod avx2;
#[cfg(feature = "sse")]
mod sse;

#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    all(target_arch = "aarch64", target_endian = "little"),
    all(
        target_family = "wasm",
        target_feature = "simd128",
        target_endian = "little"
    )
))]
const SALT: [u32; 8] = [
    0x47b6137b, 0x44974d91, 0x8824ad5b, 0xa2b7289d, 0x705495c7, 0x2df1424b, 0x9efc4947, 0x5c6bfb31,
];

const ALIGNMENT: usize = 64;
const BUCKET_SIZE: usize = 32;

pub struct Filter {
    buf: NonNull<u8>,
    size: usize,
    buckets: usize,
}

impl Clone for Filter {
    fn clone(&self) -> Self {
        let buf = unsafe {
            let ptr = alloc_zeroed(Layout::from_size_align_unchecked(self.size, ALIGNMENT));
            NonNull::new_unchecked(ptr)
        };

        Self {
            buf,
            size: self.size,
            buckets: self.buckets,
        }
    }
}

impl Drop for Filter {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.buf.as_ptr(),
                Layout::from_size_align_unchecked(self.size, ALIGNMENT),
            )
        }
    }
}

impl Filter {
    pub fn new(bits: usize, keys: usize) -> Self {
        let len = bits * keys / 8;
        let len = ((len + ALIGNMENT / 2) / ALIGNMENT) * ALIGNMENT;
        let buckets = len / BUCKET_SIZE;

        let layout = Layout::from_size_align(len, ALIGNMENT).unwrap();
        let size = layout.size();
        let buf = unsafe {
            let ptr = alloc_zeroed(layout);
            NonNull::new_unchecked(ptr)
        };

        Self { buf, size, buckets }
    }
}

#[cfg(feature = "avx")]
impl Filter {
    #[inline]
    pub fn insert(&mut self, hash: u64) {
        unsafe {
            _ = avx2::insert(self.buf.as_ptr(), self.buckets, hash);
        }
    }

    #[inline]
    pub fn contains(&self, hash: u64) -> bool {
        unsafe { avx2::contains(self.buf.as_ptr(), self.buckets, hash) }
    }
}

#[cfg(feature = "sse")]
impl Filter {
    #[inline]
    pub fn insert(&mut self, hash: u64) {
        unsafe {
            _ = sse::insert(self.buf.as_ptr(), self.buckets, hash);
        }
    }

    #[inline]
    pub fn contains(&self, hash: u64) -> bool {
        unsafe { sse::contains(self.buf.as_ptr(), self.buckets, hash) }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rand::Rng;

    use super::*;

    #[test]
    fn simple() {
        let total = 1024 * 1024;
        let mut rng = rand::rng();

        let mut filter = Filter::new(64, total);
        let mut hashes = HashSet::with_capacity(total);

        for _ in 0..total {
            let hash = rng.random();

            hashes.insert(hash);

            filter.insert(hash);
            assert!(filter.contains(hash));
        }

        let mut fp = 0;
        for _ in 0..total {
            let hash = loop {
                let hash = rng.random();
                if !hashes.contains(&hash) {
                    break hash;
                }
            };

            if filter.contains(hash) {
                fp += 1;
            }
        }

        let ratio = fp as f64 / total as f64;
        println!("total: {}", total);
        println!("fp:    {}", fp);
        println!("ratio: {}", ratio);

        assert!(ratio < 0.000001);
    }

    fn run(bits: usize, max_false_positive: f64) {
        let keys = 1024 * 1024;
        let mut rng = rand::rng();
        let mut filter = Filter::new(bits, keys);
        let mut hashes = HashSet::with_capacity(keys);

        // insert
        for _ in 0..keys {
            let hash = rng.random();

            hashes.insert(hash);
            filter.insert(hash);
            assert!(filter.contains(hash));
        }

        let mut fp = 0;
        for k in hashes {
            if !filter.contains(k) {
                fp += 1;
            }
        }

        let ratio = fp as f64 / keys as f64;
        assert!(
            ratio <= max_false_positive,
            "false positive ratio: {} should less than {}",
            ratio,
            max_false_positive
        );
    }

    #[test]
    fn false_positive() {
        run(24, 0.0002);
        run(16, 0.002);
        run(8, 0.02);
    }
}

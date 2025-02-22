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

/// Given a value "word", produces an integer in [0,p) without division.
#[inline(always)]
fn fastrange(word: u32, p: u32) -> u32 {
    ((u64::from(word) * u64::from(p)) >> 32) as u32
}

pub struct BloomFilter {
    buf: NonNull<u8>,
    size: usize,
    buckets: usize,
}

impl Drop for BloomFilter {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.buf.as_ptr(),
                Layout::from_size_align_unchecked(self.size, ALIGNMENT),
            )
        }
    }
}

impl BloomFilter {
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
impl BloomFilter {
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
impl BloomFilter {
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
    use super::*;
    use rand::Rng;
    use std::collections::HashSet;

    #[test]
    fn simple() {
        let total = 1024 * 1024;
        let mut rng = rand::rng();

        let mut filter = BloomFilter::new(64, total);
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

        println!("total: {}", total);
        println!("fp:    {}", fp);
        println!("ratio: {}", fp as f64 / total as f64);
    }
}

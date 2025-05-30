#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, _mm_add_epi32, _mm_castsi128_ps, _mm_cvtps_epi32, _mm_mullo_epi32, _mm_or_si128,
    _mm_set1_epi32, _mm_setr_epi32, _mm_slli_epi32, _mm_srli_epi32, _mm_storeu_si128,
    _mm_testc_si128,
};

use super::SALT;

// taken and adapted from https://stackoverflow.com/questions/57454416/sse-integer-2n-powers-of-2-for-32-bit-integers-without-avx2
#[inline]
unsafe fn power_of_two(b: __m128i) -> __m128i {
    let exp = _mm_add_epi32(b, _mm_set1_epi32(127));
    let f = _mm_castsi128_ps(_mm_slli_epi32(exp, 23));
    _mm_cvtps_epi32(f)
}

#[inline]
unsafe fn make_mask(hash: u32) -> (__m128i, __m128i) {
    let salt = (
        _mm_setr_epi32(
            SALT[0] as i32,
            SALT[1] as i32,
            SALT[2] as i32,
            SALT[3] as i32,
        ),
        _mm_setr_epi32(
            SALT[4] as i32,
            SALT[5] as i32,
            SALT[6] as i32,
            SALT[7] as i32,
        ),
    );
    let hash = _mm_set1_epi32(hash as i32);
    let mut acc = (_mm_mullo_epi32(salt.0, hash), _mm_mullo_epi32(salt.1, hash));
    acc = (_mm_srli_epi32(acc.0, 27), _mm_srli_epi32(acc.1, 27));
    (power_of_two(acc.0), power_of_two(acc.1))
}

#[target_feature(enable = "sse4.1")]
#[inline]
pub unsafe fn contains(buf: *const u8, buckets: usize, hash: u64) -> bool {
    let bucket_idx =
        ((u64::from(hash.rotate_left(32) as u32) * u64::from(buckets as u32)) >> 32) as u32;

    let mask = make_mask(hash as u32);
    let bucket = (buf as *const __m128i).add((bucket_idx * 2) as usize);
    _mm_testc_si128(*bucket, mask.0) != 0 && _mm_testc_si128(*bucket.add(1), mask.1) != 0
}

#[target_feature(enable = "sse4.1")]
#[inline]
pub unsafe fn insert(buf: *mut u8, buckets: usize, hash: u64) -> bool {
    let bucket_idx =
        ((u64::from(hash.rotate_left(32) as u32) * u64::from(buckets as u32)) >> 32) as u32;

    let mask = make_mask(hash as u32);
    let bucket = (buf as *mut __m128i).add((bucket_idx * 2) as usize);
    _mm_storeu_si128(bucket, _mm_or_si128(*bucket, mask.0));
    let res = _mm_testc_si128(*bucket, mask.0) != 0 && _mm_testc_si128(*bucket.add(1), mask.1) != 0;
    _mm_storeu_si128(bucket.add(1), _mm_or_si128(*bucket.add(1), mask.1));
    res
}

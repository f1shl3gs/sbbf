#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm_add_epi32, _mm_castsi128_ps, _mm_cvtps_epi32, _mm_mullo_epi32,
    _mm_or_si128, _mm_set1_epi32, _mm_setr_epi32, _mm_slli_epi32, _mm_srli_epi32, _mm_storeu_si128,
    _mm_testc_si128, _mm256_load_si256, _mm256_mullo_epi32, _mm256_or_si256, _mm256_set1_epi32,
    _mm256_setr_epi32, _mm256_sllv_epi32, _mm256_srli_epi32, _mm256_store_si256,
    _mm256_testc_si256,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m256i, _mm256_load_si256, _mm256_mullo_epi32, _mm256_or_si256, _mm256_set1_epi32,
    _mm256_setr_epi32, _mm256_sllv_epi32, _mm256_srli_epi32, _mm256_store_si256,
    _mm256_testc_si256,
};

use super::{SALT, fastrange};

#[target_feature(enable = "avx2")]
#[inline]
unsafe fn make_mask(hash: u32) -> __m256i {
    let salt = _mm256_setr_epi32(
        SALT[0] as i32,
        SALT[1] as i32,
        SALT[2] as i32,
        SALT[3] as i32,
        SALT[4] as i32,
        SALT[5] as i32,
        SALT[6] as i32,
        SALT[7] as i32,
    );

    let mut acc = _mm256_set1_epi32(hash as i32);
    acc = _mm256_mullo_epi32(salt, acc);
    acc = _mm256_srli_epi32(acc, 27);
    _mm256_sllv_epi32(_mm256_set1_epi32(1), acc)
}

#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn contains(buf: *const u8, num_buckets: usize, hash: u64) -> bool {
    let bucket_idx = fastrange(hash.rotate_left(32) as u32, num_buckets as u32);
    let mask = make_mask(hash as u32);
    let bucket = (buf as *const __m256i).add(bucket_idx as usize);
    _mm256_testc_si256(_mm256_load_si256(bucket), mask) != 0
}

#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn insert(buf: *mut u8, num_buckets: usize, hash: u64) -> bool {
    let bucket_idx = fastrange(hash.rotate_left(32) as u32, num_buckets as u32);
    let mask = make_mask(hash as u32);
    let bucket = (buf as *mut __m256i).add(bucket_idx as usize);
    let val = _mm256_load_si256(bucket);
    let res = _mm256_testc_si256(val, mask) != 0;
    _mm256_store_si256(bucket, _mm256_or_si256(val, mask));
    res
}

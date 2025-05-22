/// Stolen From: https://github.com/TeamDman/Cursor-Hero/blob/51611380997d74f74f76fa776be4892a9906c005/crates/math/src/shuffle.rs
/// License: MPL-2.0 (https://github.com/TeamDman/Cursor-Hero/blob/main/LICENSE)

#[cfg(target_arch = "x86")]
use std::arch::x86::_mm_shuffle_epi8;
use std::arch::x86_64::__m128i;
use std::arch::x86_64::_mm_loadu_si128;
use std::arch::x86_64::_mm_setr_epi8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm_shuffle_epi8;
use std::arch::x86_64::_mm_storeu_si128;

/// Convert BGRA to RGBA
///
/// Uses SIMD to go fast
pub fn bgra_to_rgba(data: &mut [u8]) {
    // The shuffle mask for converting BGRA -> RGBA
    let mask: __m128i = unsafe {
        _mm_setr_epi8(
            2, 1, 0, 3, // First pixel
            6, 5, 4, 7, // Second pixel
            10, 9, 8, 11, // Third pixel
            14, 13, 12, 15, // Fourth pixel
        )
    };
    // For each 16-byte chunk in your data
    for chunk in data.chunks_exact_mut(16) {
        let mut vector = unsafe { _mm_loadu_si128(chunk.as_ptr() as *const __m128i) };
        vector = unsafe { _mm_shuffle_epi8(vector, mask) };
        unsafe { _mm_storeu_si128(chunk.as_mut_ptr() as *mut __m128i, vector) };
    }
}

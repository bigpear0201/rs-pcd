// Copyright 2025 bigpear0201

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Platform-optimized endianness conversion utilities.
//!
//! PCD files are always Little Endian. On LE platforms, we use zero-copy
//! `memcpy` via `copy_nonoverlapping`. On BE platforms, we fall back to
//! byte-swapping via `byteorder`.

// ========================
// Decode (bytes → typed slice)
// ========================

#[cfg(target_endian = "little")]
#[inline]
pub fn decode_f32_slice(src: &[u8], dest: &mut [f32]) {
    debug_assert!(src.len() >= dest.len() * 4);
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr() as *mut u8, dest.len() * 4);
    }
}

#[cfg(not(target_endian = "little"))]
#[inline]
pub fn decode_f32_slice(src: &[u8], dest: &mut [f32]) {
    use byteorder::{ByteOrder, LittleEndian};
    for (i, chunk) in src.chunks_exact(4).enumerate() {
        dest[i] = LittleEndian::read_f32(chunk);
    }
}

#[cfg(target_endian = "little")]
#[inline]
pub fn decode_f64_slice(src: &[u8], dest: &mut [f64]) {
    debug_assert!(src.len() >= dest.len() * 8);
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr() as *mut u8, dest.len() * 8);
    }
}

#[cfg(not(target_endian = "little"))]
#[inline]
pub fn decode_f64_slice(src: &[u8], dest: &mut [f64]) {
    use byteorder::{ByteOrder, LittleEndian};
    for (i, chunk) in src.chunks_exact(8).enumerate() {
        dest[i] = LittleEndian::read_f64(chunk);
    }
}

#[cfg(target_endian = "little")]
#[inline]
pub fn decode_u16_slice(src: &[u8], dest: &mut [u16]) {
    debug_assert!(src.len() >= dest.len() * 2);
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr() as *mut u8, dest.len() * 2);
    }
}

#[cfg(not(target_endian = "little"))]
#[inline]
pub fn decode_u16_slice(src: &[u8], dest: &mut [u16]) {
    use byteorder::{ByteOrder, LittleEndian};
    for (i, chunk) in src.chunks_exact(2).enumerate() {
        dest[i] = LittleEndian::read_u16(chunk);
    }
}

#[cfg(target_endian = "little")]
#[inline]
pub fn decode_i16_slice(src: &[u8], dest: &mut [i16]) {
    debug_assert!(src.len() >= dest.len() * 2);
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr() as *mut u8, dest.len() * 2);
    }
}

#[cfg(not(target_endian = "little"))]
#[inline]
pub fn decode_i16_slice(src: &[u8], dest: &mut [i16]) {
    use byteorder::{ByteOrder, LittleEndian};
    for (i, chunk) in src.chunks_exact(2).enumerate() {
        dest[i] = LittleEndian::read_i16(chunk);
    }
}

#[cfg(target_endian = "little")]
#[inline]
pub fn decode_u32_slice(src: &[u8], dest: &mut [u32]) {
    debug_assert!(src.len() >= dest.len() * 4);
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr() as *mut u8, dest.len() * 4);
    }
}

#[cfg(not(target_endian = "little"))]
#[inline]
pub fn decode_u32_slice(src: &[u8], dest: &mut [u32]) {
    use byteorder::{ByteOrder, LittleEndian};
    for (i, chunk) in src.chunks_exact(4).enumerate() {
        dest[i] = LittleEndian::read_u32(chunk);
    }
}

#[cfg(target_endian = "little")]
#[inline]
pub fn decode_i32_slice(src: &[u8], dest: &mut [i32]) {
    debug_assert!(src.len() >= dest.len() * 4);
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr() as *mut u8, dest.len() * 4);
    }
}

#[cfg(not(target_endian = "little"))]
#[inline]
pub fn decode_i32_slice(src: &[u8], dest: &mut [i32]) {
    use byteorder::{ByteOrder, LittleEndian};
    for (i, chunk) in src.chunks_exact(4).enumerate() {
        dest[i] = LittleEndian::read_i32(chunk);
    }
}

// ========================
// Encode (typed slice → bytes)
// ========================

/// Write a column's data as LE bytes into a Vec.
/// On LE platforms, uses zero-copy memcpy.
#[cfg(target_endian = "little")]
#[inline]
pub fn encode_f32_slice(src: &[f32], dest: &mut Vec<u8>) {
    let bytes =
        unsafe { std::slice::from_raw_parts(src.as_ptr() as *const u8, src.len() * 4) };
    dest.extend_from_slice(bytes);
}

#[cfg(not(target_endian = "little"))]
#[inline]
pub fn encode_f32_slice(src: &[f32], dest: &mut Vec<u8>) {
    use byteorder::{LittleEndian, WriteBytesExt};
    for &val in src {
        dest.write_f32::<LittleEndian>(val).unwrap();
    }
}

#[cfg(target_endian = "little")]
#[inline]
pub fn encode_f64_slice(src: &[f64], dest: &mut Vec<u8>) {
    let bytes =
        unsafe { std::slice::from_raw_parts(src.as_ptr() as *const u8, src.len() * 8) };
    dest.extend_from_slice(bytes);
}

#[cfg(not(target_endian = "little"))]
#[inline]
pub fn encode_f64_slice(src: &[f64], dest: &mut Vec<u8>) {
    use byteorder::{LittleEndian, WriteBytesExt};
    for &val in src {
        dest.write_f64::<LittleEndian>(val).unwrap();
    }
}

#[cfg(target_endian = "little")]
#[inline]
pub fn encode_u16_slice(src: &[u16], dest: &mut Vec<u8>) {
    let bytes =
        unsafe { std::slice::from_raw_parts(src.as_ptr() as *const u8, src.len() * 2) };
    dest.extend_from_slice(bytes);
}

#[cfg(not(target_endian = "little"))]
#[inline]
pub fn encode_u16_slice(src: &[u16], dest: &mut Vec<u8>) {
    use byteorder::{LittleEndian, WriteBytesExt};
    for &val in src {
        dest.write_u16::<LittleEndian>(val).unwrap();
    }
}

#[cfg(target_endian = "little")]
#[inline]
pub fn encode_i16_slice(src: &[i16], dest: &mut Vec<u8>) {
    let bytes =
        unsafe { std::slice::from_raw_parts(src.as_ptr() as *const u8, src.len() * 2) };
    dest.extend_from_slice(bytes);
}

#[cfg(not(target_endian = "little"))]
#[inline]
pub fn encode_i16_slice(src: &[i16], dest: &mut Vec<u8>) {
    use byteorder::{LittleEndian, WriteBytesExt};
    for &val in src {
        dest.write_i16::<LittleEndian>(val).unwrap();
    }
}

#[cfg(target_endian = "little")]
#[inline]
pub fn encode_u32_slice(src: &[u32], dest: &mut Vec<u8>) {
    let bytes =
        unsafe { std::slice::from_raw_parts(src.as_ptr() as *const u8, src.len() * 4) };
    dest.extend_from_slice(bytes);
}

#[cfg(not(target_endian = "little"))]
#[inline]
pub fn encode_u32_slice(src: &[u32], dest: &mut Vec<u8>) {
    use byteorder::{LittleEndian, WriteBytesExt};
    for &val in src {
        dest.write_u32::<LittleEndian>(val).unwrap();
    }
}

#[cfg(target_endian = "little")]
#[inline]
pub fn encode_i32_slice(src: &[i32], dest: &mut Vec<u8>) {
    let bytes =
        unsafe { std::slice::from_raw_parts(src.as_ptr() as *const u8, src.len() * 4) };
    dest.extend_from_slice(bytes);
}

#[cfg(not(target_endian = "little"))]
#[inline]
pub fn encode_i32_slice(src: &[i32], dest: &mut Vec<u8>) {
    use byteorder::{LittleEndian, WriteBytesExt};
    for &val in src {
        dest.write_i32::<LittleEndian>(val).unwrap();
    }
}

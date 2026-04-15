// Copyright 2025 bigpear0201
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(not(target_endian = "little"))]
use byteorder::{ByteOrder, LittleEndian};

#[inline]
pub(crate) fn decode_i8_slice(src: &[u8], dest: &mut [i8]) {
    for (value, byte) in dest.iter_mut().zip(src.iter().copied()) {
        *value = byte as i8;
    }
}

#[cfg(target_endian = "little")]
macro_rules! impl_copy_decode {
    ($name:ident, $ty:ty, $bytes:expr) => {
        #[inline]
        pub(crate) fn $name(src: &[u8], dest: &mut [$ty]) {
            assert!(src.len() >= dest.len() * $bytes);
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src.as_ptr(),
                    dest.as_mut_ptr() as *mut u8,
                    dest.len() * $bytes,
                );
            }
        }
    };
}

#[cfg(not(target_endian = "little"))]
macro_rules! impl_read_decode {
    ($name:ident, $read_fn:ident, $ty:ty, $bytes:expr) => {
        #[inline]
        pub(crate) fn $name(src: &[u8], dest: &mut [$ty]) {
            for (i, chunk) in src.chunks_exact($bytes).enumerate() {
                dest[i] = LittleEndian::$read_fn(chunk);
            }
        }
    };
}

#[cfg(target_endian = "little")]
impl_copy_decode!(decode_f32_slice, f32, 4);
#[cfg(not(target_endian = "little"))]
impl_read_decode!(decode_f32_slice, read_f32, f32, 4);

#[cfg(target_endian = "little")]
impl_copy_decode!(decode_f64_slice, f64, 8);
#[cfg(not(target_endian = "little"))]
impl_read_decode!(decode_f64_slice, read_f64, f64, 8);

#[cfg(target_endian = "little")]
impl_copy_decode!(decode_u16_slice, u16, 2);
#[cfg(not(target_endian = "little"))]
impl_read_decode!(decode_u16_slice, read_u16, u16, 2);

#[cfg(target_endian = "little")]
impl_copy_decode!(decode_i16_slice, i16, 2);
#[cfg(not(target_endian = "little"))]
impl_read_decode!(decode_i16_slice, read_i16, i16, 2);

#[cfg(target_endian = "little")]
impl_copy_decode!(decode_u32_slice, u32, 4);
#[cfg(not(target_endian = "little"))]
impl_read_decode!(decode_u32_slice, read_u32, u32, 4);

#[cfg(target_endian = "little")]
impl_copy_decode!(decode_i32_slice, i32, 4);
#[cfg(not(target_endian = "little"))]
impl_read_decode!(decode_i32_slice, read_i32, i32, 4);

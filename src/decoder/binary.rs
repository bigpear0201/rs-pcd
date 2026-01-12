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

use crate::error::{PcdError, Result};
use crate::header::ValueType;
use crate::layout::PcdLayout;
use crate::storage::PointBlock;
use std::io::Read;

/// Batch size for buffered reading - minimizes syscalls while keeping memory footprint reasonable
const BATCH_SIZE: usize = 1024;

pub struct BinaryReader<'a, R: Read> {
    reader: &'a mut R,
    layout: &'a PcdLayout,
    points_to_read: usize,
}

impl<'a, R: Read> BinaryReader<'a, R> {
    pub fn new(reader: &'a mut R, layout: &'a PcdLayout, points_to_read: usize) -> Self {
        Self {
            reader,
            layout,
            points_to_read,
        }
    }

    pub fn decode(&mut self, output: &mut PointBlock) -> Result<()> {
        let required_cols: Vec<String> =
            self.layout.fields.iter().map(|f| f.name.clone()).collect();

        // Ensure all columns exist
        for name in &required_cols {
            if output.get_column(name).is_none() {
                return Err(PcdError::LayoutMismatch {
                    expected: 0,
                    got: 0,
                });
            }
        }

        output.resize(self.points_to_read);

        // Get mutable references to all columns at once
        let mut columns = output.get_columns_mut(&required_cols).ok_or_else(|| {
            PcdError::Other("Failed to acquire columns mutable borrow".to_string())
        })?;

        let point_step = self.layout.total_size;
        
        // Batch read optimization: read multiple points at once to reduce syscalls
        let batch_bytes = point_step * BATCH_SIZE;
        let mut batch_buffer = vec![0u8; batch_bytes];

        let mut point_idx = 0;
        while point_idx < self.points_to_read {
            let batch_end = (point_idx + BATCH_SIZE).min(self.points_to_read);
            let points_in_batch = batch_end - point_idx;
            let read_size = points_in_batch * point_step;

            self.reader.read_exact(&mut batch_buffer[..read_size])?;

            // Process all points in this batch
            for batch_offset in 0..points_in_batch {
                let buffer_start = batch_offset * point_step;
                let i = point_idx + batch_offset;

                for (field_idx, field) in self.layout.fields.iter().enumerate() {
                    let col = &mut columns[field_idx];
                    let start = buffer_start + field.offset;
                    let end = start + field.size;
                    let data = &batch_buffer[start..end];
                    let dest_start = i * field.count;

                    decode_field(col, field.type_, field.count, data, dest_start);
                }
            }

            point_idx = batch_end;
        }

        Ok(())
    }
}

/// Decode a single field from raw bytes into the column.
/// Uses platform-optimized path for Little Endian systems.
#[inline]
fn decode_field(
    col: &mut crate::storage::Column,
    value_type: ValueType,
    count: usize,
    data: &[u8],
    dest_start: usize,
) {
    match value_type {
        ValueType::U8 => {
            let vec = col.as_u8_mut().unwrap();
            vec[dest_start..dest_start + count].copy_from_slice(data);
        }
        ValueType::I8 => {
            let vec = col.as_i8_mut().unwrap();
            for (k, &b) in data.iter().enumerate() {
                vec[dest_start + k] = b as i8;
            }
        }
        ValueType::U16 => {
            let vec = col.as_u16_mut().unwrap();
            decode_u16_slice(&data[..count * 2], &mut vec[dest_start..dest_start + count]);
        }
        ValueType::I16 => {
            let vec = col.as_i16_mut().unwrap();
            decode_i16_slice(&data[..count * 2], &mut vec[dest_start..dest_start + count]);
        }
        ValueType::U32 => {
            let vec = col.as_u32_mut().unwrap();
            decode_u32_slice(&data[..count * 4], &mut vec[dest_start..dest_start + count]);
        }
        ValueType::I32 => {
            let vec = col.as_i32_mut().unwrap();
            decode_i32_slice(&data[..count * 4], &mut vec[dest_start..dest_start + count]);
        }
        ValueType::F32 => {
            let vec = col.as_f32_mut().unwrap();
            decode_f32_slice(&data[..count * 4], &mut vec[dest_start..dest_start + count]);
        }
        ValueType::F64 => {
            let vec = col.as_f64_mut().unwrap();
            decode_f64_slice(&data[..count * 8], &mut vec[dest_start..dest_start + count]);
        }
    }
}

// Platform-optimized decode functions
// On Little Endian platforms, we can use direct memory copy for significant speedup

#[cfg(target_endian = "little")]
#[inline]
fn decode_f32_slice(src: &[u8], dest: &mut [f32]) {
    // Safety: src length is pre-validated, and f32 is 4 bytes
    // On LE platforms, the byte order matches, so direct copy is valid
    assert!(src.len() >= dest.len() * 4);
    unsafe {
        std::ptr::copy_nonoverlapping(
            src.as_ptr(),
            dest.as_mut_ptr() as *mut u8,
            dest.len() * 4,
        );
    }
}

#[cfg(not(target_endian = "little"))]
#[inline]
fn decode_f32_slice(src: &[u8], dest: &mut [f32]) {
    use byteorder::{ByteOrder, LittleEndian};
    for (i, chunk) in src.chunks_exact(4).enumerate() {
        dest[i] = LittleEndian::read_f32(chunk);
    }
}

#[cfg(target_endian = "little")]
#[inline]
fn decode_f64_slice(src: &[u8], dest: &mut [f64]) {
    assert!(src.len() >= dest.len() * 8);
    unsafe {
        std::ptr::copy_nonoverlapping(
            src.as_ptr(),
            dest.as_mut_ptr() as *mut u8,
            dest.len() * 8,
        );
    }
}

#[cfg(not(target_endian = "little"))]
#[inline]
fn decode_f64_slice(src: &[u8], dest: &mut [f64]) {
    use byteorder::{ByteOrder, LittleEndian};
    for (i, chunk) in src.chunks_exact(8).enumerate() {
        dest[i] = LittleEndian::read_f64(chunk);
    }
}

#[cfg(target_endian = "little")]
#[inline]
fn decode_u16_slice(src: &[u8], dest: &mut [u16]) {
    assert!(src.len() >= dest.len() * 2);
    unsafe {
        std::ptr::copy_nonoverlapping(
            src.as_ptr(),
            dest.as_mut_ptr() as *mut u8,
            dest.len() * 2,
        );
    }
}

#[cfg(not(target_endian = "little"))]
#[inline]
fn decode_u16_slice(src: &[u8], dest: &mut [u16]) {
    use byteorder::{ByteOrder, LittleEndian};
    for (i, chunk) in src.chunks_exact(2).enumerate() {
        dest[i] = LittleEndian::read_u16(chunk);
    }
}

#[cfg(target_endian = "little")]
#[inline]
fn decode_i16_slice(src: &[u8], dest: &mut [i16]) {
    assert!(src.len() >= dest.len() * 2);
    unsafe {
        std::ptr::copy_nonoverlapping(
            src.as_ptr(),
            dest.as_mut_ptr() as *mut u8,
            dest.len() * 2,
        );
    }
}

#[cfg(not(target_endian = "little"))]
#[inline]
fn decode_i16_slice(src: &[u8], dest: &mut [i16]) {
    use byteorder::{ByteOrder, LittleEndian};
    for (i, chunk) in src.chunks_exact(2).enumerate() {
        dest[i] = LittleEndian::read_i16(chunk);
    }
}

#[cfg(target_endian = "little")]
#[inline]
fn decode_u32_slice(src: &[u8], dest: &mut [u32]) {
    assert!(src.len() >= dest.len() * 4);
    unsafe {
        std::ptr::copy_nonoverlapping(
            src.as_ptr(),
            dest.as_mut_ptr() as *mut u8,
            dest.len() * 4,
        );
    }
}

#[cfg(not(target_endian = "little"))]
#[inline]
fn decode_u32_slice(src: &[u8], dest: &mut [u32]) {
    use byteorder::{ByteOrder, LittleEndian};
    for (i, chunk) in src.chunks_exact(4).enumerate() {
        dest[i] = LittleEndian::read_u32(chunk);
    }
}

#[cfg(target_endian = "little")]
#[inline]
fn decode_i32_slice(src: &[u8], dest: &mut [i32]) {
    assert!(src.len() >= dest.len() * 4);
    unsafe {
        std::ptr::copy_nonoverlapping(
            src.as_ptr(),
            dest.as_mut_ptr() as *mut u8,
            dest.len() * 4,
        );
    }
}

#[cfg(not(target_endian = "little"))]
#[inline]
fn decode_i32_slice(src: &[u8], dest: &mut [i32]) {
    use byteorder::{ByteOrder, LittleEndian};
    for (i, chunk) in src.chunks_exact(4).enumerate() {
        dest[i] = LittleEndian::read_i32(chunk);
    }
}

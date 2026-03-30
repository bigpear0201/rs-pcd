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

use super::endian;
use crate::error::{PcdError, Result};
use crate::header::ValueType;
use crate::layout::{FieldLayout, PcdLayout};
use crate::storage::Column;
use crate::storage::PointBlock;
use rayon::prelude::*;

pub struct BinaryParallelDecoder<'a> {
    layout: &'a PcdLayout,
    points: usize,
}

impl<'a> BinaryParallelDecoder<'a> {
    pub fn new(layout: &'a PcdLayout, points: usize) -> Self {
        Self { layout, points }
    }

    /// Decode binary PCD data in parallel.
    ///
    /// Safety approach: instead of using raw pointers with SyncPtr, we parallelize
    /// over columns. Each column is independently &mut borrowed, so no data races.
    /// The input data (AoS) is shared read-only across all threads.
    pub fn decode_par(&self, data: &[u8], output: &mut PointBlock) -> Result<()> {
        let point_step = self.layout.total_size;
        let total_bytes = point_step * self.points;
        if data.len() < total_bytes {
            return Err(PcdError::BufferTooSmall {
                expected: total_bytes,
                got: data.len(),
            });
        }

        output.resize(self.points);

        let fields = &self.layout.fields;
        let columns = output.columns_mut();
        let points = self.points;

        // Parallel over columns: each thread owns one &mut Column exclusively.
        // Input data is &[u8] (shared, read-only). No data races possible.
        columns
            .par_iter_mut()
            .zip(fields.par_iter())
            .for_each(|(col, field)| {
                decode_column_from_aos(col, field, data, point_step, points);
            });

        Ok(())
    }
}

/// Decode one column's data from AoS input into the column's SoA storage.
/// Each call processes all points for one field — safe because columns don't overlap.
fn decode_column_from_aos(
    col: &mut Column,
    field: &FieldLayout,
    data: &[u8],
    point_step: usize,
    points: usize,
) {
    match field.type_ {
        ValueType::U8 => {
            let vec = col.as_u8_mut().unwrap();
            for i in 0..points {
                let src_offset = i * point_step + field.offset;
                for k in 0..field.count {
                    vec[i * field.count + k] = data[src_offset + k];
                }
            }
        }
        ValueType::I8 => {
            let vec = col.as_i8_mut().unwrap();
            for i in 0..points {
                let src_offset = i * point_step + field.offset;
                for k in 0..field.count {
                    vec[i * field.count + k] = data[src_offset + k] as i8;
                }
            }
        }
        ValueType::U16 => {
            let vec = col.as_u16_mut().unwrap();
            for i in 0..points {
                let src_offset = i * point_step + field.offset;
                let src = &data[src_offset..src_offset + field.count * 2];
                let dest_start = i * field.count;
                endian::decode_u16_slice(src, &mut vec[dest_start..dest_start + field.count]);
            }
        }
        ValueType::I16 => {
            let vec = col.as_i16_mut().unwrap();
            for i in 0..points {
                let src_offset = i * point_step + field.offset;
                let src = &data[src_offset..src_offset + field.count * 2];
                let dest_start = i * field.count;
                endian::decode_i16_slice(src, &mut vec[dest_start..dest_start + field.count]);
            }
        }
        ValueType::U32 => {
            let vec = col.as_u32_mut().unwrap();
            for i in 0..points {
                let src_offset = i * point_step + field.offset;
                let src = &data[src_offset..src_offset + field.count * 4];
                let dest_start = i * field.count;
                endian::decode_u32_slice(src, &mut vec[dest_start..dest_start + field.count]);
            }
        }
        ValueType::I32 => {
            let vec = col.as_i32_mut().unwrap();
            for i in 0..points {
                let src_offset = i * point_step + field.offset;
                let src = &data[src_offset..src_offset + field.count * 4];
                let dest_start = i * field.count;
                endian::decode_i32_slice(src, &mut vec[dest_start..dest_start + field.count]);
            }
        }
        ValueType::F32 => {
            let vec = col.as_f32_mut().unwrap();
            for i in 0..points {
                let src_offset = i * point_step + field.offset;
                let src = &data[src_offset..src_offset + field.count * 4];
                let dest_start = i * field.count;
                endian::decode_f32_slice(src, &mut vec[dest_start..dest_start + field.count]);
            }
        }
        ValueType::F64 => {
            let vec = col.as_f64_mut().unwrap();
            for i in 0..points {
                let src_offset = i * point_step + field.offset;
                let src = &data[src_offset..src_offset + field.count * 8];
                let dest_start = i * field.count;
                endian::decode_f64_slice(src, &mut vec[dest_start..dest_start + field.count]);
            }
        }
    }
}

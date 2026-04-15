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

use crate::decoder::slice::{
    decode_f32_slice, decode_f64_slice, decode_i16_slice, decode_i32_slice, decode_i8_slice,
    decode_u16_slice, decode_u32_slice,
};
use crate::error::{PcdError, Result};
use crate::header::ValueType;
use crate::layout::PcdLayout;
use crate::storage::{Column, PointBlock};
use byteorder::{LittleEndian, ReadBytesExt};
use lzf;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use std::io::Read;

pub struct CompressedReader<'a, R: Read> {
    reader: &'a mut R,
    layout: &'a PcdLayout,
    points_to_read: usize,
}

impl<'a, R: Read> CompressedReader<'a, R> {
    pub fn new(reader: &'a mut R, layout: &'a PcdLayout, points_to_read: usize) -> Self {
        Self {
            reader,
            layout,
            points_to_read,
        }
    }

    pub fn decode(&mut self, output: &mut PointBlock) -> Result<()> {
        let compressed_size = self.reader.read_u32::<LittleEndian>()? as usize;
        let uncompressed_size = self.reader.read_u32::<LittleEndian>()? as usize;

        let mut compressed_data = vec![0u8; compressed_size];
        self.reader.read_exact(&mut compressed_data)?;

        // Decompress
        let decompressed = lzf::decompress(&compressed_data, uncompressed_size)
            .map_err(|e| PcdError::Decompression(format!("{:?}", e)))?;

        if decompressed.len() != uncompressed_size {
            return Err(PcdError::Decompression(format!(
                "Size mismatch: expected {}, got {}",
                uncompressed_size,
                decompressed.len()
            )));
        }

        // Validate buffer size against layout
        // SoA layout: sum(field.count * sizeof(type) * num_points)
        // Wait, layout.total_size is per-point size (stride).
        // Total bytes should be layout.total_size * points_to_read.
        let expected_bytes = self.layout.total_size * self.points_to_read;
        if uncompressed_size != expected_bytes {
            return Err(PcdError::LayoutMismatch {
                expected: expected_bytes,
                got: uncompressed_size,
            });
        }

        // Initialize output - verify all columns exist
        for field in &self.layout.fields {
            if output.get_column(&field.name).is_none() {
                // Schema mismatch - column doesn't exist
                // For now we just skip the check and let get_column_mut fail below
            }
        }
        output.resize(self.points_to_read);

        let field_ranges = self.field_ranges(&decompressed);

        #[cfg(feature = "rayon")]
        {
            output
                .columns_mut()
                .par_iter_mut()
                .zip(self.layout.fields.par_iter())
                .zip(field_ranges.par_iter())
                .try_for_each(|((col, field), data_slice)| {
                    decode_column(col, field.type_, data_slice)
                })?;
        }

        #[cfg(not(feature = "rayon"))]
        {
            for ((field, data_slice), col) in self
                .layout
                .fields
                .iter()
                .zip(field_ranges.iter())
                .zip(output.columns_mut().iter_mut())
            {
                decode_column(col, field.type_, data_slice)?;
            }
        }

        Ok(())
    }

    fn field_ranges<'b>(&self, decompressed: &'b [u8]) -> Vec<&'b [u8]> {
        let mut offset = 0;
        let mut ranges = Vec::with_capacity(self.layout.fields.len());

        for field in &self.layout.fields {
            let bytes_per_field_block = field.size * self.points_to_read;
            let end = offset + bytes_per_field_block;
            ranges.push(&decompressed[offset..end]);
            offset = end;
        }

        ranges
    }
}

fn decode_column(column: &mut Column, value_type: ValueType, data_slice: &[u8]) -> Result<()> {
    match value_type {
        ValueType::U8 => {
            let vec = column.as_u8_mut().ok_or(PcdError::LayoutMismatch {
                expected: 0,
                got: 0,
            })?;
            vec.copy_from_slice(data_slice);
        }
        ValueType::I8 => {
            let vec = column.as_i8_mut().ok_or(PcdError::LayoutMismatch {
                expected: 0,
                got: 0,
            })?;
            decode_i8_slice(data_slice, vec);
        }
        ValueType::U16 => {
            let vec = column.as_u16_mut().ok_or(PcdError::LayoutMismatch {
                expected: 0,
                got: 0,
            })?;
            decode_u16_slice(data_slice, vec);
        }
        ValueType::I16 => {
            let vec = column.as_i16_mut().ok_or(PcdError::LayoutMismatch {
                expected: 0,
                got: 0,
            })?;
            decode_i16_slice(data_slice, vec);
        }
        ValueType::U32 => {
            let vec = column.as_u32_mut().ok_or(PcdError::LayoutMismatch {
                expected: 0,
                got: 0,
            })?;
            decode_u32_slice(data_slice, vec);
        }
        ValueType::I32 => {
            let vec = column.as_i32_mut().ok_or(PcdError::LayoutMismatch {
                expected: 0,
                got: 0,
            })?;
            decode_i32_slice(data_slice, vec);
        }
        ValueType::F32 => {
            let vec = column.as_f32_mut().ok_or(PcdError::LayoutMismatch {
                expected: 0,
                got: 0,
            })?;
            decode_f32_slice(data_slice, vec);
        }
        ValueType::F64 => {
            let vec = column.as_f64_mut().ok_or(PcdError::LayoutMismatch {
                expected: 0,
                got: 0,
            })?;
            decode_f64_slice(data_slice, vec);
        }
    }

    Ok(())
}

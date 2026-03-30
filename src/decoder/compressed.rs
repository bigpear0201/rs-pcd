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
use crate::layout::PcdLayout;
use crate::storage::PointBlock;
use byteorder::{LittleEndian, ReadBytesExt};
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

        // Validate total buffer size against layout
        let expected_bytes = self.layout.total_size * self.points_to_read;
        if uncompressed_size != expected_bytes {
            return Err(PcdError::LayoutMismatch {
                expected: expected_bytes,
                got: uncompressed_size,
            });
        }

        // Verify all required columns exist before proceeding
        for field in &self.layout.fields {
            if output.get_column(&field.name).is_none() {
                return Err(PcdError::InvalidDataFormat(format!(
                    "Missing required column '{}'",
                    field.name
                )));
            }
        }

        output.resize(self.points_to_read);

        // Process fields (SoA in buffer: [Field1 All Points][Field2 All Points]...)
        let mut offset = 0;

        for field in &self.layout.fields {
            let col = output
                .get_column_mut(&field.name)
                .ok_or_else(|| {
                    PcdError::InvalidDataFormat(format!("Missing column '{}'", field.name))
                })?;

            let bytes_per_element = field.element_size;
            let elements_per_point = field.count;
            let bytes_per_field_block =
                bytes_per_element * elements_per_point * self.points_to_read;

            let end = offset + bytes_per_field_block;
            let data_slice = &decompressed[offset..end];
            offset = end;

            let total_elements = elements_per_point * self.points_to_read;

            // Use shared LE-optimized decode functions
            match field.type_ {
                ValueType::U8 => {
                    let vec = col.as_u8_mut().unwrap();
                    vec[..total_elements].copy_from_slice(data_slice);
                }
                ValueType::I8 => {
                    let vec = col.as_i8_mut().unwrap();
                    for (dest, &src) in vec.iter_mut().zip(data_slice.iter()) {
                        *dest = src as i8;
                    }
                }
                ValueType::U16 => {
                    let vec = col.as_u16_mut().unwrap();
                    endian::decode_u16_slice(data_slice, &mut vec[..total_elements]);
                }
                ValueType::I16 => {
                    let vec = col.as_i16_mut().unwrap();
                    endian::decode_i16_slice(data_slice, &mut vec[..total_elements]);
                }
                ValueType::U32 => {
                    let vec = col.as_u32_mut().unwrap();
                    endian::decode_u32_slice(data_slice, &mut vec[..total_elements]);
                }
                ValueType::I32 => {
                    let vec = col.as_i32_mut().unwrap();
                    endian::decode_i32_slice(data_slice, &mut vec[..total_elements]);
                }
                ValueType::F32 => {
                    let vec = col.as_f32_mut().unwrap();
                    endian::decode_f32_slice(data_slice, &mut vec[..total_elements]);
                }
                ValueType::F64 => {
                    let vec = col.as_f64_mut().unwrap();
                    endian::decode_f64_slice(data_slice, &mut vec[..total_elements]);
                }
            }
        }

        Ok(())
    }
}

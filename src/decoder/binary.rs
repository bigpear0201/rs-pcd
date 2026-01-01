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
use byteorder::{ByteOrder, LittleEndian};
use std::io::Read;

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
            if !output.columns.contains_key(name) {
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
        let mut buffer = vec![0u8; point_step];

        for i in 0..self.points_to_read {
            self.reader.read_exact(&mut buffer)?;

            for (field_idx, field) in self.layout.fields.iter().enumerate() {
                let col = &mut columns[field_idx];
                let start = field.offset;
                let end = start + field.size;
                let data = &buffer[start..end];
                let dest_start = i * field.count;

                match field.type_ {
                    ValueType::U8 => {
                        let vec = col.as_u8_mut().unwrap();
                        vec[dest_start..dest_start + field.count].copy_from_slice(data);
                    }
                    ValueType::I8 => {
                        let vec = col.as_i8_mut().unwrap();
                        // I8 is same size as U8, just cast bytes
                        for (k, &b) in data.iter().enumerate() {
                            vec[dest_start + k] = b as i8;
                        }
                    }
                    ValueType::U16 => {
                        let vec = col.as_u16_mut().unwrap();
                        for k in 0..field.count {
                            let offset = k * 2;
                            vec[dest_start + k] = LittleEndian::read_u16(&data[offset..offset + 2]);
                        }
                    }
                    ValueType::I16 => {
                        let vec = col.as_i16_mut().unwrap();
                        for k in 0..field.count {
                            let offset = k * 2;
                            vec[dest_start + k] = LittleEndian::read_i16(&data[offset..offset + 2]);
                        }
                    }
                    ValueType::U32 => {
                        let vec = col.as_u32_mut().unwrap();
                        for k in 0..field.count {
                            let offset = k * 4;
                            vec[dest_start + k] = LittleEndian::read_u32(&data[offset..offset + 4]);
                        }
                    }
                    ValueType::I32 => {
                        let vec = col.as_i32_mut().unwrap();
                        for k in 0..field.count {
                            let offset = k * 4;
                            vec[dest_start + k] = LittleEndian::read_i32(&data[offset..offset + 4]);
                        }
                    }
                    ValueType::F32 => {
                        let vec = col.as_f32_mut().unwrap();
                        for k in 0..field.count {
                            let offset = k * 4;
                            vec[dest_start + k] = LittleEndian::read_f32(&data[offset..offset + 4]);
                        }
                    }
                    ValueType::F64 => {
                        let vec = col.as_f64_mut().unwrap();
                        for k in 0..field.count {
                            let offset = k * 8;
                            vec[dest_start + k] = LittleEndian::read_f64(&data[offset..offset + 8]);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

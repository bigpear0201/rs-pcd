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
use rayon::prelude::*;

// Wrapper to make raw pointers Sync+Send for Rayon
struct SyncPtr(*mut u8);
unsafe impl Sync for SyncPtr {}
unsafe impl Send for SyncPtr {}

pub struct BinaryParallelDecoder<'a> {
    layout: &'a PcdLayout,
    points: usize,
}

impl<'a> BinaryParallelDecoder<'a> {
    pub fn new(layout: &'a PcdLayout, points: usize) -> Self {
        Self { layout, points }
    }

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

        // Collect raw pointers for columns
        let mut col_ptrs = Vec::new();
        for field in &self.layout.fields {
            if let Some(col) = output.get_column_mut(&field.name) {
                let (ptr, _len_bytes) = unsafe { col.as_ptr_mut() };
                // Calculate length in elements (already consistent with resize)
                let len = col.len();
                col_ptrs.push((field, SyncPtr(ptr), len, field.type_));
            }
        }

        // Rayon parallel loop
        // Input data is AoS. size = points * stride.
        // We iterate over chunks of bytes corresponding to points concurrently.
        data.par_chunks_exact(point_step)
            .enumerate()
            .for_each(|(i, point_data)| {
                for (field, ptr_wrapper, len, vtype) in &col_ptrs {
                    let ptr = ptr_wrapper.0;

                    if i >= *len {
                        continue;
                    }

                    let field_offset_in_point = field.offset;
                    let src_slice =
                        &point_data[field_offset_in_point..field_offset_in_point + field.size];

                    match vtype {
                        ValueType::U8 => {
                            let u8_ptr = ptr;
                            for k in 0..field.count {
                                unsafe {
                                    *u8_ptr.add(i * field.count + k) = src_slice[k];
                                }
                            }
                        }
                        ValueType::I8 => {
                            let i8_ptr = ptr as *mut i8;
                            for k in 0..field.count {
                                unsafe {
                                    *i8_ptr.add(i * field.count + k) = src_slice[k] as i8;
                                }
                            }
                        }
                        ValueType::U16 => {
                            let u16_ptr = ptr as *mut u16;
                            for k in 0..field.count {
                                let offset = k * 2;
                                let val = LittleEndian::read_u16(&src_slice[offset..offset + 2]);
                                unsafe {
                                    *u16_ptr.add(i * field.count + k) = val;
                                }
                            }
                        }
                        ValueType::I16 => {
                            let i16_ptr = ptr as *mut i16;
                            for k in 0..field.count {
                                let offset = k * 2;
                                let val = LittleEndian::read_i16(&src_slice[offset..offset + 2]);
                                unsafe {
                                    *i16_ptr.add(i * field.count + k) = val;
                                }
                            }
                        }
                        ValueType::U32 => {
                            let u32_ptr = ptr as *mut u32;
                            for k in 0..field.count {
                                let offset = k * 4;
                                let val = LittleEndian::read_u32(&src_slice[offset..offset + 4]);
                                unsafe {
                                    *u32_ptr.add(i * field.count + k) = val;
                                }
                            }
                        }
                        ValueType::I32 => {
                            let i32_ptr = ptr as *mut i32;
                            for k in 0..field.count {
                                let offset = k * 4;
                                let val = LittleEndian::read_i32(&src_slice[offset..offset + 4]);
                                unsafe {
                                    *i32_ptr.add(i * field.count + k) = val;
                                }
                            }
                        }
                        ValueType::F32 => {
                            let f32_ptr = ptr as *mut f32;
                            for k in 0..field.count {
                                let offset = k * 4;
                                let val = LittleEndian::read_f32(&src_slice[offset..offset + 4]);
                                unsafe {
                                    *f32_ptr.add(i * field.count + k) = val;
                                }
                            }
                        }
                        ValueType::F64 => {
                            let f64_ptr = ptr as *mut f64;
                            for k in 0..field.count {
                                let offset = k * 8;
                                let val = LittleEndian::read_f64(&src_slice[offset..offset + 8]);
                                unsafe {
                                    *f64_ptr.add(i * field.count + k) = val;
                                }
                            }
                        }
                    }
                }
            });

        Ok(())
    }
}

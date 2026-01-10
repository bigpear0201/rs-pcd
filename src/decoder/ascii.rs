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
use std::io::BufRead;

pub struct AsciiReader<'a, R: BufRead> {
    reader: &'a mut R,
    layout: &'a PcdLayout,
    points_to_read: usize,
}

impl<'a, R: BufRead> AsciiReader<'a, R> {
    pub fn new(reader: &'a mut R, layout: &'a PcdLayout, points_to_read: usize) -> Self {
        Self {
            reader,
            layout,
            points_to_read,
        }
    }

    pub fn decode(&mut self, output: &mut PointBlock) -> Result<()> {
        output.resize(self.points_to_read);

        let required_cols: Vec<String> =
            self.layout.fields.iter().map(|f| f.name.clone()).collect();

        // Ensure all columns exist
        for name in &required_cols {
            if output.get_column(name).is_none() {
                return Err(PcdError::LayoutMismatch {
                    expected: 0,
                    got: 0,
                }); // Todo: better error
            }
        }

        let mut columns = output
            .get_columns_mut(&required_cols)
            .ok_or_else(|| PcdError::Other("Failed to mutate columns".to_string()))?;

        let mut line_buffer = String::new();

        for i in 0..self.points_to_read {
            line_buffer.clear();
            let bytes = self.reader.read_line(&mut line_buffer)?;
            if bytes == 0 {
                return Err(PcdError::Io(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF in ASCII data",
                )));
            }

            let tokens: Vec<&str> = line_buffer.split_whitespace().collect();
            let mut token_idx = 0;

            for (field_idx, field) in self.layout.fields.iter().enumerate() {
                let col = &mut columns[field_idx];
                let count = field.count;

                for k in 0..count {
                    if token_idx >= tokens.len() {
                        return Err(PcdError::InvalidDataFormat(format!(
                            "Not enough tokens for point {}, field {}",
                            i, field.name
                        )));
                    }
                    let token = tokens[token_idx];
                    token_idx += 1;

                    let idx = i * count + k;

                    match field.type_ {
                        ValueType::U8 => {
                            let val = token.parse::<u8>().map_err(|_| {
                                PcdError::InvalidDataFormat(format!("Invalid u8: {}", token))
                            })?;
                            col.as_u8_mut().unwrap()[idx] = val;
                        }
                        ValueType::I8 => {
                            let val = token.parse::<i8>().map_err(|_| {
                                PcdError::InvalidDataFormat(format!("Invalid i8: {}", token))
                            })?;
                            col.as_i8_mut().unwrap()[idx] = val;
                        }
                        ValueType::U16 => {
                            let val = token.parse::<u16>().map_err(|_| {
                                PcdError::InvalidDataFormat(format!("Invalid u16: {}", token))
                            })?;
                            col.as_u16_mut().unwrap()[idx] = val;
                        }
                        ValueType::I16 => {
                            let val = token.parse::<i16>().map_err(|_| {
                                PcdError::InvalidDataFormat(format!("Invalid i16: {}", token))
                            })?;
                            col.as_i16_mut().unwrap()[idx] = val;
                        }
                        ValueType::U32 => {
                            let val = token.parse::<u32>().map_err(|_| {
                                PcdError::InvalidDataFormat(format!("Invalid u32: {}", token))
                            })?;
                            col.as_u32_mut().unwrap()[idx] = val;
                        }
                        ValueType::I32 => {
                            let val = token.parse::<i32>().map_err(|_| {
                                PcdError::InvalidDataFormat(format!("Invalid i32: {}", token))
                            })?;
                            col.as_i32_mut().unwrap()[idx] = val;
                        }
                        ValueType::F32 => {
                            let val = token.parse::<f32>().map_err(|_| {
                                PcdError::InvalidDataFormat(format!("Invalid f32: {}", token))
                            })?;
                            col.as_f32_mut().unwrap()[idx] = val;
                        }
                        ValueType::F64 => {
                            let val = token.parse::<f64>().map_err(|_| {
                                PcdError::InvalidDataFormat(format!("Invalid f64: {}", token))
                            })?;
                            col.as_f64_mut().unwrap()[idx] = val;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

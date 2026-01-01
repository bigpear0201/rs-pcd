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

use crate::error::Result;
use crate::header::DataFormat;
use crate::header::PcdHeader;
// use crate::header::ValueType;
use crate::error::PcdError;
use crate::storage::PointBlock;
use byteorder::{LittleEndian, WriteBytesExt};
use lzf;
use std::io::Write;

pub struct PcdWriter<W: Write> {
    writer: W,
}

impl<W: Write> PcdWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_pcd(&mut self, header: &PcdHeader, data: &PointBlock) -> Result<()> {
        self.write_header(header)?;
        match header.data {
            DataFormat::Binary => self.write_binary(header, data)?,
            DataFormat::Ascii => self.write_ascii(header, data)?,
            DataFormat::BinaryCompressed => self.write_compressed_binary(header, data)?,
        }
        Ok(())
    }

    fn write_header(&mut self, header: &PcdHeader) -> Result<()> {
        writeln!(self.writer, "VERSION {}", header.version)?;
        writeln!(self.writer, "FIELDS {}", header.fields.join(" "))?;

        let sizes_str: Vec<String> = header.sizes.iter().map(|s| s.to_string()).collect();
        writeln!(self.writer, "SIZE {}", sizes_str.join(" "))?;

        let types_str: Vec<String> = header.types.iter().map(|t| t.to_string()).collect();
        writeln!(self.writer, "TYPE {}", types_str.join(" "))?;

        let counts_str: Vec<String> = header.counts.iter().map(|c| c.to_string()).collect();
        writeln!(self.writer, "COUNT {}", counts_str.join(" "))?;

        writeln!(self.writer, "WIDTH {}", header.width)?;
        writeln!(self.writer, "HEIGHT {}", header.height)?;

        // VIEWPOINT
        writeln!(
            self.writer,
            "VIEWPOINT {} {} {} {} {} {} {}",
            header.viewpoint[0],
            header.viewpoint[1],
            header.viewpoint[2],
            header.viewpoint[3],
            header.viewpoint[4],
            header.viewpoint[5],
            header.viewpoint[6]
        )?;

        writeln!(self.writer, "POINTS {}", header.points)?;

        match header.data {
            DataFormat::Ascii => writeln!(self.writer, "DATA ascii")?,
            DataFormat::Binary => writeln!(self.writer, "DATA binary")?,
            DataFormat::BinaryCompressed => writeln!(self.writer, "DATA binary_compressed")?,
        }

        Ok(())
    }

    fn write_binary(&mut self, header: &PcdHeader, data: &PointBlock) -> Result<()> {
        // Optimization: Collect column references once
        let mut columns = Vec::with_capacity(header.fields.len());
        for name in &header.fields {
            columns.push(
                data.get_column(name).ok_or_else(|| {
                    PcdError::InvalidDataFormat(format!("Missing column {}", name))
                })?,
            );
        }

        // Loop points, then fields (AoS)
        for i in 0..header.points {
            for (field_idx, _name) in header.fields.iter().enumerate() {
                let col = columns[field_idx];
                let count = header.counts[field_idx];
                let start = i * count;

                match header.types[field_idx] {
                    'F' => {
                        // Check sizes: 4 bytes -> F32, 8 bytes -> F64
                        match header.sizes[field_idx] {
                            4 => {
                                let vec = col.as_f32().ok_or_else(|| PcdError::LayoutMismatch {
                                    expected: 0,
                                    got: 0,
                                })?; // Todo better error
                                for k in 0..count {
                                    self.writer.write_f32::<LittleEndian>(vec[start + k])?;
                                }
                            }
                            8 => {
                                let vec = col.as_f64().ok_or_else(|| PcdError::LayoutMismatch {
                                    expected: 0,
                                    got: 0,
                                })?;
                                for k in 0..count {
                                    self.writer.write_f64::<LittleEndian>(vec[start + k])?;
                                }
                            }
                            _ => {
                                return Err(PcdError::UnsupportedType(format!(
                                    "F{}",
                                    header.sizes[field_idx]
                                )));
                            }
                        }
                    }
                    'U' => match header.sizes[field_idx] {
                        1 => {
                            let vec = col.as_u8().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                self.writer.write_u8(vec[start + k])?;
                            }
                        }
                        2 => {
                            let vec = col.as_u16().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                self.writer.write_u16::<LittleEndian>(vec[start + k])?;
                            }
                        }
                        4 => {
                            let vec = col.as_u32().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                self.writer.write_u32::<LittleEndian>(vec[start + k])?;
                            }
                        }
                        _ => {
                            return Err(PcdError::UnsupportedType(format!(
                                "U{}",
                                header.sizes[field_idx]
                            )));
                        }
                    },
                    'I' => match header.sizes[field_idx] {
                        1 => {
                            let vec = col.as_i8().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                self.writer.write_i8(vec[start + k])?;
                            }
                        }
                        2 => {
                            let vec = col.as_i16().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                self.writer.write_i16::<LittleEndian>(vec[start + k])?;
                            }
                        }
                        4 => {
                            let vec = col.as_i32().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                self.writer.write_i32::<LittleEndian>(vec[start + k])?;
                            }
                        }
                        _ => {
                            return Err(PcdError::UnsupportedType(format!(
                                "I{}",
                                header.sizes[field_idx]
                            )));
                        }
                    },
                    _ => {
                        return Err(PcdError::UnsupportedType(
                            header.types[field_idx].to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn write_ascii(&mut self, header: &PcdHeader, data: &PointBlock) -> Result<()> {
        // Optimization: Collect column references once
        let mut columns = Vec::with_capacity(header.fields.len());
        for name in &header.fields {
            columns.push(
                data.get_column(name).ok_or_else(|| {
                    PcdError::InvalidDataFormat(format!("Missing column {}", name))
                })?,
            );
        }

        for i in 0..header.points {
            let mut line_tokens = Vec::with_capacity(header.fields.len());
            for (field_idx, _name) in header.fields.iter().enumerate() {
                let col = columns[field_idx];
                let count = header.counts[field_idx];
                let start = i * count;

                match header.types[field_idx] {
                    'F' => match header.sizes[field_idx] {
                        4 => {
                            let vec = col.as_f32().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                line_tokens.push(format!("{:.6}", vec[start + k]));
                            }
                        }
                        8 => {
                            let vec = col.as_f64().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                line_tokens.push(format!("{:.6}", vec[start + k]));
                            }
                        }
                        _ => {}
                    },
                    'U' => match header.sizes[field_idx] {
                        1 => {
                            let vec = col.as_u8().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                line_tokens.push(format!("{}", vec[start + k]));
                            }
                        }
                        2 => {
                            let vec = col.as_u16().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                line_tokens.push(format!("{}", vec[start + k]));
                            }
                        }
                        4 => {
                            let vec = col.as_u32().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                line_tokens.push(format!("{}", vec[start + k]));
                            }
                        }
                        _ => {}
                    },
                    'I' => match header.sizes[field_idx] {
                        1 => {
                            let vec = col.as_i8().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                line_tokens.push(format!("{}", vec[start + k]));
                            }
                        }
                        2 => {
                            let vec = col.as_i16().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                line_tokens.push(format!("{}", vec[start + k]));
                            }
                        }
                        4 => {
                            let vec = col.as_i32().ok_or(PcdError::LayoutMismatch {
                                expected: 0,
                                got: 0,
                            })?;
                            for k in 0..count {
                                line_tokens.push(format!("{}", vec[start + k]));
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            writeln!(self.writer, "{}", line_tokens.join(" "))?;
        }
        Ok(())
    }
    fn write_compressed_binary(&mut self, header: &PcdHeader, data: &PointBlock) -> Result<()> {
        let mut uncompressed_data = Vec::new();

        // Binary Compressed is SoA in the buffer
        for (field_idx, name) in header.fields.iter().enumerate() {
            let col = data
                .get_column(name)
                .ok_or_else(|| PcdError::InvalidDataFormat(format!("Missing column {}", name)))?;
            let _count = header.counts[field_idx];

            match header.types[field_idx] {
                'F' => {
                    if header.sizes[field_idx] == 4 {
                        let vec = col.as_f32().unwrap();
                        for val in vec {
                            uncompressed_data.write_f32::<LittleEndian>(*val)?;
                        }
                    } else {
                        let vec = col.as_f64().unwrap();
                        for val in vec {
                            uncompressed_data.write_f64::<LittleEndian>(*val)?;
                        }
                    }
                }
                'U' => match header.sizes[field_idx] {
                    1 => uncompressed_data.write_all(col.as_u8().unwrap())?,
                    2 => {
                        let vec = col.as_u16().unwrap();
                        for val in vec {
                            uncompressed_data.write_u16::<LittleEndian>(*val)?;
                        }
                    }
                    4 => {
                        let vec = col.as_u32().unwrap();
                        for val in vec {
                            uncompressed_data.write_u32::<LittleEndian>(*val)?;
                        }
                    }
                    _ => {}
                },
                'I' => match header.sizes[field_idx] {
                    1 => {
                        let vec = col.as_i8().unwrap();
                        for val in vec {
                            uncompressed_data.write_i8(*val)?;
                        }
                    }
                    2 => {
                        let vec = col.as_i16().unwrap();
                        for val in vec {
                            uncompressed_data.write_i16::<LittleEndian>(*val)?;
                        }
                    }
                    4 => {
                        let vec = col.as_i32().unwrap();
                        for val in vec {
                            uncompressed_data.write_i32::<LittleEndian>(*val)?;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let uncompressed_size = uncompressed_data.len();
        let compressed_result = lzf::compress(&uncompressed_data);

        let (final_compressed_size, final_data) = match compressed_result {
            Ok(data) => (data.len(), data),
            Err(lzf::LzfError::NoCompressionPossible) => (uncompressed_size, uncompressed_data),
            Err(e) => return Err(PcdError::Other(format!("Compression failed: {:?}", e))),
        };

        self.writer
            .write_u32::<LittleEndian>(final_compressed_size as u32)?;
        self.writer
            .write_u32::<LittleEndian>(uncompressed_size as u32)?;
        self.writer.write_all(&final_data)?;

        Ok(())
    }
}

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

use crate::decoder::endian;
use crate::error::Result;
use crate::header::DataFormat;
use crate::header::PcdHeader;
use crate::header::ValueType;
use crate::error::PcdError;
use crate::storage::{Column, PointBlock};
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
        let value_types = header.value_types()?;

        // Collect column references once
        let columns: Vec<&Column> = header
            .fields
            .iter()
            .map(|name| {
                data.get_column(name).ok_or_else(|| {
                    PcdError::InvalidDataFormat(format!(
                        "Missing column '{}' in PointBlock",
                        name
                    ))
                })
            })
            .collect::<Result<_>>()?;

        // Pre-allocate a buffer for one point stride
        let point_step = header.point_step();
        let mut point_buf = vec![0u8; point_step];

        for i in 0..header.points {
            let mut buf_offset = 0;

            for (field_idx, col) in columns.iter().enumerate() {
                let vtype = value_types[field_idx];
                let count = header.counts[field_idx];
                let start = i * count;
                let field_bytes = vtype.size() * count;

                write_field_to_buffer(
                    &mut point_buf[buf_offset..buf_offset + field_bytes],
                    col,
                    vtype,
                    start,
                    count,
                )?;

                buf_offset += field_bytes;
            }

            self.writer.write_all(&point_buf)?;
        }
        Ok(())
    }

    fn write_ascii(&mut self, header: &PcdHeader, data: &PointBlock) -> Result<()> {
        let value_types = header.value_types()?;

        let columns: Vec<&Column> = header
            .fields
            .iter()
            .map(|name| {
                data.get_column(name).ok_or_else(|| {
                    PcdError::InvalidDataFormat(format!(
                        "Missing column '{}' in PointBlock",
                        name
                    ))
                })
            })
            .collect::<Result<_>>()?;

        // Pre-allocate string buffer to reduce allocations
        let mut line = String::with_capacity(header.fields.len() * 12);

        for i in 0..header.points {
            line.clear();

            for (field_idx, col) in columns.iter().enumerate() {
                let vtype = value_types[field_idx];
                let count = header.counts[field_idx];
                let start = i * count;

                for k in 0..count {
                    if !line.is_empty() {
                        line.push(' ');
                    }
                    format_value(&mut line, col, vtype, start + k)?;
                }
            }

            writeln!(self.writer, "{}", line)?;
        }
        Ok(())
    }

    fn write_compressed_binary(
        &mut self,
        header: &PcdHeader,
        data: &PointBlock,
    ) -> Result<()> {
        let value_types = header.value_types()?;

        // Pre-allocate uncompressed buffer: exact size = sum of all field data
        let total_bytes: usize = header
            .fields
            .iter()
            .enumerate()
            .map(|(i, _)| value_types[i].size() * header.counts[i] * header.points)
            .sum();
        let mut uncompressed_data = Vec::with_capacity(total_bytes);

        // Binary Compressed is SoA: [all values of field1][all values of field2]...
        for (field_idx, name) in header.fields.iter().enumerate() {
            let col = data.get_column(name).ok_or_else(|| {
                PcdError::InvalidDataFormat(format!("Missing column '{}' in PointBlock", name))
            })?;

            let vtype = value_types[field_idx];

            // Use LE-optimized encode for bulk column data
            encode_column_to_buffer(&mut uncompressed_data, col, vtype)?;
        }

        let uncompressed_size = uncompressed_data.len();
        let compressed_result = lzf::compress(&uncompressed_data);

        let (final_compressed_size, final_data) = match compressed_result {
            Ok(data) => (data.len(), data),
            Err(lzf::LzfError::NoCompressionPossible) => (uncompressed_size, uncompressed_data),
            Err(e) => return Err(PcdError::Other(format!("Compression failed: {:?}", e))),
        };

        use byteorder::{LittleEndian, WriteBytesExt};
        self.writer
            .write_u32::<LittleEndian>(final_compressed_size as u32)?;
        self.writer
            .write_u32::<LittleEndian>(uncompressed_size as u32)?;
        self.writer.write_all(&final_data)?;

        Ok(())
    }
}

/// Write a field's value(s) into a byte buffer — used by write_binary.
/// On LE platforms, multi-byte types use zero-copy memcpy.
#[inline]
fn write_field_to_buffer(
    buf: &mut [u8],
    col: &Column,
    vtype: ValueType,
    start: usize,
    count: usize,
) -> Result<()> {
    match vtype {
        ValueType::U8 => {
            let src = col.as_u8().ok_or_else(|| type_mismatch_error("U8", col))?;
            buf[..count].copy_from_slice(&src[start..start + count]);
        }
        ValueType::I8 => {
            let src = col.as_i8().ok_or_else(|| type_mismatch_error("I8", col))?;
            for k in 0..count {
                buf[k] = src[start + k] as u8;
            }
        }
        ValueType::U16 => {
            let src = col.as_u16().ok_or_else(|| type_mismatch_error("U16", col))?;
            #[cfg(target_endian = "little")]
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src[start..].as_ptr() as *const u8,
                    buf.as_mut_ptr(),
                    count * 2,
                );
            }
            #[cfg(not(target_endian = "little"))]
            {
                use byteorder::{LittleEndian, ByteOrder};
                for k in 0..count {
                    LittleEndian::write_u16(&mut buf[k * 2..], src[start + k]);
                }
            }
        }
        ValueType::I16 => {
            let src = col.as_i16().ok_or_else(|| type_mismatch_error("I16", col))?;
            #[cfg(target_endian = "little")]
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src[start..].as_ptr() as *const u8,
                    buf.as_mut_ptr(),
                    count * 2,
                );
            }
            #[cfg(not(target_endian = "little"))]
            {
                use byteorder::{LittleEndian, ByteOrder};
                for k in 0..count {
                    LittleEndian::write_i16(&mut buf[k * 2..], src[start + k]);
                }
            }
        }
        ValueType::U32 => {
            let src = col.as_u32().ok_or_else(|| type_mismatch_error("U32", col))?;
            #[cfg(target_endian = "little")]
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src[start..].as_ptr() as *const u8,
                    buf.as_mut_ptr(),
                    count * 4,
                );
            }
            #[cfg(not(target_endian = "little"))]
            {
                use byteorder::{LittleEndian, ByteOrder};
                for k in 0..count {
                    LittleEndian::write_u32(&mut buf[k * 4..], src[start + k]);
                }
            }
        }
        ValueType::I32 => {
            let src = col.as_i32().ok_or_else(|| type_mismatch_error("I32", col))?;
            #[cfg(target_endian = "little")]
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src[start..].as_ptr() as *const u8,
                    buf.as_mut_ptr(),
                    count * 4,
                );
            }
            #[cfg(not(target_endian = "little"))]
            {
                use byteorder::{LittleEndian, ByteOrder};
                for k in 0..count {
                    LittleEndian::write_i32(&mut buf[k * 4..], src[start + k]);
                }
            }
        }
        ValueType::F32 => {
            let src = col.as_f32().ok_or_else(|| type_mismatch_error("F32", col))?;
            #[cfg(target_endian = "little")]
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src[start..].as_ptr() as *const u8,
                    buf.as_mut_ptr(),
                    count * 4,
                );
            }
            #[cfg(not(target_endian = "little"))]
            {
                use byteorder::{LittleEndian, ByteOrder};
                for k in 0..count {
                    LittleEndian::write_f32(&mut buf[k * 4..], src[start + k]);
                }
            }
        }
        ValueType::F64 => {
            let src = col.as_f64().ok_or_else(|| type_mismatch_error("F64", col))?;
            #[cfg(target_endian = "little")]
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src[start..].as_ptr() as *const u8,
                    buf.as_mut_ptr(),
                    count * 8,
                );
            }
            #[cfg(not(target_endian = "little"))]
            {
                use byteorder::{LittleEndian, ByteOrder};
                for k in 0..count {
                    LittleEndian::write_f64(&mut buf[k * 8..], src[start + k]);
                }
            }
        }
    }
    Ok(())
}

/// Encode an entire column into a byte buffer (SoA layout for compressed format).
/// Uses LE-optimized bulk copy on Little Endian platforms.
#[inline]
fn encode_column_to_buffer(
    dest: &mut Vec<u8>,
    col: &Column,
    vtype: ValueType,
) -> Result<()> {
    match vtype {
        ValueType::U8 => {
            dest.extend_from_slice(col.as_u8().ok_or_else(|| type_mismatch_error("U8", col))?);
        }
        ValueType::I8 => {
            let src = col.as_i8().ok_or_else(|| type_mismatch_error("I8", col))?;
            dest.extend(src.iter().map(|&v| v as u8));
        }
        ValueType::U16 => {
            let src = col.as_u16().ok_or_else(|| type_mismatch_error("U16", col))?;
            endian::encode_u16_slice(src, dest);
        }
        ValueType::I16 => {
            let src = col.as_i16().ok_or_else(|| type_mismatch_error("I16", col))?;
            endian::encode_i16_slice(src, dest);
        }
        ValueType::U32 => {
            let src = col.as_u32().ok_or_else(|| type_mismatch_error("U32", col))?;
            endian::encode_u32_slice(src, dest);
        }
        ValueType::I32 => {
            let src = col.as_i32().ok_or_else(|| type_mismatch_error("I32", col))?;
            endian::encode_i32_slice(src, dest);
        }
        ValueType::F32 => {
            let src = col.as_f32().ok_or_else(|| type_mismatch_error("F32", col))?;
            endian::encode_f32_slice(src, dest);
        }
        ValueType::F64 => {
            let src = col.as_f64().ok_or_else(|| type_mismatch_error("F64", col))?;
            endian::encode_f64_slice(src, dest);
        }
    }
    Ok(())
}

/// Format a single value as ASCII string.
#[inline]
fn format_value(
    line: &mut String,
    col: &Column,
    vtype: ValueType,
    idx: usize,
) -> Result<()> {
    use std::fmt::Write;
    match vtype {
        ValueType::U8 => {
            let v = col.as_u8().ok_or_else(|| type_mismatch_error("U8", col))?;
            write!(line, "{}", v[idx]).unwrap();
        }
        ValueType::I8 => {
            let v = col.as_i8().ok_or_else(|| type_mismatch_error("I8", col))?;
            write!(line, "{}", v[idx]).unwrap();
        }
        ValueType::U16 => {
            let v = col.as_u16().ok_or_else(|| type_mismatch_error("U16", col))?;
            write!(line, "{}", v[idx]).unwrap();
        }
        ValueType::I16 => {
            let v = col.as_i16().ok_or_else(|| type_mismatch_error("I16", col))?;
            write!(line, "{}", v[idx]).unwrap();
        }
        ValueType::U32 => {
            let v = col.as_u32().ok_or_else(|| type_mismatch_error("U32", col))?;
            write!(line, "{}", v[idx]).unwrap();
        }
        ValueType::I32 => {
            let v = col.as_i32().ok_or_else(|| type_mismatch_error("I32", col))?;
            write!(line, "{}", v[idx]).unwrap();
        }
        ValueType::F32 => {
            let v = col.as_f32().ok_or_else(|| type_mismatch_error("F32", col))?;
            write!(line, "{:.6}", v[idx]).unwrap();
        }
        ValueType::F64 => {
            let v = col.as_f64().ok_or_else(|| type_mismatch_error("F64", col))?;
            write!(line, "{:.6}", v[idx]).unwrap();
        }
    }
    Ok(())
}

/// Produce a descriptive error when column type doesn't match header expectation.
#[inline]
fn type_mismatch_error(expected: &str, col: &Column) -> PcdError {
    PcdError::InvalidDataFormat(format!(
        "Type mismatch: header expects {}, but column is {:?}",
        expected,
        col.value_type()
    ))
}

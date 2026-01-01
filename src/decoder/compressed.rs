use crate::error::{PcdError, Result};
use crate::header::ValueType;
use crate::layout::PcdLayout;
use crate::storage::PointBlock;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use lzf;
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

        // Initialize output
        for field in &self.layout.fields {
            if !output.columns.contains_key(&field.name) {
                // Or we could auto-create. For now, strict check.
                // Actually better to auto-create? PointBlock usage assumes schemas match.
                // Let's ensure capacity.
            }
        }
        output.resize(self.points_to_read);

        // Process fields (SoA in buffer: [Field1 All Points][Field2 All Points]...)
        let mut offset = 0;

        for field in &self.layout.fields {
            let col = output
                .get_column_mut(&field.name)
                .ok_or(PcdError::InvalidDataFormat(format!(
                    "Missing column {}",
                    field.name
                )))?;

            let bytes_per_element = field.element_size; // e.g. 4 for f32
            let elements_per_point = field.count; // e.g. 1
            let bytes_per_field_block =
                bytes_per_element * elements_per_point * self.points_to_read;

            let end = offset + bytes_per_field_block;
            let data_slice = &decompressed[offset..end];
            offset = end;

            match field.type_ {
                ValueType::U8 => {
                    let vec = col.as_u8_mut().unwrap();
                    vec.copy_from_slice(data_slice);
                }
                ValueType::F32 => {
                    let vec = col.as_f32_mut().unwrap();
                    // Efficient copy using unsafe cast if alignment permits, or safely
                    // Since standard lzf returns Vec<u8>, it might not be aligned to 4.
                    // We iterate.
                    // Optimizations: chunks_exact(4).

                    // We must fill target vec. Target vec is flat for all points.
                    // field.count > 1 means interleaved? No, SoA usually means:
                    // If count=1: x1, x2, x3...
                    // If count=3 (Normal): nx1, ny1, nz1, nx2, ny2... ?
                    // OR: nx1..nxN, ny1..nyN ?
                    // PCL spec: "The fields are stored sequentially... field_1_point_1, field_1_point_2... field_2_point_1..."
                    // Wait. "binary_compressed format... The data is stored in a column-major order."
                    // Does it mean field 1 for all points, then field 2?
                    // Yes. "stored sequentially by field".
                    // But if a field has count > 1 (e.g. viewpoint), how is it stored?
                    // "dimensions corresponding to the same field are stored contiguously".
                    // So if field is "normal" (count=3), is it nx1, ny1, nz1, nx2...?
                    // OR nx1..nxN, ny1..nyN?
                    // Usually PCL treats distinct Names as fields. "Normal" is usually split into "normal_x, normal_y, normal_z" in fields list.
                    // If one field has count > 1 (e.g. FPFH signature 33 floats), it is stored as struct?
                    // PCL generic: it is simply array of structs compressed? No.
                    // It says "reorganized to column array".
                    // I will assume for count > 1, it's (val_1_1, val_1_2... val_1_count), (val_2_1...)...
                    // i.e. The unit being transposed is the whole field value (array).
                    // So yes, loop over points, copy `count` elements.

                    let mut i = 0;
                    for chunk in data_slice.chunks_exact(4) {
                        vec[i] = LittleEndian::read_f32(chunk);
                        i += 1;
                    }
                }
                ValueType::F64 => {
                    let vec = col.as_f64_mut().unwrap();
                    let mut i = 0;
                    for chunk in data_slice.chunks_exact(8) {
                        vec[i] = LittleEndian::read_f64(chunk);
                        i += 1;
                    }
                }
                ValueType::U16 => {
                    let vec = col.as_u16_mut().unwrap();
                    let mut i = 0;
                    for chunk in data_slice.chunks_exact(2) {
                        vec[i] = LittleEndian::read_u16(chunk);
                        i += 1;
                    }
                }
                ValueType::U32 => {
                    let vec = col.as_u32_mut().unwrap();
                    let mut i = 0;
                    for chunk in data_slice.chunks_exact(4) {
                        vec[i] = LittleEndian::read_u32(chunk);
                        i += 1;
                    }
                }
                ValueType::I8 => {
                    let vec = col.as_i8_mut().unwrap();
                    vec.copy_from_slice(unsafe { std::mem::transmute(data_slice) });
                }
                ValueType::I16 => {
                    let vec = col.as_i16_mut().unwrap();
                    let mut i = 0;
                    for chunk in data_slice.chunks_exact(2) {
                        vec[i] = LittleEndian::read_i16(chunk);
                        i += 1;
                    }
                }
                ValueType::I32 => {
                    let vec = col.as_i32_mut().unwrap();
                    let mut i = 0;
                    for chunk in data_slice.chunks_exact(4) {
                        vec[i] = LittleEndian::read_i32(chunk);
                        i += 1;
                    }
                }
            }
        }

        Ok(())
    }
}

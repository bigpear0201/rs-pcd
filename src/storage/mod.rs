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

use crate::header::ValueType;
use std::collections::{HashMap, HashSet};

pub mod view;
pub use view::{ColumnView, PointView};

#[derive(Debug, Clone)]
pub enum Column {
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
    I8(Vec<i8>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    F32(Vec<f32>),
    F64(Vec<f64>),
}

impl Column {
    pub fn new(value_type: ValueType, capacity: usize) -> Self {
        match value_type {
            ValueType::U8 => Column::U8(vec![0; capacity]),
            ValueType::U16 => Column::U16(vec![0; capacity]),
            ValueType::U32 => Column::U32(vec![0; capacity]),
            ValueType::I8 => Column::I8(vec![0; capacity]),
            ValueType::I16 => Column::I16(vec![0; capacity]),
            ValueType::I32 => Column::I32(vec![0; capacity]),
            ValueType::F32 => Column::F32(vec![0.0; capacity]),
            ValueType::F64 => Column::F64(vec![0.0; capacity]),
        }
    }

    pub fn resize(&mut self, new_len: usize) {
        match self {
            Column::U8(v) => v.resize(new_len, 0),
            Column::U16(v) => v.resize(new_len, 0),
            Column::U32(v) => v.resize(new_len, 0),
            Column::I8(v) => v.resize(new_len, 0),
            Column::I16(v) => v.resize(new_len, 0),
            Column::I32(v) => v.resize(new_len, 0),
            Column::F32(v) => v.resize(new_len, 0.0),
            Column::F64(v) => v.resize(new_len, 0.0),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Column::U8(v) => v.len(),
            Column::U16(v) => v.len(),
            Column::U32(v) => v.len(),
            Column::I8(v) => v.len(),
            Column::I16(v) => v.len(),
            Column::I32(v) => v.len(),
            Column::F32(v) => v.len(),
            Column::F64(v) => v.len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_f32_slice(&self) -> Option<&[f32]> {
        if let Column::F32(v) = self {
            Some(v)
        } else {
            None
        }
    }

    // Mutable access for decoders
    // Safe internal mutable access
    pub fn as_u8_mut(&mut self) -> Option<&mut Vec<u8>> {
        if let Column::U8(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_u16_mut(&mut self) -> Option<&mut Vec<u16>> {
        if let Column::U16(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_u32_mut(&mut self) -> Option<&mut Vec<u32>> {
        if let Column::U32(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_i8_mut(&mut self) -> Option<&mut Vec<i8>> {
        if let Column::I8(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_i16_mut(&mut self) -> Option<&mut Vec<i16>> {
        if let Column::I16(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_i32_mut(&mut self) -> Option<&mut Vec<i32>> {
        if let Column::I32(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_f32_mut(&mut self) -> Option<&mut Vec<f32>> {
        if let Column::F32(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_f64_mut(&mut self) -> Option<&mut Vec<f64>> {
        if let Column::F64(v) = self {
            Some(v)
        } else {
            None
        }
    }

    // Read access variants (useful for Writer)
    pub fn as_u8(&self) -> Option<&[u8]> {
        if let Column::U8(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_u16(&self) -> Option<&[u16]> {
        if let Column::U16(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_u32(&self) -> Option<&[u32]> {
        if let Column::U32(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_i8(&self) -> Option<&[i8]> {
        if let Column::I8(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_i16(&self) -> Option<&[i16]> {
        if let Column::I16(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_i32(&self) -> Option<&[i32]> {
        if let Column::I32(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_f32(&self) -> Option<&[f32]> {
        if let Column::F32(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_f64(&self) -> Option<&[f64]> {
        if let Column::F64(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns a raw pointer to the column storage and its byte length.
    ///
    /// # Safety
    ///
    /// Callers must ensure any writes through the returned pointer uphold Rust's
    /// aliasing rules and only touch disjoint regions when used concurrently.
    pub unsafe fn as_ptr_mut(&mut self) -> (*mut u8, usize) {
        match self {
            Column::U8(v) => (v.as_mut_ptr(), v.len()),
            Column::U16(v) => (v.as_mut_ptr() as *mut u8, v.len() * 2),
            Column::U32(v) => (v.as_mut_ptr() as *mut u8, v.len() * 4),
            Column::I8(v) => (v.as_mut_ptr() as *mut u8, v.len()),
            Column::I16(v) => (v.as_mut_ptr() as *mut u8, v.len() * 2),
            Column::I32(v) => (v.as_mut_ptr() as *mut u8, v.len() * 4),
            Column::F32(v) => (v.as_mut_ptr() as *mut u8, v.len() * 4),
            Column::F64(v) => (v.as_mut_ptr() as *mut u8, v.len() * 8),
        }
    }
}

/// SoA (Structure of Arrays) storage for point cloud data.
///
/// Internally uses Vec<Column> for O(1) index-based access, with a HashMap
/// for name-based lookups. This provides efficient iteration while maintaining
/// backwards-compatible named access.
/// `(x, y, z)` coordinate slices.
pub type Xyz<'a> = (&'a [f32], &'a [f32], &'a [f32]);
/// `(x, y, z, intensity)` slices.
pub type Xyzi<'a> = (&'a [f32], &'a [f32], &'a [f32], &'a [f32]);
/// `(x, y, z, rgb)` slices, where `rgb` is packed as `u32`.
pub type XyzRgb<'a> = (&'a [f32], &'a [f32], &'a [f32], &'a [u32]);

#[derive(Debug, Default)]
pub struct PointBlock {
    /// Column data stored in schema order for O(1) indexed access
    columns: Vec<Column>,
    /// Field names in schema order
    schema: Vec<String>,
    /// Name to index mapping for backwards-compatible get_column(name) API
    name_to_index: HashMap<String, usize>,
    /// Number of points
    pub len: usize,
}

impl PointBlock {
    pub fn new(schema: &[(String, ValueType)], capacity: usize) -> Self {
        let mut columns = Vec::with_capacity(schema.len());
        let mut names = Vec::with_capacity(schema.len());
        let mut name_to_index = HashMap::with_capacity(schema.len());

        for (i, (name, dtype)) in schema.iter().enumerate() {
            columns.push(Column::new(*dtype, capacity));
            names.push(name.clone());
            name_to_index.insert(name.clone(), i);
        }

        PointBlock {
            columns,
            schema: names,
            name_to_index,
            len: capacity,
        }
    }

    pub fn resize(&mut self, new_len: usize) {
        for col in &mut self.columns {
            col.resize(new_len);
        }
        self.len = new_len;
    }

    /// Get a column by name (backwards-compatible API).
    /// For performance-critical code, prefer `get_column_by_index`.
    #[must_use]
    pub fn get_column(&self, name: &str) -> Option<&Column> {
        self.name_to_index.get(name).map(|&idx| &self.columns[idx])
    }

    /// Get a mutable column by name (backwards-compatible API).
    /// For performance-critical code, prefer `get_column_mut_by_index`.
    pub fn get_column_mut(&mut self, name: &str) -> Option<&mut Column> {
        if let Some(&idx) = self.name_to_index.get(name) {
            Some(&mut self.columns[idx])
        } else {
            None
        }
    }

    /// O(1) indexed access to a column.
    #[inline]
    #[must_use]
    pub fn get_column_by_index(&self, index: usize) -> Option<&Column> {
        self.columns.get(index)
    }

    /// O(1) mutable indexed access to a column.
    #[inline]
    pub fn get_column_mut_by_index(&mut self, index: usize) -> Option<&mut Column> {
        self.columns.get_mut(index)
    }

    /// Get the index of a column by name.
    #[inline]
    #[must_use]
    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.name_to_index.get(name).copied()
    }

    /// Get the schema (field names in order).
    #[must_use]
    pub fn schema(&self) -> &[String] {
        &self.schema
    }

    /// Number of columns.
    #[must_use]
    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }

    /// Optimized: Get multiple mutable columns simultaneously.
    /// Returns None if any column is missing or if names contain duplicates.
    /// This avoids O(N*M) lookup inside tight loops.
    pub fn get_columns_mut(&mut self, names: &[String]) -> Option<Vec<&mut Column>> {
        let mut seen = HashSet::with_capacity(names.len());
        for name in names {
            if !seen.insert(name.as_str()) {
                return None;
            }
        }

        // Get indices for all requested names
        let mut indices = Vec::with_capacity(names.len());
        for name in names {
            if let Some(&idx) = self.name_to_index.get(name) {
                indices.push(idx);
            } else {
                return None; // Missing column
            }
        }

        // Use raw pointers to bypass borrow checker for multiple mutable references
        // Safety: We verified all indices are unique above, so all pointers point to disjoint memory.
        let mut results = Vec::with_capacity(names.len());
        let base_ptr = self.columns.as_mut_ptr();
        for idx in indices {
            unsafe {
                results.push(&mut *base_ptr.add(idx));
            }
        }
        Some(results)
    }

    /// Optimized: Get multiple mutable columns by pre-resolved indices.
    /// Returns None if any index is out of bounds or duplicated.
    pub fn get_columns_mut_by_index(&mut self, indices: &[usize]) -> Option<Vec<&mut Column>> {
        let mut seen = HashSet::with_capacity(indices.len());
        for &idx in indices {
            if idx >= self.columns.len() || !seen.insert(idx) {
                return None;
            }
        }

        let mut results = Vec::with_capacity(indices.len());
        let base_ptr = self.columns.as_mut_ptr();
        for &idx in indices {
            unsafe {
                results.push(&mut *base_ptr.add(idx));
            }
        }
        Some(results)
    }

    /// Access underlying columns slice (for iteration).
    #[must_use]
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Access underlying columns mutably.
    pub fn columns_mut(&mut self) -> &mut [Column] {
        &mut self.columns
    }

    // ========================
    // Typed Convenience Accessors
    // ========================

    /// Get XYZ coordinates as f32 slices.
    /// Returns None if any of x, y, z columns are missing or not F32.
    #[must_use]
    pub fn xyz(&self) -> Option<Xyz<'_>> {
        let x = self.get_column("x")?.as_f32()?;
        let y = self.get_column("y")?.as_f32()?;
        let z = self.get_column("z")?.as_f32()?;
        Some((x, y, z))
    }

    /// Get XYZ + intensity as f32 slices.
    /// Returns None if any column is missing or has wrong type.
    #[must_use]
    pub fn xyzi(&self) -> Option<Xyzi<'_>> {
        let x = self.get_column("x")?.as_f32()?;
        let y = self.get_column("y")?.as_f32()?;
        let z = self.get_column("z")?.as_f32()?;
        let i = self.get_column("intensity")?.as_f32()?;
        Some((x, y, z, i))
    }

    /// Get XYZ + RGB (packed as u32) slices.
    /// Returns None if any column is missing or has wrong type.
    #[must_use]
    pub fn xyzrgb(&self) -> Option<XyzRgb<'_>> {
        let x = self.get_column("x")?.as_f32()?;
        let y = self.get_column("y")?.as_f32()?;
        let z = self.get_column("z")?.as_f32()?;
        let rgb = self.get_column("rgb")?.as_u32()?;
        Some((x, y, z, rgb))
    }
}

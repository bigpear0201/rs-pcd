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
use std::collections::HashMap;

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

    // Unsafe methods to get mutable slice for parallel writing.
    // Safety: Caller must ensure exclusive access to the slice regions if writing in parallel.
    pub unsafe fn as_ptr_mut(&mut self) -> (*mut u8, usize) {
        match self {
            Column::U8(v) => (v.as_mut_ptr() as *mut u8, v.len() * 1),
            Column::U16(v) => (v.as_mut_ptr() as *mut u8, v.len() * 2),
            Column::U32(v) => (v.as_mut_ptr() as *mut u8, v.len() * 4),
            Column::I8(v) => (v.as_mut_ptr() as *mut u8, v.len() * 1),
            Column::I16(v) => (v.as_mut_ptr() as *mut u8, v.len() * 2),
            Column::I32(v) => (v.as_mut_ptr() as *mut u8, v.len() * 4),
            Column::F32(v) => (v.as_mut_ptr() as *mut u8, v.len() * 4),
            Column::F64(v) => (v.as_mut_ptr() as *mut u8, v.len() * 8),
        }
    }
}

#[derive(Debug, Default)]
pub struct PointBlock {
    pub columns: HashMap<String, Column>,
    pub len: usize,
}

impl PointBlock {
    pub fn new(schema: &Vec<(String, ValueType)>, capacity: usize) -> Self {
        let mut columns = HashMap::new();
        for (name, dtype) in schema {
            columns.insert(name.clone(), Column::new(*dtype, capacity));
        }
        PointBlock {
            columns,
            len: capacity,
        }
    }

    pub fn resize(&mut self, new_len: usize) {
        for col in self.columns.values_mut() {
            col.resize(new_len);
        }
        self.len = new_len;
    }

    pub fn get_column(&self, name: &str) -> Option<&Column> {
        self.columns.get(name)
    }

    pub fn get_column_mut(&mut self, name: &str) -> Option<&mut Column> {
        self.columns.get_mut(name)
    }

    /// Optimized: Get multiple mutable columns simultaneously.
    /// Returns error if any column is missing or if names contain duplicates.
    /// This avoids O(N*M) lookup inside tight loops.
    pub fn get_columns_mut(&mut self, names: &[String]) -> Option<Vec<&mut Column>> {
        // Simple check for duplicates (O(M^2) but M is small, e.g. < 10)
        for i in 0..names.len() {
            for j in i + 1..names.len() {
                if names[i] == names[j] {
                    return None; // Duplicate requested
                }
            }
        }

        // We can't use `get_many_mut` (unstable) yet, so we iterate and use unsafe or separate scopes.
        // Actually, since this is for a specific set of keys, we can just iterate self.columns if we want,
        // but self.columns is HashMap.
        // Safe approach: Split borrow? No, HashMap doesn't support easy split borrow by key.
        // We will use raw pointers here to bypass borrow checker, BUT we verify uniqueness of keys above.

        let mut ptrs = Vec::with_capacity(names.len());
        for name in names {
            if let Some(col) = self.columns.get_mut(name) {
                ptrs.push(col as *mut Column);
            } else {
                return None; // Missing column
            }
        }

        // Reconstruct mutable references
        // Safety: We verified all keys are unique, so all pointers point to disjoint memory.
        let mut results = Vec::with_capacity(names.len());
        for ptr in ptrs {
            unsafe { results.push(&mut *ptr) };
        }
        Some(results)
    }
}

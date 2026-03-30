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

//! Borrowed view types for zero-copy point cloud access.
//!
//! **Note**: This module is a placeholder for future mmap-backed views.
//! Consider using `PointBlock` with typed accessors (`xyz()`, `xyzirt()`, etc.)
//! for current use cases.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum ColumnView<'a> {
    U8(&'a [u8]),
    U16(&'a [u16]),
    U32(&'a [u32]),
    I8(&'a [i8]),
    I16(&'a [i16]),
    I32(&'a [i32]),
    F32(&'a [f32]),
    F64(&'a [f64]),
}

impl<'a> ColumnView<'a> {
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            ColumnView::U8(v) => v.len(),
            ColumnView::U16(v) => v.len(),
            ColumnView::U32(v) => v.len(),
            ColumnView::I8(v) => v.len(),
            ColumnView::I16(v) => v.len(),
            ColumnView::I32(v) => v.len(),
            ColumnView::F32(v) => v.len(),
            ColumnView::F64(v) => v.len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct PointView<'a> {
    pub columns: HashMap<String, ColumnView<'a>>,
    pub len: usize,
}

impl<'a> Default for PointView<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PointView<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
            len: 0,
        }
    }
}

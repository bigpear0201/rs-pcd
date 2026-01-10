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

//! Builder pattern for constructing PcdHeader.
//! 
//! This provides a more ergonomic API than manually constructing a PcdHeader,
//! automatically deriving sizes, types, and counts from the ValueType.
//! 
//! # Example
//! 
//! ```rust
//! use rs_pcd::header::{PcdHeaderBuilder, ValueType, DataFormat};
//! 
//! let header = PcdHeaderBuilder::new()
//!     .add_field("x", ValueType::F32)
//!     .add_field("y", ValueType::F32)
//!     .add_field("z", ValueType::F32)
//!     .add_field("intensity", ValueType::F32)
//!     .width(1000)
//!     .data_format(DataFormat::Binary)
//!     .build()
//!     .unwrap();
//! ```

use super::{DataFormat, PcdHeader, ValueType};
use crate::error::{PcdError, Result};

/// Builder for constructing PcdHeader with a fluent API.
#[derive(Debug, Clone)]
pub struct PcdHeaderBuilder {
    fields: Vec<(String, ValueType)>,
    width: Option<u32>,
    height: u32,
    data: DataFormat,
    viewpoint: [f64; 7],
    version: String,
}

impl Default for PcdHeaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PcdHeaderBuilder {
    /// Create a new builder with default values.
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            width: None,
            height: 1,
            data: DataFormat::Binary,
            viewpoint: [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
            version: "0.7".to_string(),
        }
    }

    /// Add a field with the given name and type.
    /// Fields are added in order and can only have count=1.
    /// For fields with count > 1, use `add_field_with_count`.
    #[must_use]
    pub fn add_field(mut self, name: &str, value_type: ValueType) -> Self {
        self.fields.push((name.to_string(), value_type));
        self
    }

    /// Set the width (number of points per row).
    /// For unorganized point clouds, this equals the total number of points.
    #[must_use]
    pub fn width(mut self, w: u32) -> Self {
        self.width = Some(w);
        self
    }

    /// Set the height (number of rows).
    /// Default is 1 (unorganized point cloud).
    #[must_use]
    pub fn height(mut self, h: u32) -> Self {
        self.height = h;
        self
    }

    /// Set the data format (Ascii, Binary, or BinaryCompressed).
    /// Default is Binary.
    #[must_use]
    pub fn data_format(mut self, fmt: DataFormat) -> Self {
        self.data = fmt;
        self
    }

    /// Set the viewpoint (tx, ty, tz, qw, qx, qy, qz).
    /// Default is identity: [0, 0, 0, 1, 0, 0, 0].
    #[must_use]
    pub fn viewpoint(mut self, vp: [f64; 7]) -> Self {
        self.viewpoint = vp;
        self
    }

    /// Set the PCD version string.
    /// Default is "0.7".
    #[must_use]
    pub fn version(mut self, v: &str) -> Self {
        self.version = v.to_string();
        self
    }

    /// Build the PcdHeader.
    /// Returns an error if width is not set.
    pub fn build(self) -> Result<PcdHeader> {
        let width = self.width.ok_or_else(|| {
            PcdError::InvalidHeader {
                line: 0,
                msg: "Width must be set".to_string(),
            }
        })?;

        if self.fields.is_empty() {
            return Err(PcdError::InvalidHeader {
                line: 0,
                msg: "At least one field must be added".to_string(),
            });
        }

        let mut field_names = Vec::with_capacity(self.fields.len());
        let mut sizes = Vec::with_capacity(self.fields.len());
        let mut types = Vec::with_capacity(self.fields.len());
        let mut counts = Vec::with_capacity(self.fields.len());

        for (name, vtype) in &self.fields {
            field_names.push(name.clone());
            sizes.push(vtype.size());
            types.push(value_type_to_char(*vtype));
            counts.push(1);
        }

        let points = (width as usize) * (self.height as usize);

        Ok(PcdHeader {
            version: self.version,
            fields: field_names,
            sizes,
            types,
            counts,
            width,
            height: self.height,
            viewpoint: self.viewpoint,
            points,
            data: self.data,
        })
    }
}

/// Convert ValueType to PCD type character.
fn value_type_to_char(vtype: ValueType) -> char {
    match vtype {
        ValueType::I8 | ValueType::I16 | ValueType::I32 => 'I',
        ValueType::U8 | ValueType::U16 | ValueType::U32 => 'U',
        ValueType::F32 | ValueType::F64 => 'F',
    }
}

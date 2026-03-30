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
use std::str::FromStr;

mod builder;
mod parser;
pub use builder::PcdHeaderBuilder;
pub use parser::parse_header;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataFormat {
    #[default]
    Ascii,
    Binary,
    BinaryCompressed,
}

impl FromStr for DataFormat {
    type Err = PcdError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "ascii" => Ok(DataFormat::Ascii),
            "binary" => Ok(DataFormat::Binary),
            "binary_compressed" => Ok(DataFormat::BinaryCompressed),
            _ => Err(PcdError::UnsupportedDataFormat(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    F32,
    F64,
}

impl ValueType {
    #[inline]
    #[must_use]
    pub fn size(&self) -> usize {
        match self {
            ValueType::U8 | ValueType::I8 => 1,
            ValueType::U16 | ValueType::I16 => 2,
            ValueType::U32 | ValueType::I32 | ValueType::F32 => 4,
            ValueType::F64 => 8,
        }
    }

    /// Convert from PCD type char + size to ValueType.
    pub fn from_type_and_size(type_char: char, size: usize) -> Result<Self> {
        match (type_char, size) {
            ('I', 1) => Ok(ValueType::I8),
            ('I', 2) => Ok(ValueType::I16),
            ('I', 4) => Ok(ValueType::I32),
            ('U', 1) => Ok(ValueType::U8),
            ('U', 2) => Ok(ValueType::U16),
            ('U', 4) => Ok(ValueType::U32),
            ('F', 4) => Ok(ValueType::F32),
            ('F', 8) => Ok(ValueType::F64),
            _ => Err(PcdError::UnsupportedType(format!("{}{}", type_char, size))),
        }
    }

    /// Convert to PCD type character ('F', 'U', 'I').
    #[inline]
    #[must_use]
    pub fn type_char(&self) -> char {
        match self {
            ValueType::I8 | ValueType::I16 | ValueType::I32 => 'I',
            ValueType::U8 | ValueType::U16 | ValueType::U32 => 'U',
            ValueType::F32 | ValueType::F64 => 'F',
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PcdHeader {
    pub version: String,
    pub fields: Vec<String>,
    pub sizes: Vec<usize>,
    pub types: Vec<char>,
    pub counts: Vec<usize>,
    pub width: u32,
    pub height: u32,
    pub viewpoint: [f64; 7],
    pub points: usize,
    pub data: DataFormat,
}

impl PcdHeader {
    #[inline]
    #[must_use]
    pub fn is_organized(&self) -> bool {
        self.height > 1
    }

    /// Compute the per-point stride in bytes (sum of size * count for all fields).
    #[must_use]
    pub fn point_step(&self) -> usize {
        self.sizes
            .iter()
            .zip(self.counts.iter())
            .map(|(size, count)| size * count)
            .sum()
    }

    /// Derive ValueType for each field from the raw types/sizes arrays.
    /// Returns Err if any (type, size) combination is unsupported.
    pub fn value_types(&self) -> Result<Vec<ValueType>> {
        self.types
            .iter()
            .zip(self.sizes.iter())
            .map(|(&t, &s)| ValueType::from_type_and_size(t, s))
            .collect()
    }
}

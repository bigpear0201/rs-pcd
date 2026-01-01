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
use crate::header::{PcdHeader, ValueType};

#[derive(Debug, Clone, PartialEq)]
pub struct FieldLayout {
    pub name: String,
    pub offset: usize,
    pub size: usize,         // size in bytes of the field (type_size * count)
    pub element_size: usize, // size of single element
    pub count: usize,
    pub type_: ValueType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PcdLayout {
    pub fields: Vec<FieldLayout>,
    pub total_size: usize,
}

impl PcdLayout {
    pub fn from_header(header: &PcdHeader) -> Result<Self> {
        let mut fields = Vec::with_capacity(header.fields.len());
        let mut offset = 0;

        for (i, name) in header.fields.iter().enumerate() {
            let type_char = header
                .types
                .get(i)
                .ok_or_else(|| PcdError::LayoutMismatch {
                    expected: header.fields.len(),
                    got: i,
                })?;

            let size_in_header = *header
                .sizes
                .get(i)
                .ok_or_else(|| PcdError::LayoutMismatch {
                    expected: header.fields.len(),
                    got: i,
                })?;

            let count = *header.counts.get(i).unwrap_or(&1);

            let value_type = match type_char {
                'I' => match size_in_header {
                    1 => ValueType::I8,
                    2 => ValueType::I16,
                    4 => ValueType::I32,
                    _ => return Err(PcdError::UnsupportedType(format!("I{}", size_in_header))),
                },
                'U' => match size_in_header {
                    1 => ValueType::U8,
                    2 => ValueType::U16,
                    4 => ValueType::U32,
                    _ => return Err(PcdError::UnsupportedType(format!("U{}", size_in_header))),
                },
                'F' => match size_in_header {
                    4 => ValueType::F32,
                    8 => ValueType::F64,
                    _ => return Err(PcdError::UnsupportedType(format!("F{}", size_in_header))),
                },
                _ => return Err(PcdError::UnsupportedType(type_char.to_string())),
            };

            // Check if size * count matches expected logic if strict?
            // PCD Header SIZE is size of *one* element typically (like '4' for float), even if count > 1.
            // But let's verify. Yes, SIZE is per-element bytes.

            let element_size = value_type.size();
            if element_size != size_in_header {
                return Err(PcdError::LayoutMismatch {
                    expected: element_size,
                    got: size_in_header,
                });
            }

            let field_size = element_size * count;

            fields.push(FieldLayout {
                name: name.clone(),
                offset,
                size: field_size,
                element_size,
                count,
                type_: value_type,
            });

            offset += field_size;
        }

        Ok(PcdLayout {
            fields,
            total_size: offset,
        })
    }

    pub fn get_field(&self, name: &str) -> Option<&FieldLayout> {
        self.fields.iter().find(|f| f.name == name)
    }
}

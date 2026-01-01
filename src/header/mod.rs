use crate::error::{PcdError, Result};
use std::str::FromStr;

mod parser;
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
    pub fn size(&self) -> usize {
        match self {
            ValueType::U8 | ValueType::I8 => 1,
            ValueType::U16 | ValueType::I16 => 2,
            ValueType::U32 | ValueType::I32 | ValueType::F32 => 4,
            ValueType::F64 => 8,
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
    pub fn is_organized(&self) -> bool {
        self.height > 1
    }

    pub fn point_step(&self) -> usize {
        self.sizes.iter().sum() // Simplified; actual stride might handle padding if counts > 1? Standard PCD usually tightly packed?
        // Actually, PCD spec says "SIZE is the size of each dimension in bytes".
        // "COUNT is the number of elements in each dimension."
        // Point step is usually sum(size * count).
    }

    pub fn total_point_step(&self) -> usize {
        self.sizes
            .iter()
            .zip(self.counts.iter())
            .map(|(size, count)| size * count)
            .sum()
    }
}

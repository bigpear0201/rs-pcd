// use crate::header::ValueType;

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
}

pub struct PointView<'a> {
    pub columns: HashMap<String, ColumnView<'a>>,
    pub len: usize,
}

impl<'a> PointView<'a> {
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
            len: 0,
        }
    }
}

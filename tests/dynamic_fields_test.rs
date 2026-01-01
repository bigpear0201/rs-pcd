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

use pcd_rs::header::{DataFormat, PcdHeader, ValueType};
use pcd_rs::io::{PcdReader, PcdWriter};
use pcd_rs::storage::PointBlock;
use std::io::Cursor;

#[test]
fn test_dynamic_fields_binary() {
    let fields = vec![
        ("x".to_string(), ValueType::F32),
        ("y".to_string(), ValueType::F32),
        ("z".to_string(), ValueType::F32),
        ("id".to_string(), ValueType::U32),
        ("label".to_string(), ValueType::U8),
        ("timestamp".to_string(), ValueType::F64),
    ];
    let num_points = 10;

    // Create data
    let mut block = PointBlock::new(&fields, num_points);
    {
        let names = vec![
            "x".to_string(),
            "y".to_string(),
            "z".to_string(),
            "id".to_string(),
            "label".to_string(),
            "timestamp".to_string(),
        ];
        let mut cols = block
            .get_columns_mut(&names)
            .expect("Failed to get columns");

        // We have to split the vector to get individual mutable references to columns...
        // Or we can just iterate. But to assign specific logic we need them separate.
        // Since `cols` is `Vec<&mut Column>`, we can use split_at_mut or similar, but 6 items is tedious.
        // Actually, we can just access them by index if we used consistent order.
        // `cols[0]` is 'x', `cols[1]` is 'y', ...

        let (x_col, rest) = cols.split_first_mut().unwrap();
        let (y_col, rest) = rest.split_first_mut().unwrap();
        let (z_col, rest) = rest.split_first_mut().unwrap();
        let (id_col, rest) = rest.split_first_mut().unwrap();
        let (label_col, rest) = rest.split_first_mut().unwrap();
        let (ts_col, _) = rest.split_first_mut().unwrap();

        let x = x_col.as_f32_mut().unwrap();
        let y = y_col.as_f32_mut().unwrap();
        let z = z_col.as_f32_mut().unwrap();
        let id = id_col.as_u32_mut().unwrap();
        let label = label_col.as_u8_mut().unwrap();
        let ts = ts_col.as_f64_mut().unwrap();

        for i in 0..num_points {
            x[i] = i as f32;
            y[i] = (i * 2) as f32;
            z[i] = (i * 3) as f32;
            id[i] = 1000 + i as u32;
            label[i] = (i % 255) as u8;
            ts[i] = i as f64 * 0.1;
        }
    }

    // Create Header
    let header = PcdHeader {
        version: "0.7".to_string(),
        fields: vec![
            "x".into(),
            "y".into(),
            "z".into(),
            "id".into(),
            "label".into(),
            "timestamp".into(),
        ],
        sizes: vec![4, 4, 4, 4, 1, 8],
        types: vec!['F', 'F', 'F', 'U', 'U', 'F'],
        counts: vec![1, 1, 1, 1, 1, 1],
        width: num_points as u32,
        height: 1,
        viewpoint: [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        points: num_points as usize,
        data: DataFormat::Binary,
    };

    // Write to buffer
    let mut buffer = Vec::new();
    {
        let mut writer = PcdWriter::new(&mut buffer);
        writer.write_pcd(&header, &block).expect("Write failed");
    }

    // Read back
    let reader = PcdReader::new(Cursor::new(buffer)).expect("Reader creation failed");
    let read_block = reader.read_all().expect("Read failed");

    // Verify
    assert_eq!(read_block.len, num_points);
    let id_col = read_block.get_column("id").unwrap().as_u32().unwrap();
    let label_col = read_block.get_column("label").unwrap().as_u8().unwrap();
    let ts_col = read_block
        .get_column("timestamp")
        .unwrap()
        .as_f64()
        .unwrap();

    for i in 0..num_points {
        assert_eq!(id_col[i], 1000 + i as u32);
        assert_eq!(label_col[i], (i % 255) as u8);
        assert!((ts_col[i] - i as f64 * 0.1).abs() < 1e-10);
    }
}

#[test]
fn test_dynamic_fields_ascii() {
    let fields = vec![
        ("x".to_string(), ValueType::F32),
        ("intensity".to_string(), ValueType::F32),
        ("id".to_string(), ValueType::I32),
    ];
    let num_points = 5;

    let mut block = PointBlock::new(&fields, num_points);
    {
        let names = vec!["x".to_string(), "intensity".to_string(), "id".to_string()];
        let mut cols = block
            .get_columns_mut(&names)
            .expect("Failed to get columns");

        let (x_col, rest) = cols.split_first_mut().unwrap();
        let (ints_col, rest) = rest.split_first_mut().unwrap();
        let (id_col, _) = rest.split_first_mut().unwrap();

        let x = x_col.as_f32_mut().unwrap();
        let ints = ints_col.as_f32_mut().unwrap();
        let id = id_col.as_i32_mut().unwrap();

        for i in 0..num_points {
            x[i] = i as f32;
            ints[i] = 0.5;
            id[i] = -(i as i32);
        }
    }

    let header = PcdHeader {
        version: "0.7".to_string(),
        fields: vec!["x".into(), "intensity".into(), "id".into()],
        sizes: vec![4, 4, 4],
        types: vec!['F', 'F', 'I'],
        counts: vec![1, 1, 1],
        width: num_points as u32,
        height: 1,
        viewpoint: [0.0; 7],
        points: num_points as usize,
        data: DataFormat::Ascii,
    };

    let mut buffer = Vec::new();
    {
        let mut writer = PcdWriter::new(&mut buffer);
        writer.write_pcd(&header, &block).expect("Write failed");
    }

    // Check ASCII content visually (optional debugging)
    let s = String::from_utf8(buffer.clone()).unwrap();
    println!("ASCII content:\n{}", s);

    let reader = PcdReader::new(Cursor::new(buffer)).expect("Reader creation failed");
    let read_block = reader.read_all().expect("Read failed");

    let id_col = read_block.get_column("id").unwrap().as_i32().unwrap();
    for i in 0..num_points {
        assert_eq!(id_col[i], -(i as i32));
    }
}

#[test]
fn test_dynamic_fields_compressed() {
    let fields = vec![
        ("x".to_string(), ValueType::F32),
        ("y".to_string(), ValueType::F32),
        ("id".to_string(), ValueType::U32),
    ];
    let num_points = 20;

    let mut block = PointBlock::new(&fields, num_points);
    {
        let names = vec!["x".to_string(), "y".to_string(), "id".to_string()];
        let mut cols = block
            .get_columns_mut(&names)
            .expect("Failed to get columns");
        let (x_col, rest) = cols.split_first_mut().unwrap();
        let (y_col, rest) = rest.split_first_mut().unwrap();
        let (id_col, _) = rest.split_first_mut().unwrap();

        let x = x_col.as_f32_mut().unwrap();
        let y = y_col.as_f32_mut().unwrap();
        let id = id_col.as_u32_mut().unwrap();

        for i in 0..num_points {
            x[i] = i as f32 * 0.5;
            y[i] = i as f32 * 2.0;
            id[i] = i as u32 + 500;
        }
    }

    let header = PcdHeader {
        version: "0.7".to_string(),
        fields: vec!["x".into(), "y".into(), "id".into()],
        sizes: vec![4, 4, 4],
        types: vec!['F', 'F', 'U'],
        counts: vec![1, 1, 1],
        width: num_points as u32,
        height: 1,
        viewpoint: [0.0; 7],
        points: num_points as usize,
        data: DataFormat::BinaryCompressed,
    };

    let mut buffer = Vec::new();
    {
        let mut writer = PcdWriter::new(&mut buffer);
        writer.write_pcd(&header, &block).expect("Write failed");
    }

    let reader = PcdReader::new(Cursor::new(buffer)).expect("Reader creation failed");
    let read_block = reader.read_all().expect("Read failed");

    assert_eq!(read_block.len, num_points);
    let x_col = read_block.get_column("x").unwrap().as_f32().unwrap();
    let id_col = read_block.get_column("id").unwrap().as_u32().unwrap();

    for i in 0..num_points {
        assert_eq!(x_col[i], i as f32 * 0.5);
        assert_eq!(id_col[i], i as u32 + 500);
    }
}

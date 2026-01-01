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

//! # PCD-RS Example: Basic Usage
//!
//! This example demonstrates how to:
//! 1. Create a synthetic PCD file.
//! 2. Read it back using the `PcdReader`.
//! 3. Access data in a columnar (SoA) format.
//! 4. Use the `memmap` feature for zero-copy reading (if available).
//!
//! To run this example:
//! `cargo run --example basic_usage --features "rayon memmap2"`

use anyhow::Result;
use pcd_rs::header::ValueType;
use pcd_rs::header::{DataFormat, PcdHeader};
#[cfg(feature = "memmap2")]
use pcd_rs::io::PcdReader;
use pcd_rs::io::{PcdWriter, read_pcd_file};
use pcd_rs::storage::PointBlock;
use rand::Rng;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<()> {
    let filename = "example_data.pcd";

    // --- Step 1: Create and Write a PCD file ---
    println!("Creating {}...", filename);
    create_synthetic_pcd(filename)?;

    // --- Step 2: Read it back using standard Reader ---
    println!("Reading {} with standard reader...", filename);
    let block = read_pcd_file(filename)?;

    println!("Successfully read {} points.", block.len);

    // Inspect specific columns
    if let Some(x_col) = block.get_column("x") {
        if let Some(data) = x_col.as_f32_slice() {
            println!("First point X: {:.4}", data[0]);
            println!("Last point X:  {:.4}", data[data.len() - 1]);
        }
    }

    // --- Step 3: Zero-Copy / Mmap Usage (Optional) ---
    #[cfg(feature = "memmap2")]
    {
        println!("Reading {} with Mmap reader (Zero-Copy path)...", filename);
        let reader = PcdReader::from_path_mmap(filename)?;
        let header = reader.header();
        println!(
            "Header info: width={}, height={}, format={:?}",
            header.width, header.height, header.data
        );

        let block_mmap = reader.read_all()?;
        println!("Mmap read completed. Points: {}", block_mmap.len);
    }

    // --- Step 4: Performance Benchmark (XYZIRT) ---
    println!("\n--- Performance Benchmark (XYZIRT Format) ---");
    run_performance_test(100_000, DataFormat::Binary)?;
    run_performance_test(100_000, DataFormat::BinaryCompressed)?;

    run_performance_test(1_000_000, DataFormat::Binary)?;
    run_performance_test(1_000_000, DataFormat::BinaryCompressed)?;

    Ok(())
}

/// Runs a performance test for a given number of points and format using XYZIRT format.
fn run_performance_test(points: usize, format: DataFormat) -> Result<()> {
    println!("\nTesting {} points ({:?})...", points, format);
    let start_gen = Instant::now();

    // 1. Prepare Schema
    let schema = vec![
        ("x".to_string(), ValueType::F32),
        ("y".to_string(), ValueType::F32),
        ("z".to_string(), ValueType::F32),
        ("intensity".to_string(), ValueType::F32),
        ("ring".to_string(), ValueType::U16),
        ("timestamp".to_string(), ValueType::F64),
    ];

    // 2. Prepare Header
    let header = pcd_rs::header::PcdHeader {
        version: "0.7".to_string(),
        width: points as u32,
        points,
        data: format,
        fields: schema.iter().map(|(n, _)| n.clone()).collect(),
        sizes: vec![4, 4, 4, 4, 2, 8],
        types: vec!['F', 'F', 'F', 'F', 'U', 'F'],
        counts: vec![1, 1, 1, 1, 1, 1],
        ..Default::default()
    };

    let mut block = PointBlock::new(&schema, points);

    // 3. Generate Random Data
    {
        let names: Vec<String> = schema.iter().map(|(n, _)| n.clone()).collect();
        let mut cols = block.get_columns_mut(&names).unwrap();

        let (x_col, rest) = cols.split_first_mut().unwrap();
        let (y_col, rest) = rest.split_first_mut().unwrap();
        let (z_col, rest) = rest.split_first_mut().unwrap();
        let (i_col, rest) = rest.split_first_mut().unwrap();
        let (r_col, rest) = rest.split_first_mut().unwrap();
        let (ts_col, _) = rest.split_first_mut().unwrap();

        let x = x_col.as_f32_mut().unwrap();
        let y = y_col.as_f32_mut().unwrap();
        let z = z_col.as_f32_mut().unwrap();
        let intensity = i_col.as_f32_mut().unwrap();
        let ring = r_col.as_u16_mut().unwrap();
        let timestamp = ts_col.as_f64_mut().unwrap();

        let mut rng = rand::rng();
        for i in 0..points {
            x[i] = rng.random_range(-100.0..100.0);
            y[i] = rng.random_range(-100.0..100.0);
            z[i] = rng.random_range(-20.0..30.0);
            intensity[i] = rng.random();
            ring[i] = rng.random_range(0..64);
            timestamp[i] = 1700000000.0 + (i as f64) * 0.1;
        }
    }
    println!("  Data Generation:  {:?}", start_gen.elapsed());

    // 4. Write Test
    let format_str = match format {
        DataFormat::Binary => "binary",
        DataFormat::BinaryCompressed => "compressed",
        DataFormat::Ascii => "ascii",
    };
    let tmp_file = format!("perf_{}_{}.pcd", points, format_str);
    let start_write = Instant::now();
    {
        let file = File::create(&tmp_file)?;
        let buf_writer = BufWriter::new(file);
        let mut writer = PcdWriter::new(buf_writer);
        writer
            .write_pcd(&header, &block)
            .map_err(|e| anyhow::anyhow!(e))?;
    }
    println!("  {:?} Write:     {:?}", format, start_write.elapsed());

    // 5. Read Test
    let start_read = Instant::now();
    {
        let _read_block = read_pcd_file(&tmp_file)?;
    }
    println!("  {:?} Read:      {:?}", format, start_read.elapsed());

    #[cfg(feature = "memmap2")]
    {
        let start_mmap = Instant::now();
        // Mmap doesn't support compressed files directly in zero-copy way
        // because we still need to decompress. But our PcdReader handles it.
        let reader = PcdReader::from_path_mmap(&tmp_file)?;
        let _block_mmap = reader.read_all()?;
        println!("  {:?} Mmap Read: {:?}", format, start_mmap.elapsed());
    }

    // Cleanup removed as per user request to inspect files
    // let _ = std::fs::remove_file(tmp_file);

    Ok(())
}

/// Helper to create a dummy PCD file
fn create_synthetic_pcd<P: AsRef<Path>>(path: P) -> Result<()> {
    let points = 1000;

    // 1. Prepare Header
    let header = PcdHeader {
        version: "0.7".to_string(),
        fields: vec![
            "x".to_string(),
            "y".to_string(),
            "z".to_string(),
            "intensity".to_string(),
        ],
        sizes: vec![4, 4, 4, 4],
        types: vec!['F', 'F', 'F', 'F'],
        counts: vec![1, 1, 1, 1],
        width: points as u32,
        height: 1,
        viewpoint: [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        points,
        data: DataFormat::Binary,
    };

    // 2. Prepare Data (SoA)
    // We manually construct a PointBlock for writing.
    // In a real app, you might build this using `PointBlock::new` and filling columns.

    // Schema
    let schema = vec![
        ("x".to_string(), pcd_rs::header::ValueType::F32),
        ("y".to_string(), pcd_rs::header::ValueType::F32),
        ("z".to_string(), pcd_rs::header::ValueType::F32),
        ("intensity".to_string(), pcd_rs::header::ValueType::F32),
    ];

    let mut block = PointBlock::new(&schema, points);

    // Fill data
    // Fill data using multi-column mutable access
    {
        // We can request multiple mutable columns at once
        let names = vec![
            "x".to_string(),
            "y".to_string(),
            "z".to_string(),
            "intensity".to_string(),
        ];
        let mut cols = block.get_columns_mut(&names).unwrap();

        let (x_col, rest) = cols.split_first_mut().unwrap();
        let (y_col, rest) = rest.split_first_mut().unwrap();
        let (z_col, rest) = rest.split_first_mut().unwrap();
        let (i_col, _) = rest.split_first_mut().unwrap();

        let x = x_col.as_f32_mut().unwrap();
        let y = y_col.as_f32_mut().unwrap();
        let z = z_col.as_f32_mut().unwrap();
        let intens = i_col.as_f32_mut().unwrap();

        for i in 0..points {
            x[i] = (i as f32) * 0.1;
            y[i] = (i as f32) * 0.2;
            z[i] = (i as f32) * 0.3;
            intens[i] = 1.0;
        }
    }

    // 3. Write to File
    let file = File::create(path)?;
    let mut writer = PcdWriter::new(file);
    writer
        .write_pcd(&header, &block)
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}

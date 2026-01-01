# PCD-RS

**High-Performance Rust Library for Parsing Point Cloud Data (PCD)**

[![Crates.io](https://img.shields.io/crates/v/pcd-rs.svg)](https://crates.io/crates/pcd-rs)
[![Documentation](https://docs.rs/pcd-rs/badge.svg)](https://docs.rs/pcd-rs)

`pcd-rs` is a modern, pure-Rust library designed for parsing and processing PCD files with a focus on speed, safety, and correctness. It is built for high-throughput applications like autonomous driving and 3D perception.

## Features

- **🚀 High Performance**: Built from the ground up to be fast.
- **💾 Structure of Arrays (SoA)**: Data is stored in column-major format (`storage::PointBlock`), optimizing for SIMD and CPU cache locality.
- **⚡ Zero-Copy Support**: Supports memory-mapped (`mmap`) reading for handling large files without unnecessary copying.
- **🧵 Parallel Processing**: Optional `rayon` integration for parallel decoding of binary data.
- **📦 Comprehensive Format Support**:
  - `Data Formats`: ASCII, Binary, Binary Compressed (Read-only for compressed).
  - `Field Types`: Full support for `I8`, `I16`, `I32`, `U8`, `U16`, `U32`, `F32`, `F64`.
  - `Dynamic Schema`: Handles arbitrary field combinations (e.g., `x`, `y`, `z`, `intensity`, `timestamp`, `ring`, `label`).

### ✅ Supported Field Combinations
| Scenario | Common Fields | Notes |
|----------|---------------|-------|
| Basic    | `x`, `y`, `z` | Mandatory for 3D points |
| LiDAR    | `x`, `y`, `z`, `intensity` | Standard LiDAR format |
| Timestamp| `x`, `y`, `z`, `timestamp` | For motion compensation |
| Full Info| `x`, `y`, `z`, `intensity`, `ring`, `timestamp` | Velodyne/Robosense outputs |
| Semantic | `x`, `y`, `z`, `label`, `id` | Annotated data |
| RGB      | `x`, `y`, `z`, `rgb` | Parsed as f32/u32 (packed) |
## Installation

Add `pcd-rs` to your `Cargo.toml`:

```toml
[dependencies]
pcd-rs = { version = "0.1", features = ["rayon", "memmap2"] }
```

## Quick Start

### Reading a PCD File

```rust
use pcd_rs::io::read_pcd_file;

fn main() -> anyhow::Result<()> {
    // Read the entire file into memory (SoA block)
    let block = read_pcd_file("data.pcd")?;
    
    println!("Loaded {} points", block.len);
    
    // Access "x" coordinate column
    if let Some(x_col) = block.get_column("x") {
        // as_f32_slice() creates a safe view into the data
        if let Some(x_values) = x_col.as_f32_slice() {
            println!("First X: {}", x_values[0]);
        }
    }
    
    Ok(())
}
```

### Writing a PCD File

```rust
use pcd_rs::header::{PcdHeader, DataFormat, ValueType};
use pcd_rs::io::PcdWriter;
use pcd_rs::storage::PointBlock;
use std::fs::File;

fn main() -> anyhow::Result<()> {
    let num_points = 100;
    
    // 1. Define Schema and Data
    let schema = vec![
        ("x".to_string(), ValueType::F32),
        ("y".to_string(), ValueType::F32),
        ("z".to_string(), ValueType::F32),
    ];
    let mut block = PointBlock::new(&schema, num_points);
    
    // 2. Prepare Header
    let header = PcdHeader {
        width: num_points as u32,
        points: num_points,
        data: DataFormat::Binary,
        fields: vec!["x".into(), "y".into(), "z".into()],
        sizes: vec![4, 4, 4],
        types: vec!['F', 'F', 'F'],
        counts: vec![1, 1, 1],
        ..Default::default()
    };

    // 3. Write to File
    let file = File::create("output.pcd")?;
    let mut writer = PcdWriter::new(file);
    writer.write_pcd(&header, &block)?;
    
    Ok(())
}
```

### Advanced: Zero-Copy with Mmap

For handling files larger than available RAM or maximizing IO throughput:

```rust
use pcd_rs::io::PcdReader;

fn main() -> anyhow::Result<()> {
    let reader = PcdReader::from_path_mmap("huge_cloud.pcd")?;
    let block = reader.read_all()?;
    Ok(())
}
```

## Architecture

- `io`: High-level Reader/Writer interfaces.
- `header`: Robust header parsing with validation.
- `storage`: Columnar storage container (`PointBlock`).
- `layout`: Schema definition and memory layout calculations.
- `decoder`: Low-level parsers for different data formats.

## License

Apache-2.0

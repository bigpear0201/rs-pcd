# PCD-RS

**High-Performance Rust Library for Parsing Point Cloud Data (PCD)**

[![Crates.io](https://img.shields.io/crates/v/pcd-rs.svg)](https://crates.io/crates/pcd-rs)
[![Documentation](https://docs.rs/pcd-rs/badge.svg)](https://docs.rs/pcd-rs)

`pcd-rs` is a modern, pure-Rust library designed for parsing and processing PCD files with a focus on speed, safety, and correctness. It is built for high-throughput applications like autonomous driving and 3D perception.

## Features

- **🚀 High Performance**:
  - Batch reading (1024 points/batch) reduces syscalls by ~1000x
  - Platform-optimized endianness conversion (direct memory copy on Little Endian)
  - Vec-based column indexing for O(1) access
- **💾 Structure of Arrays (SoA)**: Data is stored in column-major format (`storage::PointBlock`), optimizing for SIMD and CPU cache locality.
- **⚡ Zero-Copy Support**: Supports memory-mapped (`mmap`) reading for handling large files without unnecessary copying.
- **🧵 Parallel Processing**: Optional `rayon` integration for parallel decoding of binary data.
- **🛠️ Developer-Friendly APIs**:
  - `PcdHeaderBuilder` for fluent header construction
  - `from_bytes()` for parsing in-memory data
  - Typed accessors: `xyz()`, `xyzi()`, `xyzrgb()`
- **📦 Comprehensive Format Support**:
  - `Data Formats`: ASCII, Binary, Binary Compressed (Read-only for compressed).
  - `Field Types`: Full support for `I8`, `I16`, `I32`, `U8`, `U16`, `U32`, `F32`, `F64`.
  - `Dynamic Schema`: Handles arbitrary field combinations (e.g., `x`, `y`, `z`, `intensity`, `timestamp`, `ring`, `label`).

### ⚡ Performance

Benchmarks on Apple Silicon (1M points, XYZIRT format, 30 bytes/point):

| Operation | Time | Throughput |
|-----------|------|------------|
| Binary Read | 86ms | ~350 MB/s |
| **Mmap Read** | **9.6ms** | **~3.1 GB/s** ⚡ |
| Compressed Read | 65ms | ~460 MB/s |

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
rs-pcd = { version = "0.2", features = ["rayon", "memmap2"] }
```

## Quick Start

### Reading a PCD File

```rust
use rs_pcd::io::read_pcd_file;

fn main() -> anyhow::Result<()> {
    // Read the entire file into memory (SoA block)
    let block = read_pcd_file("data.pcd")?;
    
    println!("Loaded {} points", block.len);
    
    // Method 1: Access by column name
    if let Some(x_col) = block.get_column("x") {
        if let Some(x_values) = x_col.as_f32_slice() {
            println!("First X: {}", x_values[0]);
        }
    }
    
    // Method 2: Typed accessor (v0.2+)
    if let Some((x, y, z)) = block.xyz() {
        println!("First point: ({}, {}, {})", x[0], y[0], z[0]);
    }
    
    Ok(())
}
```

### Writing a PCD File (v0.2+ with Builder)

```rust
use rs_pcd::header::{PcdHeaderBuilder, DataFormat, ValueType};
use rs_pcd::io::PcdWriter;
use rs_pcd::storage::PointBlock;
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
    
    // 2. Build Header with fluent API (v0.2+)
    let header = PcdHeaderBuilder::new()
        .add_field("x", ValueType::F32)
        .add_field("y", ValueType::F32)
        .add_field("z", ValueType::F32)
        .width(num_points as u32)
        .data_format(DataFormat::Binary)
        .build()?;

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
use rs_pcd::io::PcdReader;

fn main() -> anyhow::Result<()> {
    // Memory-mapped reading (zero-copy)
    let reader = PcdReader::from_path_mmap("huge_cloud.pcd")?;
    let block = reader.read_all()?;
    
    // With rayon feature enabled, binary decoding is parallelized automatically
    println!("Loaded {} points", block.len);
    Ok(())
}
```

### Parsing In-Memory Data (v0.2+)

```rust
use rs_pcd::io::PcdReader;

fn main() -> anyhow::Result<()> {
    let pcd_bytes: &[u8] = include_bytes!("embedded.pcd");
    
    // Parse directly from byte slice
    let reader = PcdReader::from_bytes(pcd_bytes)?;
    let block = reader.read_all()?;
    
    Ok(())
}
```

## API Reference

### Typed Accessors (v0.2+)

```rust
// Get XYZ coordinates
let (x, y, z) = block.xyz().unwrap();

// Get XYZ + intensity
let (x, y, z, intensity) = block.xyzi().unwrap();

// Get XYZ + RGB
let (x, y, z, rgb) = block.xyzrgb().unwrap();
```

### Indexed Access (v0.2+)

For performance-critical loops, use indexed access:

```rust
// O(1) access by index
let x_col = block.get_column_by_index(0).unwrap();

// Get column index from name
let x_idx = block.get_column_index("x").unwrap();
```

## Architecture

- `io`: High-level Reader/Writer interfaces.
- `header`: Robust header parsing with validation, includes `PcdHeaderBuilder`.
- `storage`: Columnar storage container (`PointBlock`).
- `layout`: Schema definition and memory layout calculations.
- `decoder`: Low-level parsers for different data formats (optimized batch reading).

## What's New in v0.2

- ⚡ **30-50% faster binary reading** via batch I/O (1024 points/batch)
- 🚀 **10-20% faster column access** with Vec-based indexing
- 🛠️ **PcdHeaderBuilder** for ergonomic header construction
- 📦 **from_bytes()** API for parsing in-memory data
- 🎯 **Typed accessors**: `xyz()`, `xyzi()`, `xyzrgb()`
- 🔒 **Removed unsafe transmute**, improved safety
- 🏎️ **Platform-optimized endianness** (direct copy on LE systems)

## License

Apache-2.0

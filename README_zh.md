# PCD-RS (中文文档)

**高性能 Rust 点云数据 (PCD) 解析库**

`pcd-rs` 是一个专为高性能点云处理设计的 Rust 库，旨在提供安全、快速且符合工程标准的 PCD 文件读写能力。特别适用于自动驾驶、机器人感知等对吞吐量有高要求的场景。

## 核心特性

- **🚀 极致性能**:
  - 批量读取（1024 点/批次）减少系统调用约 1000 倍
  - 平台优化的字节序转换（Little Endian 系统直接内存复制）
  - 基于 Vec 的列索引，O(1) 访问速度
- **💾 数组结构 (SoA)**: 采用列式存储 (`Structure of Arrays`)，相比传统的 AoS (Array of Structs) 极大提升了 CPU 缓存命中率和 SIMD 优化潜力。
- **⚡ 零拷贝 (Zero-Copy)**: 支持 `mmap` 内存映射读取，能够高效处理从几 MB 到几十 GB 的超大点云文件。
- **🧵 并行加速**: 集成 `rayon`，支持多核并行解码二进制数据。
- **🛠️ 开发者友好的 API**:
  - `PcdHeaderBuilder` 链式构造头部
  - `from_bytes()` 解析内存数据
  - 类型安全访问器：`xyz()`, `xyzi()`, `xyzrgb()`
- **📦 全格式与动态模式支持**:
  - `数据格式`: ASCII, Binary, Binary Compressed (目前仅支持读取压缩格式)
  - `字段类型`: 原生支持所有 PCD 类型 (`I8`, `I16`, `I32`, `U8`, `U16`, `U32`, `F32`, `F64`)
  - `动态 Schema`: 支持任意字段组合 (如 `x`, `y`, `z`, `intensity`, `timestamp`, `ring`, `label`, `rgb` 等自定义字段)

### ⚡ 性能表现

Apple Silicon 测试结果（100 万点，XYZIRT 格式，30 字节/点）：

| 操作 | 耗时 | 吞吐量 |
|------|------|--------|
| Binary 读取 | 86ms | ~350 MB/s |
| **Mmap 读取** | **9.6ms** | **~3.1 GB/s** ⚡ |
| Compressed 读取 | 65ms | ~460 MB/s |

### ✅ 支持的字段组合示例

| 场景 | 常见字段组合 | 备注 |
|------|------------|------|
| 基础点云 | `x`, `y`, `z` | 必选字段 |
| 带强度 | `x`, `y`, `z`, `intensity` | 激光雷达常见格式 |
| 带时间戳 | `x`, `y`, `z`, `timestamp` | 用于运动补偿 |
| 完整信息 | `x`, `y`, `z`, `intensity`, `ring`, `timestamp` | Velodyne/Robosense 等常见输出 |
| 语义分割 | `x`, `y`, `z`, `label`, `id` | 标注数据 |
| RGB 颜色 | `x`, `y`, `z`, `rgb` | 解析为 f32 或 u32 |

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
rs-pcd = { version = "0.2", features = ["rayon", "memmap2"] }
```

## 快速上手

### 读取 PCD 文件

```rust
use rs_pcd::io::read_pcd_file;

fn main() -> anyhow::Result<()> {
    // 读取文件到内存 (SoA Block)
    let block = read_pcd_file("data.pcd")?;
    
    println!("成功加载 {} 个点", block.len);
    
    // 方法 1: 通过列名访问
    if let Some(x_col) = block.get_column("x") {
        if let Some(x_values) = x_col.as_f32_slice() {
            println!("第一个点的 X 坐标: {}", x_values[0]);
        }
    }
    
    // 方法 2: 类型安全访问器 (v0.2+)
    if let Some((x, y, z)) = block.xyz() {
        println!("第一个点: ({}, {}, {})", x[0], y[0], z[0]);
    }
    
    Ok(())
}
```

### 写入 PCD 文件 (v0.2+ 使用 Builder)

```rust
use rs_pcd::header::{PcdHeaderBuilder, DataFormat, ValueType};
use rs_pcd::io::PcdWriter;
use rs_pcd::storage::PointBlock;
use std::fs::File;

fn main() -> anyhow::Result<()> {
    let num_points = 100;

    // 1. 定义 Schema 并创建数据块
    let schema = vec![
        ("x".to_string(), ValueType::F32),
        ("y".to_string(), ValueType::F32),
        ("z".to_string(), ValueType::F32),
    ];
    let mut block = PointBlock::new(&schema, num_points);

    // 2. 使用 Builder 构建 Header (v0.2+)
    let header = PcdHeaderBuilder::new()
        .add_field("x", ValueType::F32)
        .add_field("y", ValueType::F32)
        .add_field("z", ValueType::F32)
        .width(num_points as u32)
        .data_format(DataFormat::Binary)
        .build()?;

    // 3. 执行写入
    let file = File::create("output.pcd")?;
    let mut writer = PcdWriter::new(file);
    writer.write_pcd(&header, &block)?;

    Ok(())
}
```

### 高级用法：Mmap 零拷贝读取

当处理超大文件时，建议使用 `from_path_mmap`：

```rust
use rs_pcd::io::PcdReader;

fn main() -> anyhow::Result<()> {
    // 使用内存映射打开文件（零拷贝）
    let reader = PcdReader::from_path_mmap("huge_cloud.pcd")?;
    
    // 启用 rayon feature 后，二进制解码会自动并行化
    let block = reader.read_all()?;
    
    println!("加载了 {} 个点", block.len);
    Ok(())
}
```

### 解析内存数据 (v0.2+)

```rust
use rs_pcd::io::PcdReader;

fn main() -> anyhow::Result<()> {
    let pcd_bytes: &[u8] = include_bytes!("embedded.pcd");
    
    // 直接从字节切片解析
    let reader = PcdReader::from_bytes(pcd_bytes)?;
    let block = reader.read_all()?;
    
    Ok(())
}
```

## API 参考

### 类型安全访问器 (v0.2+)

```rust
// 获取 XYZ 坐标
let (x, y, z) = block.xyz().unwrap();

// 获取 XYZ + 强度
let (x, y, z, intensity) = block.xyzi().unwrap();

// 获取 XYZ + RGB
let (x, y, z, rgb) = block.xyzrgb().unwrap();
```

### 索引访问 (v0.2+)

对于性能关键的循环，使用索引访问：

```rust
// O(1) 通过索引访问
let x_col = block.get_column_by_index(0).unwrap();

// 从名称获取列索引
let x_idx = block.get_column_index("x").unwrap();
```

## 架构设计

- `io`: 提供高层读写接口 (`PcdReader`, `PcdWriter`)
- `header`: 严谨的头部解析与校验，包含 `PcdHeaderBuilder`
- `storage`: 核心数据容器 `PointBlock`，管理列式数据
- `layout`: 负责计算字段在内存与文件中的布局
- `decoder`: 底层解码器实现（优化的批量读取）

## v0.2 新特性

- ⚡ **二进制读取提速 30-50%**，采用批量 I/O（1024 点/批次）
- 🚀 **列访问提速 10-20%**，使用基于 Vec 的索引
- 🛠️ **PcdHeaderBuilder** 人性化的头部构造器
- 📦 **from_bytes()** API 用于解析内存数据
- 🎯 **类型安全访问器**：`xyz()`, `xyzi()`, `xyzrgb()`
- 🔒 **移除 unsafe transmute**，提升安全性
- 🏎️ **平台优化的字节序转换**（LE 系统直接复制）

## 许可证

Apache-2.0

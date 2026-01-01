# PCD-RS (中文文档)

**高性能 Rust 点云数据 (PCD) 解析库**

`pcd-rs` 是一个专为高性能点云处理设计的 Rust 库，旨在提供安全、快速且符合工程标准的 PCD 文件读写能力。特别适用于自动驾驶、机器人感知等对吞吐量有高要求的场景。

## 核心特性

- **🚀 极致性能**: 专为速度优化的解析路径。
- **💾 数组结构 (SoA)**: 采用列式存储 (`Structure of Arrays`)，相比传统的 AoS (Array of Structs) 极大提升了 CPU 缓存命中率和 SIMD 优化潜力。
- **⚡ 零拷贝 (Zero-Copy)**: 支持 `mmap` 内存映射读取，能够高效处理从几 MB 到几十 GB 的超大点云文件。
- **🧵 并行加速**: 集成 `rayon`，支持多核并行解码二进制数据 (Binary features)。
- **📦 全格式与动态模式支持**:
  - `数据格式`: ASCII, Binary, Binary Compressed (目前仅支持读取压缩格式).
  - `字段类型`: 原生支持所有 PCD 类型 (`I8`, `I16`, `I32`, `U8`, `U16`, `U32`, `F32`, `F64`).
  - `动态 Schema`: 支持任意字段组合 (如 `x`, `y`, `z`, `intensity`, `timestamp`, `ring`, `label`, `rgb` 等自定义字段).

### ✅ 支持的字段组合示例
| 场景 | 常见字段组合 | 备注 |
|------|------------|------|
| 基础点云 | `x`, `y`, `z` | 必选字段 |
| 带强度 | `x`, `y`, `z`, `intensity` | 激光雷达常见格式 |
| 带时间戳 | `x`, `y`, `z`, `timestamp` | 用于运动补偿 |
| 完整信息 | `x`, `y`, `z`, `intensity`, `ring`, `timestamp` | Velodyne/Robosense 等常见输出 |
| 语义分割 | `x`, `y`, `z`, `label`, `id` | 标注数据 |
| RGB 颜色 | `x`, `y`, `z`, `rgb` | 暂时将 rgb 解析为 f32 或 u32 |
## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
pcd-rs = { version = "0.1", features = ["rayon", "memmap2"] }
```

## 快速上手

### 读取 PCD 文件

```rust
use pcd_rs::io::read_pcd_file;

fn main() -> anyhow::Result<()> {
    // 读取文件到内存 (SoA Block)
    let block = read_pcd_file("data.pcd")?;
    
    println!("成功加载 {} 个点", block.len);
    
    // 获取 "x" 坐标列
    if let Some(x_col) = block.get_column("x") {
        // as_f32_slice() 返回 &[f32] 切片，无额外拷贝
        if let Some(x_values) = x_col.as_f32_slice() {
            println!("第一个点的 X 坐标: {}", x_values[0]);
        }
    }
    
    Ok(())
}
```

### 写入 PCD 文件

```rust
use pcd_rs::header::{PcdHeader, DataFormat, ValueType};
use pcd_rs::io::PcdWriter;
use pcd_rs::storage::PointBlock;
use std::fs::File;

fn main() -> anyhow::Result<()> {
    let num_points = 100;

    // 1. 定义 Schema 并创建数据块 (SoA)
    let schema = vec![
        ("x".to_string(), ValueType::F32),
        ("y".to_string(), ValueType::F32),
        ("z".to_string(), ValueType::F32),
    ];
    let mut block = PointBlock::new(&schema, num_points);

    // 2. 配置 Header
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
use pcd_rs::io::PcdReader;

fn main() -> anyhow::Result<()> {
    // 使用内存映射打开文件
    let reader = PcdReader::from_path_mmap("huge_cloud.pcd")?;
    
    // read_all 会自动利用 Rayon 进行并行解码（如果启用了 feature）
    let block = reader.read_all()?;
    
    Ok(())
}
```

## 架构设计

- `io`: 提供高层读写接口 (`PcdReader`, `PcdWriter`)。
- `header`: 严谨的头部解析与校验。
- `storage`: 核心数据容器 `PointBlock`，管理列式数据。
- `layout`: 负责计算字段在内存与文件中的布局。
- `decoder`: 底层解码器实现 (ASCII/Binary/Compressed)。

## 许可证

Apache-2.0

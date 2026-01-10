// Copyright 2025 bigpear0201
// 点云遍历示例

use anyhow::Result;
use rs_pcd::io::read_pcd_file;

fn main() -> Result<()> {
    // 读取点云文件（支持 Binary Compressed 格式）
    let block = read_pcd_file("data.pcd")?;
    
    println!("加载了 {} 个点", block.len);
    
    // ============================================
    // 方法 1: 使用类型安全访问器（推荐，最简洁）
    // ============================================
    if let Some((x, y, z)) = block.xyz() {
        println!("\n=== 方法 1: 类型安全访问器 ===");
        for i in 0..block.len.min(5) {
            println!("点 {}: ({:.3}, {:.3}, {:.3})", i, x[i], y[i], z[i]);
        }
    }
    
    // ============================================
    // 方法 2: 带强度的遍历
    // ============================================
    if let Some((x, y, z, intensity)) = block.xyzi() {
        println!("\n=== 方法 2: XYZ + 强度 ===");
        for i in 0..block.len.min(5) {
            println!(
                "点 {}: ({:.3}, {:.3}, {:.3}), 强度: {:.3}",
                i, x[i], y[i], z[i], intensity[i]
            );
        }
    }
    
    // ============================================
    // 方法 3: 手动获取列（灵活性最高）
    // ============================================
    println!("\n=== 方法 3: 手动获取列 ===");
    let x = block.get_column("x").unwrap().as_f32().unwrap();
    let y = block.get_column("y").unwrap().as_f32().unwrap();
    let z = block.get_column("z").unwrap().as_f32().unwrap();
    
    for i in 0..block.len.min(5) {
        println!("点 {}: ({:.3}, {:.3}, {:.3})", i, x[i], y[i], z[i]);
    }
    
    // ============================================
    // 方法 4: 访问任意字段（动态字段）
    // ============================================
    println!("\n=== 方法 4: 访问自定义字段 ===");
    
    // 检查是否有 timestamp 字段
    if let Some(ts_col) = block.get_column("timestamp") {
        if let Some(timestamps) = ts_col.as_f64() {
            println!("前 5 个点的时间戳:");
            for i in 0..block.len.min(5) {
                println!("  点 {}: {:.6}", i, timestamps[i]);
            }
        }
    }
    
    // 检查是否有 ring 字段（激光雷达线束）
    if let Some(ring_col) = block.get_column("ring") {
        if let Some(rings) = ring_col.as_u16() {
            println!("\n前 5 个点的线束编号:");
            for i in 0..block.len.min(5) {
                println!("  点 {}: ring {}", i, rings[i]);
            }
        }
    }
    
    // ============================================
    // 方法 5: 高性能遍历（使用索引访问）
    // ============================================
    println!("\n=== 方法 5: 高性能索引访问 ===");
    
    // 先获取列索引（只需一次）
    let x_idx = block.get_column_index("x").unwrap();
    let y_idx = block.get_column_index("y").unwrap();
    let z_idx = block.get_column_index("z").unwrap();
    
    // 使用索引访问（O(1)，无 HashMap 查找开销）
    let x = block.get_column_by_index(x_idx).unwrap().as_f32().unwrap();
    let y = block.get_column_by_index(y_idx).unwrap().as_f32().unwrap();
    let z = block.get_column_by_index(z_idx).unwrap().as_f32().unwrap();
    
    for i in 0..block.len.min(5) {
        println!("点 {}: ({:.3}, {:.3}, {:.3})", i, x[i], y[i], z[i]);
    }
    
    // ============================================
    // 方法 6: 并行处理（需要 rayon feature）
    // ============================================
    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        
        println!("\n=== 方法 6: 并行处理 ===");
        
        if let Some((x, y, z)) = block.xyz() {
            // 并行计算所有点到原点的距离
            let distances: Vec<f32> = (0..block.len)
                .into_par_iter()
                .map(|i| {
                    (x[i] * x[i] + y[i] * y[i] + z[i] * z[i]).sqrt()
                })
                .collect();
            
            println!("前 5 个点到原点的距离:");
            for i in 0..5.min(distances.len()) {
                println!("  点 {}: {:.3} 米", i, distances[i]);
            }
        }
    }
    
    // ============================================
    // 方法 7: 遍历所有字段（查看 schema）
    // ============================================
    println!("\n=== 方法 7: 查看所有字段 ===");
    println!("Schema: {:?}", block.schema());
    println!("字段数量: {}", block.num_columns());
    
    for (idx, field_name) in block.schema().iter().enumerate() {
        let col = block.get_column_by_index(idx).unwrap();
        println!("  字段 {}: {} (长度: {})", idx, field_name, col.len());
    }
    
    Ok(())
}

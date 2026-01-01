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

#[cfg(feature = "memmap2")]
use pcd_rs::header::{DataFormat, PcdHeader};
#[cfg(feature = "memmap2")]
use pcd_rs::io::PcdReader;
#[cfg(feature = "memmap2")]
use pcd_rs::io::read_pcd_file;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_large_file_binary_mmap_rayon() {
    let points = 100_000; // 100k points
    let mut file = NamedTempFile::new().unwrap();

    writeln!(file, "VERSION .7").unwrap();
    writeln!(file, "FIELDS x y z").unwrap();
    writeln!(file, "SIZE 4 4 4").unwrap();
    writeln!(file, "TYPE F F F").unwrap();
    writeln!(file, "COUNT 1 1 1").unwrap();
    writeln!(file, "WIDTH {}", points).unwrap();
    writeln!(file, "HEIGHT 1").unwrap();
    writeln!(file, "POINTS {}", points).unwrap();
    writeln!(file, "DATA binary").unwrap();

    // Write 100k points (3 floats each) = 1.2MB.
    // Small enough for quick test, large enough to trigger loops.
    let mut data = Vec::with_capacity(points * 12);
    for i in 0..points {
        let val = i as f32;
        data.extend_from_slice(&val.to_le_bytes()); // x
        data.extend_from_slice(&(val * 2.0).to_le_bytes()); // y
        data.extend_from_slice(&(val * 3.0).to_le_bytes()); // z
    }
    file.write_all(&data).unwrap();

    // Read using mmap
    #[cfg(feature = "memmap2")]
    {
        let path = file.path();
        let reader = PcdReader::from_path_mmap(path).expect("Failed to open mmap");
        let block = reader.read_all().expect("Failed to read all");

        assert_eq!(block.len, points);
        let x_col = block.get_column("x").unwrap().as_f32_slice().unwrap();
        assert_eq!(x_col[123], 123.0);
        assert_eq!(x_col[points - 1], (points - 1) as f32);
    }
}

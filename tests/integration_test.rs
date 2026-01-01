use pcd_rs::header::{DataFormat, PcdHeader};
use pcd_rs::io::read_pcd_file;
// use pcd_rs::storage::PointBlock;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_dummy_pcd_ascii() -> (NamedTempFile, PcdHeader) {
    let mut file = NamedTempFile::new().unwrap();
    let content = r#"# .PCD v.7 - Point Cloud Data file format
VERSION .7
FIELDS x y z intensity
SIZE 4 4 4 4
TYPE F F F F
COUNT 1 1 1 1
WIDTH 2
HEIGHT 1
VIEWPOINT 0 0 0 1 0 0 0
POINTS 2
DATA ascii
0.1 0.2 0.3 0.5
1.1 1.2 1.3 0.8
"#;
    write!(file, "{}", content).unwrap();

    let header = PcdHeader {
        version: ".7".to_string(),
        fields: vec![
            "x".to_string(),
            "y".to_string(),
            "z".to_string(),
            "intensity".to_string(),
        ],
        sizes: vec![4, 4, 4, 4],
        types: vec!['F', 'F', 'F', 'F'],
        counts: vec![1, 1, 1, 1],
        width: 2,
        height: 1,
        viewpoint: [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        points: 2,
        data: DataFormat::Ascii,
    };

    (file, header)
}

#[test]
fn test_parse_ascii() {
    let (file, _expected_header) = create_dummy_pcd_ascii();
    let path = file.path();

    let block = read_pcd_file(path).expect("Failed to parse pcd");

    assert_eq!(block.len, 2);

    let x_col = block.get_column("x").unwrap().as_f32_slice().unwrap();
    assert_eq!(x_col[0], 0.1);
    assert_eq!(x_col[1], 1.1);
}

#[test]
fn test_parse_binary() {
    // Create binary file
    let mut file = NamedTempFile::new().unwrap();
    let header_str =
        "VERSION .7\nFIELDS x\nSIZE 4\nTYPE F\nCOUNT 1\nWIDTH 2\nHEIGHT 1\nPOINTS 2\nDATA binary\n";
    file.write_all(header_str.as_bytes()).unwrap();

    // Data: 2 floats (Little Endian)
    let val1: f32 = 42.0;
    let val2: f32 = 123.0;
    file.write_all(&val1.to_le_bytes()).unwrap();
    file.write_all(&val2.to_le_bytes()).unwrap();

    let path = file.path();
    let block = read_pcd_file(path).expect("Failed to parse binary pcd");

    let x_col = block.get_column("x").unwrap().as_f32_slice().unwrap();
    assert_eq!(x_col[0], 42.0);
    assert_eq!(x_col[1], 123.0);
}

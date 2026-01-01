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

use super::PcdHeader;
use crate::error::{PcdError, Result};
use std::io::BufRead;

pub fn parse_header<R: BufRead>(reader: &mut R) -> Result<PcdHeader> {
    let mut header = PcdHeader::default();
    let mut line_num = 0;

    // Explicitly set viewpoint default to identity
    header.viewpoint = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            return Err(PcdError::InvalidHeader {
                line: line_num,
                msg: "Unexpected EOF before DATA section".to_string(),
            });
        }
        line_num += 1;

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "VERSION" => {
                header.version = parts.get(1).map(|&s| s.to_string()).unwrap_or_default();
            }
            "FIELDS" => {
                header.fields = parts[1..].iter().map(|&s| s.to_string()).collect();
            }
            "SIZE" => {
                header.sizes = parse_vec(&parts[1..], line_num, "SIZE")?;
            }
            "TYPE" => {
                let type_chars: Result<Vec<char>> = parts[1..]
                    .iter()
                    .map(|s| {
                        if s.len() != 1 {
                            return Err(PcdError::InvalidHeader {
                                line: line_num,
                                msg: format!("Invalid TYPE: {}", s),
                            });
                        }
                        Ok(s.chars().next().unwrap())
                    })
                    .collect();
                header.types = type_chars?;
            }
            "COUNT" => {
                header.counts = parse_vec(&parts[1..], line_num, "COUNT")?;
            }
            "WIDTH" => {
                header.width = parse_single(parts.get(1), line_num, "WIDTH")?;
            }
            "HEIGHT" => {
                header.height = parse_single(parts.get(1), line_num, "HEIGHT")?;
            }
            "VIEWPOINT" => {
                let vp: Vec<f64> = parse_vec(&parts[1..], line_num, "VIEWPOINT")?;
                if vp.len() == 7 {
                    header.viewpoint.copy_from_slice(&vp);
                } else {
                    return Err(PcdError::InvalidHeader {
                        line: line_num,
                        msg: format!("VIEWPOINT expected 7 values, got {}", vp.len()),
                    });
                }
            }
            "POINTS" => {
                header.points = parse_single(parts.get(1), line_num, "POINTS")?;
            }
            "DATA" => {
                let fmt = parts.get(1).ok_or_else(|| PcdError::InvalidHeader {
                    line: line_num,
                    msg: "Missing DATA format".to_string(),
                })?;
                header.data = fmt.parse()?;

                // Post-processing: Handle optional COUNT
                if header.counts.is_empty() {
                    header.counts = vec![1; header.fields.len()];
                }

                // Validate header consistency
                validate_header(&header, line_num)?;

                return Ok(header);
            }
            _ => {
                // Ignore unknown fields as per spec? Or warn?
                // For valid PCD, usually we shouldn't see random stuff, but tolerance is good.
            }
        }
    }
}

fn parse_vec<T: std::str::FromStr>(parts: &[&str], line: usize, field: &str) -> Result<Vec<T>> {
    parts
        .iter()
        .map(|s| {
            s.parse::<T>().map_err(|_| PcdError::InvalidHeader {
                line,
                msg: format!("Invalid value for {}: {}", field, s),
            })
        })
        .collect()
}

fn parse_single<T: std::str::FromStr>(part: Option<&&str>, line: usize, field: &str) -> Result<T> {
    match part {
        Some(s) => s.parse::<T>().map_err(|_| PcdError::InvalidHeader {
            line,
            msg: format!("Invalid token for {}: {}", field, s),
        }),
        None => Err(PcdError::InvalidHeader {
            line,
            msg: format!("Missing value for {}", field),
        }),
    }
}

fn validate_header(header: &PcdHeader, line: usize) -> Result<()> {
    if header.fields.len() != header.sizes.len() {
        return Err(PcdError::InvalidHeader {
            line,
            msg: format!(
                "Mismatch in fields({}) and sizes({})",
                header.fields.len(),
                header.sizes.len()
            ),
        });
    }
    if header.fields.len() != header.types.len() {
        return Err(PcdError::InvalidHeader {
            line,
            msg: format!(
                "Mismatch in fields({}) and types({})",
                header.fields.len(),
                header.types.len()
            ),
        });
    }
    // Spec says COUNT is optional and defaults to 1.
    if header.counts.is_empty() {
        // If no COUNT line, fill with 1s
        // header is mutable reference, but here we can't mutate easily if validation is separated.
        // Actually, better to do this fixup before validation or allow mutation here?
        // Let's assume validation is read-only but we can modify in the parser loop after DATA found.
        // Wait, validate_header takes &PcdHeader. We cannot mutate.
        // We should move this logic to the main parser loop before calling validate_header.
        return Err(PcdError::InvalidHeader {
            line,
            msg: format!("Counts vector empty but fields present (logic error in parser)"),
        });
    } else if header.counts.len() != header.fields.len() {
        return Err(PcdError::InvalidHeader {
            line,
            msg: format!(
                "Mismatch in fields({}) and counts({})",
                header.fields.len(),
                header.counts.len()
            ),
        });
    }

    Ok(())
}

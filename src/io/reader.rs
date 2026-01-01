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

use crate::decoder::ascii::AsciiReader;
use crate::decoder::binary::BinaryReader;
#[cfg(feature = "rayon")]
use crate::decoder::binary_par::BinaryParallelDecoder;
use crate::decoder::compressed::CompressedReader;
use crate::error::Result;
use crate::header::{DataFormat, PcdHeader, parse_header};
use crate::layout::PcdLayout;
use crate::storage::PointBlock;

#[cfg(feature = "memmap2")]
use memmap2::Mmap;
use std::fs::File;
#[cfg(feature = "memmap2")]
use std::io::Cursor;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub enum InputSource<R: BufRead> {
    Reader(R),
    #[cfg(feature = "memmap2")]
    Mmap(Mmap),
}

pub struct PcdReader<R: BufRead> {
    source: InputSource<R>,
    header: PcdHeader,
    layout: PcdLayout,
    #[cfg(feature = "memmap2")]
    start_offset: usize, // Offset where data starts (after header)
}

impl<R: BufRead> PcdReader<R> {
    pub fn new(mut reader: R) -> Result<Self> {
        let (header, _bytes_read) = {
            let header = parse_header(&mut reader)?;
            (header, 0)
        };

        let layout = PcdLayout::from_header(&header)?;

        Ok(PcdReader {
            source: InputSource::Reader(reader),
            header,
            layout,
            #[cfg(feature = "memmap2")]
            start_offset: 0,
        })
    }
}

impl PcdReader<BufReader<File>> {
    #[cfg(feature = "memmap2")]
    pub fn from_path_mmap<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        // We mmap the whole file
        let mmap = unsafe { Mmap::map(&file)? };

        // Parse header from mmap slice
        let mut cursor = Cursor::new(&mmap[..]);
        let header = parse_header(&mut cursor)?;
        let pos = cursor.position() as usize; // This is the data start offset

        let layout = PcdLayout::from_header(&header)?;

        Ok(PcdReader {
            source: InputSource::Mmap(mmap),
            header,
            layout,
            start_offset: pos,
        })
    }
}

impl<R: BufRead> PcdReader<R> {
    pub fn header(&self) -> &PcdHeader {
        &self.header
    }

    pub fn read_all(mut self) -> Result<PointBlock> {
        let points = self.header.points;
        let mut block = PointBlock::new(
            &self
                .layout
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.type_))
                .collect(),
            points,
        );

        match &mut self.source {
            InputSource::Reader(reader) => match self.header.data {
                DataFormat::Binary => {
                    let mut decoder = BinaryReader::new(reader, &self.layout, points);
                    decoder.decode(&mut block)?;
                }
                DataFormat::BinaryCompressed => {
                    let mut decoder = CompressedReader::new(reader, &self.layout, points);
                    decoder.decode(&mut block)?;
                }
                DataFormat::Ascii => {
                    let mut decoder = AsciiReader::new(reader, &self.layout, points);
                    decoder.decode(&mut block)?;
                }
            },
            #[cfg(feature = "memmap2")]
            InputSource::Mmap(mmap) => {
                let data_slice = &mmap[self.start_offset..];

                match self.header.data {
                    DataFormat::Binary => {
                        #[cfg(feature = "rayon")]
                        {
                            // Use parallel decoder if enabled
                            let decoder = BinaryParallelDecoder::new(&self.layout, points);
                            decoder.decode_par(data_slice, &mut block)?;
                        }
                        #[cfg(not(feature = "rayon"))]
                        {
                            // Fallback to sequential using Cursor
                            let mut cursor = Cursor::new(data_slice);
                            let mut decoder = BinaryReader::new(&mut cursor, &self.layout, points);
                            decoder.decode(&mut block)?;
                        }
                    }
                    DataFormat::BinaryCompressed => {
                        // Parallel Compressed not implemented yet (needs chunks processing)
                        // Fallback to sequential
                        let mut cursor = Cursor::new(data_slice);
                        let mut decoder = CompressedReader::new(&mut cursor, &self.layout, points);
                        decoder.decode(&mut block)?;
                    }
                    DataFormat::Ascii => {
                        let mut cursor = Cursor::new(data_slice);
                        let mut decoder = AsciiReader::new(&mut cursor, &self.layout, points);
                        decoder.decode(&mut block)?;
                    }
                }
            }
        }
        Ok(block)
    }
}

pub fn read_pcd_file<P: AsRef<Path>>(path: P) -> Result<PointBlock> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let pcd_reader = PcdReader::new(reader)?;
    pcd_reader.read_all()
}

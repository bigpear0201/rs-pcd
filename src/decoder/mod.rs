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

use crate::error::Result;
use crate::storage::PointBlock;

pub mod ascii;
pub mod binary;
#[cfg(feature = "rayon")]
pub mod binary_par;
pub mod compressed;

pub trait PcdDecoder {
    fn decode(&mut self, output: &mut PointBlock) -> Result<()>;
}

// Helper trait for parallel decoding
#[cfg(feature = "rayon")]
pub trait ParPcdDecoder {
    fn decode_par(&self, data: &[u8], output: &mut PointBlock) -> Result<()>;
}

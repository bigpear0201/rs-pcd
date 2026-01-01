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

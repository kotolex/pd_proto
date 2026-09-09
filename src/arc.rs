//! arc module used for compress and decompress utf-8 encoded strings

use flate2::Compression;
use flate2::write::ZlibDecoder;
use flate2::write::ZlibEncoder;
use std::io::Write;

pub fn compress(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed_bytes = encoder.finish()?;
    Ok(compressed_bytes)
}

pub fn decompress(compressed_data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = ZlibDecoder::new(Vec::new());
    decoder.write_all(compressed_data)?;
    let decompressed_bytes = decoder.finish()?;
    Ok(decompressed_bytes)
}

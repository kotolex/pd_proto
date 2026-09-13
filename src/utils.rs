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
pub fn dec_places(f: f64) -> usize {
    let cv = f.to_string();
    if let Some((_, pos)) = cv.split_once(".") {
        let mut index = pos.len();
        while pos[index..] == *"0" {
            index -= 1
        }
        pos[0..index].len()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dec_places() {
        assert_eq!(dec_places(123.456), 3);
        assert_eq!(dec_places(7.0), 0);
        assert_eq!(dec_places(0.0001), 4);
        assert_eq!(dec_places(12.123000), 3);
        assert_eq!(dec_places(3.14), 2);
        assert_eq!(dec_places(512.1432456), 7);
    }
}

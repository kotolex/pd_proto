use flate2::Compression;
use flate2::write::ZlibDecoder;
use flate2::write::ZlibEncoder;
use memmap2::Mmap;
use pyo3::PyResult;
use simd_adler32::Adler32;
use std::fs::File;
use std::io::Write;
use std::mem::ManuallyDrop;

#[cfg(unix)]
use std::os::fd::FromRawFd;
#[cfg(windows)]
use std::os::windows::io::FromRawHandle;
pub fn bytes_by_file_descriptor(file_descriptor: i64) -> PyResult<Mmap> {
    let file = unsafe {
        #[cfg(unix)]
        {
            File::from_raw_fd(file_descriptor as std::os::fd::RawFd)
        }
        #[cfg(windows)]
        {
            File::from_raw_handle(file_descriptor as std::os::windows::io::RawHandle)
        }
    };
    let file = ManuallyDrop::new(file);
    let mmap = unsafe { Mmap::map(&*file)? };
    Ok(mmap)
}

pub fn adlers(bytes: &[u8]) -> u32 {
    let mut adler = Adler32::new();
    adler.write(bytes);
    adler.finish()
}
pub fn adler(file_descriptor: i64) -> PyResult<u32> {
    let mm = bytes_by_file_descriptor(file_descriptor)?;
    let bts: &[u8] = &mm;
    Ok(adlers(bts))
}

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

    #[test]
    fn test_adler() {
        assert_eq!(adlers(&[1, 2, 3]), 851975);
    }
}

use crate::constants::{DEFAULT_CACHE_CAPACITY, DEFAULT_CACHE_STRING_LIMIT, DEFAULT_INT_LIMIT, FLOAT_DEFAULT_LIMIT};
use flate2::Compression;
use flate2::write::ZlibDecoder;
use flate2::write::ZlibEncoder;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
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

#[derive(Clone, Copy)]
struct SafeFloat(f64);

impl SafeFloat {
    fn new(val: f64) -> Self {
        assert!(val.is_finite(), "NaN ot Inf!");
        SafeFloat(val)
    }
}

impl PartialEq for SafeFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SafeFloat {}

impl Hash for SafeFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

pub struct DecodeOptions {
    pub max_depth: u32,
    cache_str: Vec<String>,
    cache_float: Vec<f64>,
    cache_int: Vec<i64>,
    str_index: u8,
    int_index: u8,
    float_index: u8,
}
impl DecodeOptions {
    pub fn new(max_depth: u32) -> Self {
        Self {
            max_depth,
            cache_str: Vec::with_capacity(DEFAULT_CACHE_CAPACITY),
            cache_float: Vec::with_capacity(DEFAULT_CACHE_CAPACITY),
            cache_int: Vec::with_capacity(DEFAULT_CACHE_CAPACITY),
            str_index: 0,
            float_index: 0,
            int_index: 0,
        }
    }
    pub fn add_string(&mut self, data: &String) {
        if self.str_index >= u8::MAX || data.len() > DEFAULT_CACHE_STRING_LIMIT {
            return;
        }
        self.cache_str.push(data.clone());
        self.str_index += 1;
    }

    pub fn add_float(&mut self, data: f64) {
        if self.float_index >= u8::MAX || !data.is_finite() {
            return;
        }
        self.cache_float.push(data);
        self.float_index += 1;
    }

    pub fn add_int(&mut self, data: i64) {
        if self.int_index >= u8::MAX || data < DEFAULT_INT_LIMIT {
            return;
        }
        self.cache_int.push(data);
        self.int_index += 1;
    }

    pub fn get_string(&self, index: u8) -> Option<&String> {
        self.cache_str.get(index as usize)
    }

    pub fn get_float(&self, index: u8) -> Option<&f64> {
        self.cache_float.get(index as usize)
    }

    pub fn get_int(&self, index: u8) -> Option<&i64> {
        self.cache_int.get(index as usize)
    }
}

pub struct Options {
    pub max_depth: u32,
    pub string_length_limit: usize,
    pub float_limit: f64,
    cache_str: HashMap<Vec<u8>, u8>,
    cache_float: HashMap<SafeFloat, u8>,
    cache_int: HashMap<i64, u8>,
    str_index: u8,
    float_index: u8,
    int_index: u8,
}

impl Options {
    pub fn new(max_depth: u32, string_length_limit: usize, float_limit: f64) -> Self {
        Self {
            max_depth,
            string_length_limit,
            float_limit,
            cache_str: HashMap::with_capacity(DEFAULT_CACHE_CAPACITY),
            cache_float: HashMap::with_capacity(DEFAULT_CACHE_CAPACITY),
            cache_int: HashMap::with_capacity(DEFAULT_CACHE_CAPACITY),
            str_index: 0,
            float_index: 0,
            int_index: 0,
        }
    }

    pub fn new_default() -> Self {
        Self::new(1000, 100, FLOAT_DEFAULT_LIMIT)
    }

    pub fn add_string(&mut self, data: &[u8]) {
        if self.str_index >= u8::MAX && data.len() > DEFAULT_CACHE_STRING_LIMIT {
            return;
        }
        if !self.cache_str.contains_key(data) {
            self.cache_str.insert(data.to_owned(), self.str_index);
            self.str_index += 1;
        }
    }

    pub fn add_float(&mut self, data: f64) {
        if self.float_index >= u8::MAX || !data.is_finite() {
            return;
        }
        if !self.cache_float.contains_key(&SafeFloat(data)) {
            self.cache_float.insert(SafeFloat(data), self.float_index);
            self.float_index += 1;
        }
    }

    pub fn add_int(&mut self, data: i64) {
        if self.int_index >= u8::MAX || data < DEFAULT_INT_LIMIT {
            return;
        }
        if !self.cache_int.contains_key(&data) {
            self.cache_int.insert(data, self.int_index);
            self.int_index += 1;
        }
    }
    pub fn get_string_index(&self, data: &[u8]) -> Option<u8> {
        self.cache_str.get(data).copied()
    }

    pub fn get_float_index(&self, data: f64) -> Option<u8> {
        if !data.is_finite() {
            return None;
        }
        let sf = SafeFloat::new(data);
        self.cache_float.get(&sf).copied()
    }

    pub fn get_int_index(&self, data: i64) -> Option<u8> {
        self.cache_int.get(&data).copied()
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

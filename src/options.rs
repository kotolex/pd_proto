use crate::constants::{
    DEFAULT_CACHE_CAPACITY, DEFAULT_CACHE_STRING_LIMIT, DEFAULT_INT_LIMIT, FLOAT_DEFAULT_LIMIT,
};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy)]
struct SafeFloat(f64);

impl SafeFloat {
    fn new(val: f64) -> Self {
        assert!(val.is_finite(), "NaN or Inf!");
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
    pub fn add_string(&mut self, data: &str) {
        if self.str_index == u8::MAX || data.len() > DEFAULT_CACHE_STRING_LIMIT {
            return;
        }
        self.cache_str.push(data.to_owned());
        self.str_index += 1;
    }

    pub fn add_float(&mut self, data: f64) {
        if self.float_index == u8::MAX || !data.is_finite() {
            return;
        }
        self.cache_float.push(data);
        self.float_index += 1;
    }

    pub fn add_int(&mut self, data: i64) {
        if self.int_index == u8::MAX || data.abs() < DEFAULT_INT_LIMIT {
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
        if self.str_index == u8::MAX || data.len() > DEFAULT_CACHE_STRING_LIMIT {
            return;
        }
        if !self.cache_str.contains_key(data) {
            self.cache_str.insert(data.to_owned(), self.str_index);
            self.str_index += 1;
        }
    }

    pub fn add_float(&mut self, data: f64) {
        if self.float_index == u8::MAX || !data.is_finite() {
            return;
        }
        if !self.cache_float.contains_key(&SafeFloat(data)) {
            self.cache_float.insert(SafeFloat(data), self.float_index);
            self.float_index += 1;
        }
    }

    pub fn add_int(&mut self, data: i64) {
        if self.int_index == u8::MAX || data.abs() < DEFAULT_INT_LIMIT {
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
    fn test_string_dec_opts() {
        let mut d_o = DecodeOptions::new(100);
        d_o.add_string(&"1".to_string());
        let big = "a".repeat(255);
        d_o.add_string(&big);
        assert_eq!("1", d_o.get_string(0).unwrap());
        assert_eq!(true, d_o.get_string(1).is_none());
        assert_eq!(1, d_o.str_index as usize);
    }

    #[test]
    fn test_float_dec_opts() {
        let mut d_o = DecodeOptions::new(100);
        d_o.add_float(3.14);
        d_o.add_float(5.251);
        assert_eq!(true, d_o.get_float(0).is_some());
        assert_eq!(3.14, *d_o.get_float(0).unwrap());
        assert_eq!(true, d_o.get_float(1).is_some());
        assert_eq!(5.251, *d_o.get_float(1).unwrap());
        assert_eq!(2, d_o.float_index as usize);
    }

    #[test]
    fn test_int_dec_opts() {
        let mut d_o = DecodeOptions::new(100);
        d_o.add_int(100);
        d_o.add_int(100500);
        assert_eq!(true, d_o.get_int(0).is_some());
        assert_eq!(100500, *d_o.get_int(0).unwrap());
        assert_eq!(true, d_o.get_int(1).is_none());
        assert_eq!(1, d_o.int_index as usize);
    }

    #[test]
    fn test_int_dec_opts_limit() {
        let mut d_o = DecodeOptions::new(100);
        for i in 0..260 {
            d_o.add_int(17000 + i);
        }
        assert_eq!(true, d_o.get_int(254).is_some());
        assert_eq!(255, d_o.int_index as usize);
    }

    #[test]
    fn test_float_dec_opts_limit() {
        let mut d_o = DecodeOptions::new(100);
        for i in 0..260 {
            d_o.add_float(1.0 + (i as f64));
        }
        assert_eq!(true, d_o.get_float(254).is_some());
        assert_eq!(255, d_o.float_index as usize);
    }

    #[test]
    fn test_string_dec_opts_limit() {
        let mut d_o = DecodeOptions::new(100);
        for i in 0..260 {
            d_o.add_string(&i.to_string());
        }
        assert_eq!(true, d_o.get_string(254).is_some());
        assert_eq!(255, d_o.str_index as usize);
    }

    #[test]
    fn test_string_opts() {
        let mut d_o = Options::new_default();
        d_o.add_string("1".as_bytes());
        let binding = "a".repeat(255);
        let big = binding.as_bytes();
        d_o.add_string(&big);
        assert_eq!(0, d_o.get_string_index("1".as_bytes()).unwrap());
        assert_eq!(true, d_o.get_string_index(&big).is_none());
        assert_eq!(1, d_o.str_index as usize);
    }

    #[test]
    fn test_float_opts() {
        let mut d_o = Options::new_default();
        d_o.add_float(3.14);
        d_o.add_float(5.251);
        assert_eq!(true, d_o.get_float_index(3.14).is_some());
        assert_eq!(0, d_o.get_float_index(3.14).unwrap());
        assert_eq!(true, d_o.get_float_index(5.251).is_some());
        assert_eq!(1, d_o.get_float_index(5.251).unwrap());
        assert_eq!(2, d_o.float_index as usize);
    }

    #[test]
    fn test_int_opts() {
        let mut d_o = Options::new_default();
        d_o.add_int(100);
        d_o.add_int(100500);
        d_o.add_int(-100500);
        assert_eq!(true, d_o.get_int_index(100500).is_some());
        assert_eq!(0, d_o.get_int_index(100500).unwrap());
        assert_eq!(true, d_o.get_int_index(100).is_none());
        assert_eq!(true, d_o.get_int_index(-100500).is_some());
        assert_eq!(1, d_o.get_int_index(-100500).unwrap());
        assert_eq!(2, d_o.int_index as usize);
    }

    #[test]
    fn test_int_opts_limit() {
        let mut d_o = Options::new_default();
        for i in 0..260 {
            d_o.add_int(17000 + i);
        }
        assert_eq!(true, d_o.get_int_index(17200).is_some());
        assert_eq!(true, d_o.get_int_index(17256).is_none());
        assert_eq!(255, d_o.int_index as usize);
    }

    #[test]
    fn test_float_opts_limit() {
        let mut d_o = Options::new_default();
        for i in 0..260 {
            let v = 1.0 + (i as f64);
            d_o.add_float(v);
        }
        assert_eq!(true, d_o.get_float_index(220.0).is_some());
        assert_eq!(true, d_o.get_float_index(261.0).is_none());
        assert_eq!(255, d_o.float_index as usize);
    }

    #[test]
    fn test_str_opts_limit() {
        let mut d_o = Options::new_default();
        for i in 0..260 {
            let v = i.to_string() + "1";
            d_o.add_string(v.as_bytes());
        }
        assert_eq!(true, d_o.get_string_index("2201".as_bytes()).is_some());
        assert_eq!(true, d_o.get_string_index("2601".as_bytes()).is_none());
        assert_eq!(255, d_o.str_index as usize);
    }
}

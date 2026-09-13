use crate::constants::{INT_INDEX, LIST_INDEX, TEN, Variant};
use crate::options::DecodeOptions;
use crate::utils::decompress;
use pyo3::IntoPyObjectExt;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDateTime, PyDelta, PyDict, PySet, PyTuple, PyTzInfo};

const FLOAT_BYTES: usize = 8;
const EMPTY_VEC: Vec<ParsedData> = Vec::new();
const EMPTY_DICT: Vec<(ParsedData, ParsedData)> = Vec::new();

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedData {
    Null,
    BoolTrue,
    BoolFalse,
    BytesEmpty,
    Bytes(Vec<u8>),
    Int(i64),
    Float(f64),
    String(String),
    DateTimeNoTz(f64),
    DateTimeOffset((f64, i64)),
    DateTimeIana((f64, String)),
    List(Vec<ParsedData>),
    Tuple(Vec<ParsedData>),
    Set(Vec<ParsedData>),
    Dict(Vec<(ParsedData, ParsedData)>),
}

impl<'py> IntoPyObject<'py> for ParsedData {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            ParsedData::Null => Ok(py.None().into_bound_py_any(py)?),
            ParsedData::BoolTrue => Ok(true.into_bound_py_any(py)?),
            ParsedData::BoolFalse => Ok(false.into_bound_py_any(py)?),
            ParsedData::BytesEmpty => Ok(PyBytes::new(py, &[]).into_bound_py_any(py)?),
            ParsedData::Int(val) => Ok(val.into_bound_py_any(py)?),
            ParsedData::Float(val) => Ok(val.into_bound_py_any(py)?),
            ParsedData::String(val) => Ok(val.into_bound_py_any(py)?),
            ParsedData::List(val) => Ok(val.into_bound_py_any(py)?),
            ParsedData::DateTimeNoTz(val) => {
                let py_dt = PyDateTime::from_timestamp(py, val, None)?;
                Ok(py_dt.into_bound_py_any(py)?)
            }
            ParsedData::DateTimeOffset((ts, off)) => {
                let delta = PyDelta::new(py, 0, off as i32, 0, true)?;
                let tzinfo = PyTzInfo::fixed_offset(py, delta)?;
                let py_dt = PyDateTime::from_timestamp(py, ts, Some(&tzinfo))?;
                Ok(py_dt.into_bound_py_any(py)?)
            }
            ParsedData::DateTimeIana((ts, off)) => {
                let tzinfo = PyTzInfo::timezone(py, off)?;
                let py_dt = PyDateTime::from_timestamp(py, ts, Some(&tzinfo))?;
                Ok(py_dt.into_bound_py_any(py)?)
            }
            ParsedData::Tuple(val) => {
                let py_tuple = PyTuple::new(py, val)?;
                Ok(py_tuple.into_bound_py_any(py)?)
            }
            ParsedData::Set(val) => {
                let py_set = PySet::new(py, val)?;
                Ok(py_set.into_bound_py_any(py)?)
            }
            ParsedData::Dict(val) => {
                let py_dict = PyDict::new(py);
                for (k, v) in val {
                    py_dict.set_item(k, v)?;
                }
                Ok(py_dict.into_bound_py_any(py)?)
            }
            ParsedData::Bytes(val) => {
                let py_bytes = PyBytes::new(py, val.as_slice());
                Ok(py_bytes.into_bound_py_any(py)?)
            }
        }
    }
}

pub fn decode_varint(bts: &[u8], mut offset: usize) -> PyResult<(u64, usize)> {
    let start = offset;
    let mut number: u64 = 0;
    let mut shift = 0;
    loop {
        let &byte = bts.get(offset).ok_or_else(|| {
            PyValueError::new_err(format!("[END] Nothing to read at offset: {}", offset))
        })?;
        offset += 1;
        number |= ((byte & 0x7F) as u64) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift > 63 {
            // check var_int is too big for u64 (data corrupted)
            return Err(PyValueError::new_err(format!(
                "[END] Int is too long or data corrupted, offset: {}",
                offset
            )));
        }
    }
    Ok((number, offset - start))
}

fn decode_float(buffer: &[u8], offset: usize) -> PyResult<(f64, usize)> {
    if buffer.len() < offset + FLOAT_BYTES {
        let e_m = format!(
            "[FLOAT] Not enough bytes, need {}, but have only {} bytes left at offset {}",
            FLOAT_BYTES,
            buffer.len() - offset,
            offset
        );
        return Err(PyValueError::new_err(e_m));
    }
    match buffer[offset..offset + FLOAT_BYTES].try_into() {
        Ok(x) => Ok((f64::from_be_bytes(x), FLOAT_BYTES)),
        Err(_) => Err(PyValueError::new_err("[FLOAT] Wrong data for float")),
    }
}

fn decode_optimized_float(buffer: &[u8], offset: usize, tag: Variant) -> PyResult<(f64, usize)> {
    let (value, read) = decode_varint(buffer, offset)?;
    if tag == Variant::FloatNoDecimals {
        return Ok((value as f64, read));
    } else if tag == Variant::FloatNoDecimalsNeg {
        return Ok((-(value as f64), read));
    }
    if tag > Variant::FloatNoDecimals && tag <= Variant::Float6 {
        let dec_places = tag as u8 - 20; // cause FLOAT_1 = 21, FLOAT_2=22 etc.
        let pow = TEN.pow(dec_places as u32) as f64;
        let result = value as f64 / pow;
        Ok((result, read))
    } else {
        let dec_places = tag as u8 - 30; // cause FLOAT_1_NEG = 31, FLOAT_2_NEG=32 etc.
        let pow = TEN.pow(dec_places as u32) as f64;
        let result = value as f64 / pow;
        Ok((-result, read))
    }
}

fn decode_optimized_int(tag: Variant) -> i64 {
    if tag <= Variant::Int13 && tag >= Variant::Int1 {
        return tag as i64 - INT_INDEX as i64;
    }
    match tag {
        Variant::Int15 => 15,
        Variant::Int20 => 20,
        Variant::Int24 => 24,
        Variant::Int50 => 50,
        Variant::Int100 => 100,
        _ => 1000,
    }
}

fn decode_string(buffer: &[u8], offset: usize, tag: Variant) -> PyResult<(String, usize)> {
    let last_index;
    let mut new_offset = offset;
    let (value, read) = decode_varint(buffer, offset)?;
    if tag == Variant::String || tag == Variant::StringCompressed {
        last_index = read + (value as usize) + offset;
        if buffer.len() < last_index - 1 {
            let e_m = format!(
                "[STRING] Not enough bytes, need {}, but have only {} bytes left at offset {}",
                value,
                buffer.len() - offset,
                offset
            );
            return Err(PyValueError::new_err(e_m));
        }
        new_offset = offset + read;
    } else {
        last_index = (tag as u8 - 40) as usize + offset; // cause STRING_1=41 etc.
        if buffer.len() < last_index {
            let e_m = format!(
                "[STRING] Not enough bytes, need {}, but have only {} bytes left",
                value,
                buffer.len() - offset
            );
            return Err(PyValueError::new_err(e_m));
        }
    }
    let sub = &buffer[new_offset..last_index];
    let text = if tag == Variant::StringCompressed {
        decompress(sub)?
    } else {
        Vec::from(sub)
    };
    let string = String::from_utf8(text)?;
    Ok((string, last_index))
}

fn decode_list(
    buffer: &Vec<u8>,
    offset: usize,
    tag: Variant,
    opts: &mut DecodeOptions,
    current_depth: u32,
) -> PyResult<(Vec<ParsedData>, usize)> {
    let (elements_count, read) = if tag >= Variant::List1 && tag <= Variant::List10 {
        let elements_count: u64 = tag as u64 - LIST_INDEX as u64;
        (elements_count, 0)
    } else {
        decode_varint(buffer, offset)?
    };
    let mut result = Vec::with_capacity(elements_count as usize);
    let mut new_offset = offset + read;
    for _ in 0..elements_count {
        let (el, off) = decode(buffer, new_offset, opts, current_depth)?;
        result.push(el);
        new_offset = off
    }
    Ok((result, new_offset))
}

fn decode_tuple(
    buffer: &Vec<u8>,
    offset: usize,
    tag: Variant,
    opts: &mut DecodeOptions,
    current_depth: u32,
) -> PyResult<(Vec<ParsedData>, usize)> {
    let (elements_count, read) = if tag >= Variant::Tuple2 && tag <= Variant::Tuple5 {
        let elements_count: u64 = match tag {
            Variant::Tuple2 => 2,
            Variant::Tuple3 => 3,
            Variant::Tuple4 => 4,
            _ => 5,
        };
        (elements_count, 0)
    } else {
        decode_varint(buffer, offset)?
    };
    let mut result = Vec::with_capacity(elements_count as usize);
    let mut new_offset = offset + read;
    for _ in 0..elements_count {
        let (el, off) = decode(buffer, new_offset, opts, current_depth)?;
        result.push(el);
        new_offset = off
    }
    Ok((result, new_offset))
}

fn decode_set(
    buffer: &Vec<u8>,
    offset: usize,
    tag: Variant,
    opts: &mut DecodeOptions,
    current_depth: u32,
) -> PyResult<(Vec<ParsedData>, usize)> {
    let (result, new_offset) = decode_list(buffer, offset, tag, opts, current_depth)?;
    Ok((result, new_offset))
}

fn decode_dict(
    buffer: &Vec<u8>,
    offset: usize,
    opts: &mut DecodeOptions,
    current_depth: u32,
) -> PyResult<(Vec<(ParsedData, ParsedData)>, usize)> {
    let (elements_count, read) = decode_varint(buffer, offset)?;
    let mut result: Vec<(ParsedData, ParsedData)> = Vec::with_capacity(elements_count as usize);
    let mut new_offset = offset + read;
    for _ in 0..elements_count {
        let (key, off) = decode(buffer, new_offset, opts, current_depth)?;
        new_offset = off;
        let (value, off) = decode(buffer, new_offset, opts, current_depth)?;
        result.push((key, value));
        new_offset = off;
    }
    Ok((result, new_offset))
}

fn decode_bytes(buffer: &[u8], offset: usize) -> PyResult<(Vec<u8>, usize)> {
    let (size, read) = decode_varint(buffer, offset)?;
    let new_offset = offset + read;
    let last_index = new_offset + size as usize;
    if buffer.len() < last_index {
        let e_m = format!(
            "[BYTES] Not enough bytes, need {}, but have only {} bytes left at offset {}",
            last_index - offset,
            buffer.len() - offset,
            offset
        );
        return Err(PyValueError::new_err(e_m));
    }
    let data = buffer[new_offset..new_offset + size as usize].to_vec();
    Ok((data, new_offset + size as usize))
}

fn parse_float(
    buffer: &Vec<u8>,
    offset: usize,
    opts: &mut DecodeOptions,
    current_depth: u32,
) -> PyResult<(f64, usize)> {
    let (pd, new_offset) = decode(buffer, offset, opts, current_depth)?;
    match pd {
        ParsedData::Float(v) => Ok((v, new_offset)),
        _ => Err(PyValueError::new_err(
            "Unexpected type while parsing DateTime, expected Float",
        )),
    }
}

fn decode_cached_int(
    buffer: &[u8],
    offset: usize,
    opts: &mut DecodeOptions,
) -> PyResult<(ParsedData, usize)> {
    if let Some(&index) = buffer.get(offset) {
        match opts.get_int(index) {
            Some(value) => Ok((ParsedData::Int(*value), offset + 1)),
            None => {
                let e_m = format!(
                    "[CACHE] Unexpected integer cache fail: nothing at index {}",
                    index
                );
                Err(PyValueError::new_err(e_m))
            }
        }
    } else {
        let e_m = format!("[CACHE] No integer cache index at offset {}", offset);
        Err(PyValueError::new_err(e_m))
    }
}

fn decode_cached_float(
    buffer: &[u8],
    offset: usize,
    opts: &mut DecodeOptions,
) -> PyResult<(ParsedData, usize)> {
    if let Some(&index) = buffer.get(offset) {
        match opts.get_float(index) {
            Some(value) => Ok((ParsedData::Float(*value), offset + 1)),
            None => {
                let e_m = format!(
                    "[CACHE] Unexpected float cache fail: nothing at index {}",
                    index
                );
                Err(PyValueError::new_err(e_m))
            }
        }
    } else {
        let e_m = format!("[CACHE] No float cache index at offset {}", offset);
        Err(PyValueError::new_err(e_m))
    }
}

fn decode_cached_string(
    buffer: &[u8],
    offset: usize,
    opts: &mut DecodeOptions,
) -> PyResult<(ParsedData, usize)> {
    if let Some(&index) = buffer.get(offset) {
        match opts.get_string(index) {
            Some(value) => Ok((ParsedData::String((*value).parse()?), offset + 1)),
            None => {
                let e_m = format!(
                    "[CACHE] Unexpected string cache fail: nothing at index {}",
                    index
                );
                Err(PyValueError::new_err(e_m))
            }
        }
    } else {
        let e_m = format!("[CACHE] No string cache index at offset {}", offset);
        Err(PyValueError::new_err(e_m))
    }
}

fn decode(
    buffer: &Vec<u8>,
    offset: usize,
    opts: &mut DecodeOptions,
    current_depth: u32,
) -> PyResult<(ParsedData, usize)> {
    if opts.max_depth > 0 && current_depth > opts.max_depth {
        let e_m = format!(
            "[DEPTH] Nesting depth exceeded maximum of {}",
            opts.max_depth
        );
        return Err(PyValueError::new_err(e_m));
    }
    if let Some(&tag) = buffer.get(offset) {
        let new_offset = offset + 1;
        match Variant::try_from(tag) {
            Ok(Variant::Null) => Ok((ParsedData::Null, new_offset)),
            Ok(Variant::BoolTrue) => Ok((ParsedData::BoolTrue, new_offset)),
            Ok(Variant::BoolFalse) => Ok((ParsedData::BoolFalse, new_offset)),
            Ok(Variant::FloatZero) => Ok((ParsedData::Float(0.0), new_offset)),
            Ok(Variant::StringEmpty) => Ok((ParsedData::String("".to_string()), new_offset)),
            Ok(Variant::IntZero) => Ok((ParsedData::Int(0), new_offset)),
            Ok(Variant::ListEmpty) => Ok((ParsedData::List(EMPTY_VEC), new_offset)),
            Ok(Variant::TupleEmpty) => Ok((ParsedData::Tuple(EMPTY_VEC), new_offset)),
            Ok(Variant::SetEmpty) => Ok((ParsedData::Set(EMPTY_VEC), new_offset)),
            Ok(Variant::DictEmpty) => Ok((ParsedData::Dict(EMPTY_DICT), new_offset)),
            Ok(Variant::BytesEmpty) => Ok((ParsedData::BytesEmpty, new_offset)),
            Ok(Variant::CacheInt) => decode_cached_int(buffer, new_offset, opts),
            Ok(Variant::CacheFloat) => decode_cached_float(buffer, new_offset, opts),
            Ok(Variant::CacheString) => decode_cached_string(buffer, new_offset, opts),
            Ok(Variant::DateTimeNoTz) => {
                let (f, new_offset) = parse_float(buffer, new_offset, opts, current_depth)?;
                Ok((ParsedData::DateTimeNoTz(f), new_offset))
            }
            Ok(Variant::DateTimeOffset) => {
                let (f, new_offset) = parse_float(buffer, new_offset, opts, current_depth)?;
                let (pd, new_offset) = decode(buffer, new_offset, opts, current_depth)?;
                match pd {
                    ParsedData::Int(v) => Ok((ParsedData::DateTimeOffset((f, v)), new_offset)),
                    _ => Err(PyValueError::new_err(
                        "Unexpected type while parsing DateTime, expected Int",
                    )),
                }
            }
            Ok(Variant::DateTimeIana) => {
                let (f, new_offset) = parse_float(buffer, new_offset, opts, current_depth)?;
                let (pd, new_offset) = decode(buffer, new_offset, opts, current_depth)?;
                match pd {
                    ParsedData::String(v) => Ok((ParsedData::DateTimeIana((f, v)), new_offset)),
                    _ => Err(PyValueError::new_err(
                        "Unexpected type while parsing DateTimeIana, expected String",
                    )),
                }
            }
            Ok(Variant::Float) => {
                let (value, off) = decode_float(buffer, new_offset)?;
                opts.add_float(value);
                Ok((ParsedData::Float(value), new_offset + off))
            }
            Ok(i) if i >= Variant::Int1000 && i <= Variant::Int100 => {
                let value = decode_optimized_int(i);
                Ok((ParsedData::Int(value), new_offset))
            }
            Ok(t)
                if (t >= Variant::FloatNoDecimals && t <= Variant::Float6)
                    || (t >= Variant::FloatNoDecimalsNeg && t <= Variant::Float6Neg) =>
            {
                let (value, off) = decode_optimized_float(buffer, new_offset, t)?;
                opts.add_float(value);
                Ok((ParsedData::Float(value), new_offset + off))
            }
            Ok(Variant::IntPositive) => {
                let (value, off) = decode_varint(buffer, new_offset)?;
                opts.add_int(value as i64);
                Ok((ParsedData::Int(value as i64), new_offset + off))
            }
            Ok(Variant::IntNegative) => {
                let (value, off) = decode_varint(buffer, new_offset)?;
                opts.add_int(-(value as i64));
                Ok((ParsedData::Int(-(value as i64)), new_offset + off))
            }
            Ok(t) if (t >= Variant::StringCompressed && t <= Variant::String15) => {
                let (value, offset) = decode_string(buffer, new_offset, t)?;
                opts.add_string(&value);
                Ok((ParsedData::String(value), offset))
            }
            Ok(Variant::String) => {
                let (value, offset) = decode_string(buffer, new_offset, Variant::String)?;
                opts.add_string(&value);
                Ok((ParsedData::String(value), offset))
            }
            Ok(t) if t == Variant::List || (t >= Variant::List1 && t <= Variant::List10) => {
                let (value, offset) = decode_list(buffer, new_offset, t, opts, current_depth + 1)?;
                Ok((ParsedData::List(value), offset))
            }
            Ok(t) if t == Variant::Tuple || (t >= Variant::Tuple2 && t <= Variant::Tuple5) => {
                let (value, offset) = decode_tuple(buffer, new_offset, t, opts, current_depth + 1)?;
                Ok((ParsedData::Tuple(value), offset))
            }
            Ok(s) if s == Variant::Set => {
                let (value, offset) = decode_set(buffer, new_offset, s, opts, current_depth + 1)?;
                Ok((ParsedData::Set(value), offset))
            }
            Ok(Variant::Dict) => {
                let (value, offset) = decode_dict(buffer, new_offset, opts, current_depth + 1)?;
                Ok((ParsedData::Dict(value), offset))
            }
            Ok(Variant::Bytes) => {
                let (value, offset) = decode_bytes(buffer, new_offset)?;
                Ok((ParsedData::Bytes(value), offset))
            }
            _ => {
                let e_m = format!("[TAG] Unknown tag {}", tag);
                Err(PyValueError::new_err(e_m))
            }
        }
    } else {
        let e_m = format!("[DATA] Data corrupted at offset {}", offset);
        Err(PyValueError::new_err(e_m))
    }
}

pub fn unpack(
    py: Python<'_>,
    buffer: Vec<u8>,
    offset: usize,
    max_depth: i32,
) -> PyResult<(Bound<'_, PyAny>, usize)> {
    let real_depth = if max_depth < 0 { 0 } else { max_depth as u32 };
    let mut opts = DecodeOptions::new(real_depth);
    let (result, new_offset) = decode(&buffer, offset, &mut opts, 1)?;
    Ok((result.into_pyobject(py)?, new_offset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float() {
        let b = vec![1, 0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18];
        let (a, b) = decode_float(&b, 1).ok().unwrap();
        assert_eq!(a, std::f64::consts::PI);
        assert_eq!(b, 8);
    }

    #[test]
    fn test_varint() {
        let b = vec![1, 10, 255, 255, 255, 255, 255, 255, 255, 255, 127];
        let (a, b) = decode_varint(&b, 2).ok().unwrap();
        assert_eq!(a, 9_223_372_036_854_775_807);
        assert_eq!(9, b);
    }

    #[test]
    fn test_varint_negative() {
        let b = vec![1, 11, 128, 128, 128, 128, 128, 128, 128, 128, 128, 1];
        let (a, b) = decode_varint(&b, 2).ok().unwrap();
        assert_eq!(a, 9223372036854775808);
        assert_eq!(10, b);
    }

    #[test]
    fn test_dt_no_tz() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![1, 27, 12, 65, 218, 168, 5, 80, 74, 250, 240];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(a, ParsedData::DateTimeNoTz(1788876097.171566));
        assert_eq!(11, b);
    }

    #[test]
    fn test_dt_offset() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![1, 28, 12, 65, 218, 168, 5, 233, 49, 13, 246, 9];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(a, ParsedData::DateTimeOffset((1788876708.766477, 0)));
        assert_eq!(12, b);
    }

    #[test]
    fn test_dt_iana() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![
            1, 29, 12, 65, 218, 168, 12, 140, 185, 155, 145, 53, 69, 117, 114, 111, 112, 101, 47,
            76, 111, 110, 100, 111, 110,
        ];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(
            a,
            ParsedData::DateTimeIana((1788883506.90012, "Europe/London".to_string()))
        );
        assert_eq!(25, b);
    }

    #[test]
    fn test_dt_list() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![
            1, 14, 2, 28, 12, 65, 218, 168, 23, 128, 0, 0, 0, 10, 160, 56, 28, 12, 65, 218, 168,
            23, 128, 0, 0, 0, 10, 160, 56,
        ];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(
            a,
            ParsedData::List(vec![
                ParsedData::DateTimeOffset((1788894720.0, 7200)),
                ParsedData::DateTimeOffset((1788894720.0, 7200))
            ])
        );
        assert_eq!(29, b);
    }

    #[test]
    fn test_empty_bytes() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![1, 18];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(a, ParsedData::BytesEmpty);
        assert_eq!(2, b);
    }

    #[test]
    fn test_bytes() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![1, 19, 2, 1, 18];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(a, ParsedData::Bytes(vec![1, 18]));
        assert_eq!(5, b);
    }

    #[test]
    fn test_cache_string() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![1, 83, 44, 116, 101, 115, 116, 37, 0, 37, 0];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(
            a,
            ParsedData::List(vec![
                ParsedData::String("test".parse().unwrap()),
                ParsedData::String("test".parse().unwrap()),
                ParsedData::String("test".parse().unwrap()),
            ])
        );
        assert_eq!(11, b);
    }

    #[test]
    fn test_cache_string2() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![
            1, 84, 44, 116, 101, 115, 116, 45, 116, 101, 115, 116, 49, 37, 0, 37, 1,
        ];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(
            a,
            ParsedData::List(vec![
                ParsedData::String("test".parse().unwrap()),
                ParsedData::String("test1".parse().unwrap()),
                ParsedData::String("test".parse().unwrap()),
                ParsedData::String("test1".parse().unwrap()),
            ])
        );
        assert_eq!(17, b);
    }

    #[test]
    fn test_cache_floats() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![1, 84, 22, 186, 2, 22, 187, 2, 38, 0, 38, 1];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(
            a,
            ParsedData::List(vec![
                ParsedData::Float(3.14),
                ParsedData::Float(3.15),
                ParsedData::Float(3.14),
                ParsedData::Float(3.15),
            ])
        );
        assert_eq!(12, b);
    }

    #[test]
    fn test_cache_ints() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![1, 84, 10, 233, 7, 10, 232, 132, 1, 10, 233, 7, 39, 0];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(
            a,
            ParsedData::List(vec![
                ParsedData::Int(1001),
                ParsedData::Int(17000),
                ParsedData::Int(1001),
                ParsedData::Int(17000),
            ])
        );
        assert_eq!(14, b);
    }

    #[test]
    fn test_cache_dt_no_tz() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![
            1, 82, 27, 12, 65, 218, 169, 82, 17, 149, 203, 100, 27, 38, 0,
        ];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(
            a,
            ParsedData::List(vec![
                ParsedData::DateTimeNoTz(1789216838.340539),
                ParsedData::DateTimeNoTz(1789216838.340539),
            ])
        );
        assert_eq!(15, b);
    }

    #[test]
    fn test_cache_dt_offset() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![
            1, 82, 28, 12, 65, 218, 169, 83, 150, 221, 112, 197, 10, 224, 234, 4, 28, 38, 0, 39, 0,
        ];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(
            a,
            ParsedData::List(vec![
                ParsedData::DateTimeOffset((1789218395.460008, 79200)),
                ParsedData::DateTimeOffset((1789218395.460008, 79200)),
            ])
        );
        assert_eq!(21, b);
    }

    #[test]
    fn test_cache_dt_iana() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![
            1, 82, 29, 12, 65, 218, 169, 84, 2, 239, 134, 18, 53, 69, 117, 114, 111, 112, 101, 47,
            76, 111, 110, 100, 111, 110, 29, 38, 0, 37, 0,
        ];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(
            a,
            ParsedData::List(vec![
                ParsedData::DateTimeIana((1789218827.742558, "Europe/London".to_string())),
                ParsedData::DateTimeIana((1789218827.742558, "Europe/London".to_string())),
            ])
        );
        assert_eq!(31, b);
    }

    #[test]
    fn test_cache_negative_ints() {
        let mut opts = DecodeOptions::new(100);
        let b = vec![
            1, 86, 11, 232, 132, 1, 11, 240, 171, 1, 11, 1, 39, 0, 39, 1, 11, 1,
        ];
        let (a, b) = decode(&b, 1, &mut opts, 1).ok().unwrap();
        assert_eq!(
            a,
            ParsedData::List(vec![
                ParsedData::Int(-17000),
                ParsedData::Int(-22000),
                ParsedData::Int(-1),
                ParsedData::Int(-17000),
                ParsedData::Int(-22000),
                ParsedData::Int(-1),
            ])
        );
        assert_eq!(18, b);
    }
}

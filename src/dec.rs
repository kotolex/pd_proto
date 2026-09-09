use crate::arc::decompress;
use crate::constants::{TEN, Variant};
use pyo3::IntoPyObjectExt;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDateTime, PyDelta, PyDict, PySet, PyTuple, PyTzInfo};

const FLOAT_BYTES: usize = 8;

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
const EMPTY_VEC: Vec<ParsedData> = Vec::new();
const EMPTY_DICT: Vec<(ParsedData, ParsedData)> = Vec::new();
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

pub fn d_varint(bts: &Vec<u8>, offset: usize) -> PyResult<(u64, usize)> {
    let mut number: u64 = 0;
    let mut shift = 0;
    let mut bytes_read = 0;
    let safety_limit = 16;

    for &byte in &bts[offset..] {
        bytes_read += 1;
        number |= ((byte & 0x7F) as u64) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;

        if bytes_read >= safety_limit {
            let error_message = format!(
                "[END] Int is too long or data corrupted, offset: {}",
                offset + bytes_read
            );
            return Err(PyValueError::new_err(error_message));
        }
    }
    if bytes_read == 0 || (bytes_read == number as usize && number == 0) {
        let error_message = format!("[END] Nothing to read at offset: {}", offset);
        return Err(PyValueError::new_err(error_message));
    }
    Ok((number, bytes_read))
}

pub fn d_float(buffer: &Vec<u8>, offset: usize) -> PyResult<(f64, usize)> {
    if buffer.len() < offset + FLOAT_BYTES {
        let e_m = format!(
            "[FLOAT] Not enough bytes, need {}, but have only {} bytes left",
            FLOAT_BYTES,
            buffer.len() - offset
        );
        return Err(PyValueError::new_err(e_m));
    }
    match buffer[offset..offset + 8].try_into() {
        Ok(x) => Ok((f64::from_be_bytes(x), FLOAT_BYTES)),
        Err(_) => Err(PyValueError::new_err("[FLOAT] Wrong data for float")),
    }
}

pub fn d_optimized_float(buffer: &Vec<u8>, offset: usize, tag: u8) -> PyResult<(f64, usize)> {
    let (value, read) = d_varint(buffer, offset)?;
    if tag == Variant::FloatNoDecimals as u8 {
        return Ok((value as f64, read));
    } else if tag == Variant::FloatNoDecimalsNeg as u8 {
        return Ok((-(value as f64), read));
    }
    if tag > Variant::FloatNoDecimals as u8 && tag <= Variant::Float6 as u8 {
        let dec_places = tag - 20; // cause FLOAT_1 = 21, FLOAT_2=22 etc.
        let pow = TEN.pow(dec_places as u32) as f64;
        let result = value as f64 / pow;
        Ok((result, read))
    } else {
        let dec_places = tag - 30; // cause FLOAT_1_NEG = 31, FLOAT_2_NEG=32 etc.
        let pow = TEN.pow(dec_places as u32) as f64;
        let result = value as f64 / pow;
        Ok((-result, read))
    }
}

pub fn d_string(buffer: &Vec<u8>, offset: usize, tag: u8) -> PyResult<(String, usize)> {
    let last_index;
    let mut new_offset = offset;
    let (value, read) = d_varint(buffer, offset)?;
    if tag == Variant::String as u8 || tag == Variant::StringCompressed as u8 {
        last_index = read + (value as usize) + offset;
        if buffer.len() < last_index - 1 {
            let e_m = format!(
                "[STRING] Not enough bytes, need {}, but have only {} bytes left",
                value,
                buffer.len() - offset
            );
            return Err(PyValueError::new_err(e_m));
        }
        new_offset = offset + read;
    } else {
        last_index = (tag - 40) as usize + offset; // cause STRING_1=41 etc.
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
    let text = if tag == Variant::StringCompressed as u8 {
        decompress(sub)?
    } else {
        Vec::from(sub)
    };
    let string = String::from_utf8(text)?;
    Ok((string, last_index))
}

pub fn d_list(buffer: &Vec<u8>, offset: usize) -> PyResult<(Vec<ParsedData>, usize)> {
    let (elements_count, read) = d_varint(buffer, offset)?;
    let mut result = Vec::with_capacity(elements_count as usize);
    let mut new_offset = offset + read;
    for _ in 0..elements_count {
        let (el, off) = d_base(buffer, new_offset)?;
        result.push(el);
        new_offset = off
    }
    Ok((result, new_offset))
}

pub fn d_tuple(buffer: &Vec<u8>, offset: usize) -> PyResult<(Vec<ParsedData>, usize)> {
    let (result, new_offset) = d_list(buffer, offset)?;
    Ok((result, new_offset))
}

pub fn d_set(buffer: &Vec<u8>, offset: usize) -> PyResult<(Vec<ParsedData>, usize)> {
    let (result, new_offset) = d_list(buffer, offset)?;
    Ok((result, new_offset))
}

pub fn d_dict(buffer: &Vec<u8>, offset: usize) -> PyResult<(Vec<(ParsedData, ParsedData)>, usize)> {
    let (elements_count, read) = d_varint(buffer, offset)?;
    let mut result: Vec<(ParsedData, ParsedData)> = Vec::with_capacity(elements_count as usize);
    let mut new_offset = offset + read;
    for _ in 0..elements_count {
        let (key, off) = d_base(buffer, new_offset)?;
        new_offset = off;
        let (value, off) = d_base(buffer, new_offset)?;
        result.push((key, value));
        new_offset = off;
    }
    Ok((result, new_offset))
}

pub fn d_bytes(buffer: &Vec<u8>, offset: usize) -> PyResult<(Vec<u8>, usize)> {
    let (size, read) = d_varint(buffer, offset)?;
    let new_offset = offset + read;
    let data = buffer[new_offset..new_offset + size as usize].to_vec();
    Ok((data, new_offset + size as usize))
}

fn d_base(buffer: &Vec<u8>, offset: usize) -> PyResult<(ParsedData, usize)> {
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
            Ok(Variant::DateTimeNoTz) => {
                let (value, off) = d_float(buffer, new_offset + 1)?;
                Ok((ParsedData::DateTimeNoTz(value), new_offset + off + 1))
            }
            Ok(Variant::DateTimeOffset) => {
                let (value, off) = d_float(buffer, new_offset + 1)?;
                let (pd, new_offset) = d_base(buffer, new_offset + off + 1)?;
                match pd {
                    ParsedData::Int(v) => Ok((ParsedData::DateTimeOffset((value, v)), new_offset)),
                    _ => Err(PyValueError::new_err(
                        "Unexpected type while parsing DateTime, expected Int",
                    )),
                }
            }
            Ok(Variant::DateTimeIana) => {
                let (value, off) = d_float(buffer, new_offset + 1)?;
                let (pd, new_offset) = d_base(buffer, new_offset + off + 1)?;
                match pd {
                    ParsedData::String(v) => Ok((ParsedData::DateTimeIana((value, v)), new_offset)),
                    _ => Err(PyValueError::new_err(
                        "Unexpected type while parsing DateTimeIana, expected String",
                    )),
                }
            }
            Ok(Variant::Float) => {
                let (value, off) = d_float(buffer, new_offset)?;
                Ok((ParsedData::Float(value), new_offset + off))
            }
            Ok(t)
                if (t >= Variant::FloatNoDecimals && t <= Variant::Float6)
                    || (t >= Variant::FloatNoDecimalsNeg && t <= Variant::Float6Neg) =>
            {
                let (value, off) = d_optimized_float(buffer, new_offset, t as u8)?;
                Ok((ParsedData::Float(value), new_offset + off))
            }
            Ok(Variant::IntPositive) => {
                let (value, off) = d_varint(buffer, new_offset)?;
                Ok((ParsedData::Int(value as i64), new_offset + off))
            }
            Ok(Variant::IntNegative) => {
                let (value, off) = d_varint(buffer, new_offset)?;
                Ok((ParsedData::Int(-(value as i64)), new_offset + off))
            }
            Ok(t) if (t >= Variant::StringCompressed && t <= Variant::String15) => {
                let (value, offset) = d_string(buffer, new_offset, t as u8)?;
                Ok((ParsedData::String(value), offset))
            }
            Ok(Variant::String) => {
                let (value, offset) = d_string(buffer, new_offset, Variant::String as u8)?;
                Ok((ParsedData::String(value), offset))
            }
            Ok(Variant::List) => {
                let (value, offset) = d_list(buffer, new_offset)?;
                Ok((ParsedData::List(value), offset))
            }
            Ok(Variant::Tuple) => {
                let (value, offset) = d_tuple(buffer, new_offset)?;
                Ok((ParsedData::Tuple(value), offset))
            }
            Ok(Variant::Set) => {
                let (value, offset) = d_set(buffer, new_offset)?;
                Ok((ParsedData::Set(value), offset))
            }
            Ok(Variant::Dict) => {
                let (value, offset) = d_dict(buffer, new_offset)?;
                Ok((ParsedData::Dict(value), offset))
            }
            Ok(Variant::Bytes) => {
                let (value, offset) = d_bytes(buffer, new_offset)?;
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

pub fn r_decrypt_base(
    py: Python<'_>,
    buffer: Vec<u8>,
    offset: usize,
) -> PyResult<(Bound<'_, PyAny>, usize)> {
    let (result, new_offset) = d_base(&buffer, offset)?;
    Ok((result.into_pyobject(py)?, new_offset))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first() {
        let b = vec![1, 0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18];
        let (a, b) = d_float(&b, 1).ok().unwrap();
        assert_eq!(a, std::f64::consts::PI);
        assert_eq!(b, 8);
    }

    #[test]
    fn test_dt_no_tz() {
        let b = vec![1, 27, 12, 65, 218, 168, 5, 80, 74, 250, 240];
        let (a, b) = d_base(&b, 1).ok().unwrap();
        assert_eq!(a, ParsedData::DateTimeNoTz(1788876097.171566));
        assert_eq!(11, b);
    }

    #[test]
    fn test_dt_offset() {
        let b = vec![1, 28, 12, 65, 218, 168, 5, 233, 49, 13, 246, 9];
        let (a, b) = d_base(&b, 1).ok().unwrap();
        assert_eq!(a, ParsedData::DateTimeOffset((1788876708.766477, 0)));
        assert_eq!(12, b);
    }

    #[test]
    fn test_dt_iana() {
        let b = vec![
            1, 29, 12, 65, 218, 168, 12, 140, 185, 155, 145, 53, 69, 117, 114, 111, 112, 101, 47,
            76, 111, 110, 100, 111, 110,
        ];
        let (a, b) = d_base(&b, 1).ok().unwrap();
        assert_eq!(
            a,
            ParsedData::DateTimeIana((1788883506.90012, "Europe/London".to_string()))
        );
        assert_eq!(25, b);
    }

    #[test]
    fn test_dt_list() {
        let b = vec![
            1, 14, 2, 28, 12, 65, 218, 168, 23, 128, 0, 0, 0, 10, 160, 56, 28, 12, 65, 218, 168,
            23, 128, 0, 0, 0, 10, 160, 56,
        ];
        let (a, b) = d_base(&b, 1).ok().unwrap();
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
        let b = vec![1, 18];
        let (a, b) = d_base(&b, 1).ok().unwrap();
        assert_eq!(a, ParsedData::BytesEmpty);
        assert_eq!(2, b);
    }

    #[test]
    fn test_bytes() {
        let b = vec![1, 19, 2, 1, 18];
        let (a, b) = d_base(&b, 1).ok().unwrap();
        assert_eq!(a, ParsedData::Bytes(vec![1, 18]));
        assert_eq!(5, b);
    }
}

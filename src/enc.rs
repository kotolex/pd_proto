use crate::constants::*;
use crate::options::Options;
use crate::utils::{compress, dec_places};
use pyo3::exceptions::{PyAttributeError, PyOverflowError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{
    PyAnyMethods, PyBytes, PyDateTime, PyDelta, PyDeltaAccess, PyNone, PySet, PyString, PyType,
    PyTzInfoAccess,
};
use pyo3::types::{PyBool, PyFloat, PyInt, PyList, PyTuple};
use pyo3::types::{PyDict, PyListMethods};
use pyo3_file::PyFileLikeObject;
use std::io::{BufWriter, Write};

pub struct PyTypesCmp<'py> {
    pub bool_: Bound<'py, PyType>,
    pub int_: Bound<'py, PyType>,
    pub float_: Bound<'py, PyType>,
    pub none_: Bound<'py, PyType>,
    pub string_: Bound<'py, PyType>,
    pub bytes_: Bound<'py, PyType>,
    pub list_: Bound<'py, PyType>,
    pub tuple_: Bound<'py, PyType>,
    pub set_: Bound<'py, PyType>,
    pub dict_: Bound<'py, PyType>,
    pub datetime_: Bound<'py, PyType>,
}
impl<'py> PyTypesCmp<'py> {
    pub fn new(py: Python<'py>) -> Self {
        Self {
            bool_: py.get_type::<PyBool>(),
            float_: py.get_type::<PyFloat>(),
            int_: py.get_type::<PyInt>(),
            none_: py.get_type::<PyNone>(),
            string_: py.get_type::<PyString>(),
            bytes_: py.get_type::<PyBytes>(),
            list_: py.get_type::<PyList>(),
            tuple_: py.get_type::<PyTuple>(),
            set_: py.get_type::<PySet>(),
            dict_: py.get_type::<PyDict>(),
            datetime_: py.get_type::<PyDateTime>(),
        }
    }
}

pub fn encode_varint<W: Write>(mut number: u64, buffer: &mut W) -> PyResult<()> {
    let mut buf = [0u8; 10];
    let mut idx = 0;
    while number >= 0x80 {
        buf[idx] = (number as u8 & 0x7F) | 0x80;
        number >>= 7;
        idx += 1;
    }
    buf[idx] = number as u8;
    idx += 1;
    buffer.write_all(&buf[..idx])?;
    Ok(())
}

fn encode_int<W: Write>(num: i64, buffer: &mut W, opts: &mut Options) -> PyResult<()> {
    if num == 0 {
        buffer.write_all(&Variant::INT_ZERO_TAG)?;
        return Ok(());
    }
    if (1..=13).contains(&num) {
        let tag = Variant::opt_int_tag(num);
        buffer.write_all(&tag)?;
        return Ok(());
    }
    match num {
        15 => {
            buffer.write_all(&Variant::INT_TAG15)?;
            return Ok(());
        }
        20 => {
            buffer.write_all(&Variant::INT_TAG20)?;
            return Ok(());
        }
        24 => {
            buffer.write_all(&Variant::INT_TAG24)?;
            return Ok(());
        }
        50 => {
            buffer.write_all(&Variant::INT_TAG50)?;
            return Ok(());
        }
        100 => {
            buffer.write_all(&Variant::INT_TAG100)?;
            return Ok(());
        }
        1000 => {
            buffer.write_all(&Variant::INT_TAG1000)?;
            return Ok(());
        }
        _ => (),
    }
    match opts.get_int_index(num) {
        Some(index) => {
            buffer.write_all(&[Variant::CacheInt as u8, index])?;
            return Ok(());
        }
        None => {
            opts.add_int(num);
        }
    }
    let tag = if num < 0 {
        Variant::INT_NEGATIVE_TAG
    } else {
        Variant::INT_POSITIVE_TAG
    };
    let r_num: u64 = if num < 0 { -num as u64 } else { num as u64 };
    buffer.write_all(&tag)?;
    encode_varint(r_num, buffer)?;
    Ok(())
}

fn encode_none<W: Write>(buffer: &mut W) -> PyResult<()> {
    buffer.write_all(&Variant::NULL_TAG)?;
    Ok(())
}

fn encode_bool<W: Write>(value: bool, buffer: &mut W) -> PyResult<()> {
    if value {
        buffer.write_all(&Variant::BOOL_TRUE_TAG)?
    } else {
        buffer.write_all(&Variant::BOOL_FALSE_TAG)?
    };
    Ok(())
}

pub fn encode_float<W: Write>(number: f64, buffer: &mut W, opts: &mut Options) -> PyResult<()> {
    if number == 0.0 {
        buffer.write_all(&Variant::FLOAT_ZERO_TAG)?;
        return Ok(());
    }
    match opts.get_float_index(number) {
        Some(index) => {
            buffer.write_all(&[Variant::CacheFloat as u8, index])?;
            return Ok(());
        }
        None => opts.add_float(number),
    }
    let r_number = if number < 0.0 { -number } else { number };
    if opts.float_limit > 0.0 && r_number <= opts.float_limit {
        let dec_places = dec_places(r_number);
        if dec_places < 7 {
            let tag = tag_by_decimal_places(dec_places, number < 0.0);
            if dec_places == 0 {
                buffer.write_all(&[tag])?;
                encode_varint(r_number as u64, buffer)?;
                return Ok(());
            }
            let pow = TEN.pow(dec_places as u32) as f64;
            let limit = opts.float_limit / pow;
            if r_number < limit {
                let int_value = (r_number * pow).round() as u64;
                buffer.write_all(&[tag])?;
                encode_varint(int_value, buffer)?;
                return Ok(());
            }
        }
    }
    let mut bytes = [Variant::Float as u8, 0, 0, 0, 0, 0, 0, 0, 0];
    bytes[1..9].copy_from_slice(&number.to_be_bytes());
    buffer.write_all(&bytes)?;
    Ok(())
}

fn encode_datetime<W: Write>(
    py: Python<'_>,
    item: Bound<PyDateTime>,
    buffer: &mut W,
    opts: &mut Options,
) -> PyResult<()> {
    let timestamp: f64 = item.call_method0("timestamp")?.extract()?;
    match item.get_tzinfo() {
        Some(tz_info) => match tz_info.getattr("key") {
            Ok(key) => {
                let key_str = key.to_string();
                buffer.write_all(&Variant::DT_IANA_TAG)?;
                encode_float(timestamp, buffer, opts)?;
                encode_string(&key_str, buffer, opts)?;
            }
            Err(_) => {
                let delta: Bound<PyDelta> = tz_info
                    .call_method1("utcoffset", (PyNone::get(py),))
                    .unwrap()
                    .extract()?;
                let dt_offset: i32 = delta.get_seconds();
                buffer.write_all(&Variant::DT_OFFSET)?;
                encode_float(timestamp, buffer, opts)?;
                encode_int(dt_offset as i64, buffer, opts)?;
            }
        },
        None => {
            buffer.write_all(&Variant::DT_NAIVE)?;
            encode_float(timestamp, buffer, opts)?;
        }
    }
    Ok(())
}

fn encode_string<W: Write>(value: &str, buffer: &mut W, opts: &mut Options) -> PyResult<()> {
    if value.is_empty() {
        buffer.write_all(&Variant::STRING_EMPTY_TAG)?;
        return Ok(());
    }
    let encoded = value.as_bytes();
    let bytes_len = encoded.len();
    match opts.get_string_index(encoded) {
        Some(index) => {
            buffer.write_all(&[Variant::CacheString as u8, index])?;
            return Ok(());
        }
        None => opts.add_string(encoded),
    }
    if bytes_len <= 15 {
        let tag = Variant::opt_string_tag(bytes_len);
        let mut tmp = [0u8; 16];
        tmp[0] = tag[0];
        tmp[1..1 + bytes_len].copy_from_slice(encoded);
        buffer.write_all(&tmp[..1 + bytes_len])?;
        return Ok(());
    }
    if opts.string_length_limit > 0 && bytes_len > opts.string_length_limit {
        let compressed = compress(encoded)?;
        if compressed.len() < bytes_len + 3 {
            buffer.write_all(&Variant::STRING_COMPRESSED_TAG)?;
            encode_varint(compressed.len() as u64, buffer)?;
            buffer.write_all(&compressed)?;
            return Ok(());
        }
    }
    buffer.write_all(&Variant::STRING_TAG)?;
    encode_varint(bytes_len as u64, buffer)?;
    buffer.write_all(encoded)?;
    Ok(())
}

fn encode_bytes<W: Write>(value: &[u8], buffer: &mut W) -> PyResult<()> {
    if value.is_empty() {
        buffer.write_all(&Variant::BYTES_EMPTY_TAG)?;
    } else {
        buffer.write_all(&Variant::BYTES_TAG)?;
        encode_varint(value.len() as u64, buffer)?;
        buffer.write_all(value)?;
    }
    Ok(())
}

fn encode_dict<W: Write>(
    py: Python<'_>,
    a_dict: &Bound<'_, PyDict>,
    depth: u32,
    buffer: &mut W,
    opts: &mut Options,
    types: &PyTypesCmp,
) -> PyResult<()> {
    let length = a_dict.len();
    if length == 0 {
        buffer.write_all(&Variant::DICT_EMPTY_TAG)?;
        return Ok(());
    }
    buffer.write_all(&Variant::DICT_TAG)?;
    encode_varint(length as u64, buffer)?;
    for (key, value) in a_dict.iter() {
        encode(py, key, buffer, depth, opts, types)?;
        encode(py, value, buffer, depth, opts, types)?;
    }
    Ok(())
}

fn encode_set<W: Write>(
    py: Python<'_>,
    a_set: &Bound<'_, PySet>,
    depth: u32,
    buffer: &mut W,
    opts: &mut Options,
    types: &PyTypesCmp,
) -> PyResult<()> {
    let length = a_set.len();
    if length == 0 {
        buffer.write_all(&Variant::SET_EMPTY_TAG)?;
        return Ok(());
    }
    buffer.write_all(&Variant::SET_TAG)?;
    encode_varint(length as u64, buffer)?;
    for item in a_set.iter() {
        encode(py, item, buffer, depth, opts, types)?;
    }
    Ok(())
}

fn encode_tuple<W: Write>(
    py: Python<'_>,
    a_tuple: &Bound<'_, PyTuple>,
    depth: u32,
    buffer: &mut W,
    opts: &mut Options,
    types: &PyTypesCmp,
) -> PyResult<()> {
    let len = a_tuple.len();
    if len == 0 {
        buffer.write_all(&Variant::TUPLE_EMPTY_TAG)?;
        return Ok(());
    }
    match len {
        2 => {
            buffer.write_all(&Variant::TUPLE_TAG2)?;
        }
        3 => {
            buffer.write_all(&Variant::TUPLE_TAG3)?;
        }
        4 => {
            buffer.write_all(&Variant::TUPLE_TAG4)?;
        }
        5 => {
            buffer.write_all(&Variant::TUPLE_TAG5)?;
        }
        _ => {
            buffer.write_all(&Variant::TUPLE_TAG)?;
            encode_varint(len as u64, buffer)?;
        }
    }
    for item in a_tuple.iter() {
        encode(py, item, buffer, depth, opts, types)?;
    }
    Ok(())
}

fn encode_list<W: Write>(
    py: Python<'_>,
    list: &Bound<'_, PyList>,
    depth: u32,
    buffer: &mut W,
    opts: &mut Options,
    types: &PyTypesCmp,
) -> PyResult<()> {
    let length = list.len();
    if length == 0 {
        buffer.write_all(&Variant::LIST_EMPTY_TAG)?;
        return Ok(());
    }
    match length {
        x if x > 0 && x <= 10 => {
            buffer.write_all(&Variant::opt_list_tag(x))?;
        }
        _ => {
            buffer.write_all(&Variant::LIST_TAG)?;
            encode_varint(length as u64, buffer)?;
        }
    }
    for item in list.iter() {
        encode(py, item, buffer, depth, opts, types)?;
    }
    Ok(())
}

fn encode<W: Write>(
    py: Python<'_>,
    item: Bound<PyAny>,
    buffer: &mut W,
    depth: u32,
    opts: &mut Options,
    types: &PyTypesCmp,
) -> PyResult<()> {
    if opts.max_depth > 0 && depth > opts.max_depth {
        return Err(PyValueError::new_err("Depth exceeded maximum"));
    }
    let py_type = item.get_type();
    if py_type.is(&types.bool_) {
        let val: bool = item.extract()?;
        encode_bool(val, buffer)
    } else if py_type.is(&types.datetime_) {
        let val = item.cast_into::<PyDateTime>()?;
        encode_datetime(py, val, buffer, opts)
    } else if py_type.is(&types.int_) {
        let val: i64 = item.extract()?;
        if val == i64::MIN {
            return Err(PyOverflowError::new_err("[MIN] Integer is out of bounds"));
        }
        encode_int(val, buffer, opts)
    } else if py_type.is(&types.float_) {
        let val: f64 = item.extract()?;
        encode_float(val, buffer, opts)
    } else if py_type.is(&types.none_) {
        encode_none(buffer)
    } else if py_type.is(&types.string_) {
        let val: &str = item.extract()?;
        encode_string(val, buffer, opts)
    } else if py_type.is(&types.bytes_) {
        let val: &[u8] = item.extract()?;
        encode_bytes(val, buffer)
    } else if py_type.is(&types.list_) {
        let sub_list: &Bound<'_, PyList> = item.cast::<PyList>().unwrap();
        encode_list(py, sub_list, depth + 1, buffer, opts, types)
    } else if py_type.is(&types.tuple_) {
        let sub_list: &Bound<'_, PyTuple> = item.cast::<PyTuple>().unwrap();
        encode_tuple(py, sub_list, depth + 1, buffer, opts, types)
    } else if py_type.is(&types.set_) {
        let sub_list: &Bound<'_, PySet> = item.cast::<PySet>().unwrap();
        encode_set(py, sub_list, depth + 1, buffer, opts, types)
    } else if py_type.is(&types.dict_) {
        let sub_list: &Bound<'_, PyDict> = item.cast::<PyDict>().unwrap();
        encode_dict(py, sub_list, depth + 1, buffer, opts, types)
    } else {
        let name = py_type.name()?.to_string();
        let e_m = format!("Unsupported type-{}", name);
        Err(PyAttributeError::new_err(e_m))
    }
}

pub fn pack(
    py: Python<'_>,
    data: Bound<PyAny>,
    protocol_version: u8,
    max_depth: i32,
    string_length_limit: i32,
    float_limit: f64,
) -> PyResult<Vec<u8>> {
    let mut buffer = Vec::with_capacity(4096);
    buffer.push(protocol_version);
    let real_depth = if max_depth < 0 { 0 } else { max_depth as u32 };
    let real_float = if float_limit < 0.0 { 0.0 } else { float_limit };
    let real_string: usize = if string_length_limit < 0 {
        0
    } else {
        string_length_limit as usize
    };
    let types = PyTypesCmp::new(py);
    let mut opts = Options::new(real_depth, real_string, real_float);
    encode(py, data, &mut buffer, 1, &mut opts, &types)?;
    Ok(buffer)
}

pub fn pack_to_file(
    py: Python<'_>,
    py_file: Py<PyAny>,
    data: Bound<PyAny>,
    protocol_version: u8,
    max_depth: i32,
    string_length_limit: i32,
    float_limit: f64,
) -> PyResult<()> {
    let file_like = PyFileLikeObject::with_requirements(py_file, false, true, false, false)?;
    let mut buffered_writer = BufWriter::new(file_like);
    buffered_writer.write_all(&[protocol_version])?;
    let real_depth = if max_depth < 0 { 0 } else { max_depth as u32 };
    let real_float = if float_limit < 0.0 { 0.0 } else { float_limit };
    let real_string: usize = if string_length_limit < 0 {
        0
    } else {
        string_length_limit as usize
    };
    let mut opts = Options::new(real_depth, real_string, real_float);
    let types = PyTypesCmp::new(py);
    encode(py, data, &mut buffered_writer, 1, &mut opts, &types)?;
    buffered_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first() {
        let dec_places = 2;
        let pow = TEN.pow(dec_places as u32) as f64;
        let fl_limit = 268_435_455.0;
        let limit = fl_limit / pow;
        assert_eq!(limit, 2684354.55);
    }
}

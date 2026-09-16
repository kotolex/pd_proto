use crate::constants::*;
use crate::options::Options;
use crate::utils::{compress, dec_places};
use pyo3::exceptions::{PyAttributeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{
    PyAnyMethods, PyBytes, PyDateTime, PyDelta, PyDeltaAccess, PyNone, PySet, PyString,
    PyTzInfoAccess,
};
use pyo3::types::{PyBool, PyFloat, PyInt, PyList, PyTuple};
use pyo3::types::{PyDict, PyListMethods};
use pyo3_file::PyFileLikeObject;
use std::io::{BufWriter, Write};

pub fn encode_varint<W: Write>(mut number: u64, buffer: &mut W) {
    let mut buf = [0u8; 10];
    let mut idx = 0;
    while number >= 0x80 {
        buf[idx] = (number as u8 & 0x7F) | 0x80;
        number >>= 7;
        idx += 1;
    }
    buf[idx] = number as u8;
    idx += 1;
    let _ = buffer.write_all(&buf[..idx]);
}

fn encode_int<W: Write>(num: i64, buffer: &mut W, opts: &mut Options) {
    if num == 0 {
        let _ = buffer.write_all(&[Variant::IntZero as u8]);
        return;
    }
    if (1..=13).contains(&num) {
        let tag = INT_INDEX + num as usize; // cause INT_1=61 etc.
        let _ = buffer.write_all(&[tag as u8]);
        return;
    }
    match num {
        15 => {
            let _ = buffer.write_all(&[Variant::Int15 as u8]);
            return;
        }
        20 => {
            let _ = buffer.write_all(&[Variant::Int20 as u8]);
            return;
        }
        24 => {
            let _ = buffer.write_all(&[Variant::Int24 as u8]);
            return;
        }
        50 => {
            let _ = buffer.write_all(&[Variant::Int50 as u8]);
            return;
        }
        100 => {
            let _ = buffer.write_all(&[Variant::Int100 as u8]);
            return;
        }
        1000 => {
            let _ = buffer.write_all(&[Variant::Int1000 as u8]);
            return;
        }
        _ => (),
    }
    match opts.get_int_index(num) {
        Some(index) => {
            let _ = buffer.write_all(&[Variant::CacheInt as u8]);
            let _ = buffer.write_all(&[index]);
            return;
        }
        None => {
            opts.add_int(num);
        }
    }
    let tag = if num < 0 {
        Variant::IntNegative
    } else {
        Variant::IntPositive
    };
    let r_num: u64 = if num < 0 { -num as u64 } else { num as u64 };
    let _ = buffer.write_all(&[tag as u8]);
    encode_varint(r_num, buffer);
}

fn encode_none<W: Write>(buffer: &mut W) {
    let _ = buffer.write_all(&[Variant::Null as u8]);
}

fn encode_bool<W: Write>(value: bool, buffer: &mut W) {
    let _ = if value {
        buffer.write_all(&[Variant::BoolTrue as u8])
    } else {
        buffer.write_all(&[Variant::BoolFalse as u8])
    };
}

pub fn encode_float<W: Write>(number: f64, buffer: &mut W, opts: &mut Options) {
    if number == 0.0 {
        let _ = buffer.write_all(&[Variant::FloatZero as u8]);
        return;
    }
    match opts.get_float_index(number) {
        Some(index) => {
            let _ = buffer.write_all(&[Variant::CacheFloat as u8]);
            let _ = buffer.write_all(&[index]);
            return;
        }
        None => opts.add_float(number),
    }
    let r_number = if number < 0.0 { -number } else { number };
    if opts.float_limit > 0.0 && r_number <= opts.float_limit {
        let dec_places = dec_places(r_number);
        if dec_places < 7 {
            let tag = tag_by_decimal_places(dec_places, number < 0.0);
            if dec_places == 0 {
                let _ = buffer.write_all(&[tag]);
                encode_varint(r_number as u64, buffer);
                return;
            }
            let pow = TEN.pow(dec_places as u32) as f64;
            let limit = opts.float_limit / pow;
            if r_number < limit {
                let int_value = (r_number * pow).round() as u64;
                let _ = buffer.write_all(&[tag]);
                encode_varint(int_value, buffer);
                return;
            }
        }
    }
    let _ = buffer.write_all(&[Variant::Float as u8]);
    let _ = buffer.write_all(&number.to_be_bytes());
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
                let _ = buffer.write_all(&[Variant::DateTimeIana as u8]);
                encode_float(timestamp, buffer, opts);
                encode_string(&key_str, buffer, opts);
            }
            Err(_) => {
                let delta: Bound<PyDelta> = tz_info
                    .call_method1("utcoffset", (PyNone::get(py),))
                    .unwrap()
                    .extract()?;
                let dt_offset: i32 = delta.get_seconds();
                let _ = buffer.write_all(&[Variant::DateTimeOffset as u8]);
                encode_float(timestamp, buffer, opts);
                encode_int(dt_offset as i64, buffer, opts);
            }
        },
        None => {
            let _ = buffer.write_all(&[Variant::DateTimeNoTz as u8]);
            encode_float(timestamp, buffer, opts);
        }
    }
    Ok(())
}

fn encode_string<W: Write>(value: &str, buffer: &mut W, opts: &mut Options) {
    if value.is_empty() {
        let _ = buffer.write_all(&[Variant::StringEmpty as u8]);
        return;
    }
    let encoded = value.as_bytes();
    let bytes_len = encoded.len();
    match opts.get_string_index(encoded) {
        Some(index) => {
            let _ = buffer.write_all(&[Variant::CacheString as u8]);
            let _ = buffer.write_all(&[index]);
            return;
        }
        None => opts.add_string(encoded),
    }
    if bytes_len <= 15 {
        let tag = STRING_INDEX + bytes_len; // cause STRING_1=31 etc.
        let _ = buffer.write_all(&[tag as u8]);
        let _ = buffer.write_all(encoded);
        return;
    }
    if opts.string_length_limit > 0 && bytes_len > opts.string_length_limit {
        let compressed = compress(encoded).unwrap();
        if compressed.len() < bytes_len + 3 {
            let _ = buffer.write_all(&[Variant::StringCompressed as u8]);
            encode_varint(compressed.len() as u64, buffer);
            let _ = buffer.write_all(&compressed);
            return;
        }
    }
    let _ = buffer.write_all(&[Variant::String as u8]);
    encode_varint(bytes_len as u64, buffer);
    let _ = buffer.write_all(encoded);
}

fn encode_bytes<W: Write>(value: &mut [u8], buffer: &mut W) {
    if value.is_empty() {
        let _ = buffer.write_all(&[Variant::BytesEmpty as u8]);
    } else {
        let _ = buffer.write_all(&[Variant::Bytes as u8]);
        encode_varint(value.len() as u64, buffer);
        let _ = buffer.write_all(value);
    }
}

fn encode<W: Write>(
    py: Python<'_>,
    item: Bound<PyAny>,
    buffer: &mut W,
    depth: u32,
    opts: &mut Options,
) -> PyResult<()> {
    if opts.max_depth > 0 && depth > opts.max_depth {
        return Err(PyValueError::new_err("Depth exceeded maximum"));
    }
    let py_type = item.get_type();
    if py_type.is(py.get_type::<PyBool>()) {
        let val: bool = item.extract()?;
        encode_bool(val, buffer);
    } else if py_type.is(py.get_type::<PyDateTime>()) {
        let val = item.cast_into::<PyDateTime>()?;
        encode_datetime(py, val, buffer, opts)?;
    } else if py_type.is(py.get_type::<PyInt>()) {
        let val: i64 = item.extract()?;
        encode_int(val, buffer, opts);
    } else if py_type.is(py.get_type::<PyFloat>()) {
        let val: f64 = item.extract()?;
        encode_float(val, buffer, opts);
    } else if py_type.is(py.get_type::<PyNone>()) {
        encode_none(buffer);
    } else if py_type.is(py.get_type::<PyString>()) {
        let val: &str = item.extract()?;
        encode_string(val, buffer, opts);
    } else if py_type.is(py.get_type::<PyBytes>()) {
        let mut val: Vec<u8> = item.extract()?;
        encode_bytes(&mut val, buffer);
    } else if py_type.is(py.get_type::<PyList>()) {
        let sub_list: &Bound<'_, PyList> = item.cast::<PyList>().unwrap();
        encode_list(py, sub_list, depth + 1, buffer, opts)?;
    } else if py_type.is(py.get_type::<PyTuple>()) {
        let sub_list: &Bound<'_, PyTuple> = item.cast::<PyTuple>().unwrap();
        encode_tuple(py, sub_list, depth + 1, buffer, opts)?;
    } else if py_type.is(py.get_type::<PySet>()) {
        let sub_list: &Bound<'_, PySet> = item.cast::<PySet>().unwrap();
        encode_set(py, sub_list, depth + 1, buffer, opts)?;
    } else if py_type.is(py.get_type::<PyDict>()) {
        let sub_list: &Bound<'_, PyDict> = item.cast::<PyDict>().unwrap();
        encode_dict(py, sub_list, depth + 1, buffer, opts)?;
    } else {
        let name = py_type.name()?.to_string();
        let e_m = format!("Unsupported type-{}", name);
        return Err(PyAttributeError::new_err(e_m));
    }
    Ok(())
}

fn encode_dict<W: Write>(
    py: Python<'_>,
    list: &Bound<'_, PyDict>,
    depth: u32,
    buffer: &mut W,
    opts: &mut Options,
) -> PyResult<()> {
    if list.len() == 0 {
        let _ = buffer.write_all(&[Variant::DictEmpty as u8]);
        return Ok(());
    }
    let _ = buffer.write_all(&[Variant::Dict as u8]);
    encode_varint(list.len() as u64, buffer);
    for (key, value) in list.iter() {
        encode(py, key, buffer, depth, opts)?;
        encode(py, value, buffer, depth, opts)?;
    }
    Ok(())
}
fn encode_set<W: Write>(
    py: Python<'_>,
    a_set: &Bound<'_, PySet>,
    depth: u32,
    buffer: &mut W,
    opts: &mut Options,
) -> PyResult<()> {
    if a_set.len() == 0 {
        let _ = buffer.write_all(&[Variant::SetEmpty as u8]);
        return Ok(());
    }
    let _ = buffer.write_all(&[Variant::Set as u8]);
    encode_varint(a_set.len() as u64, buffer);
    for item in a_set.iter() {
        encode(py, item, buffer, depth, opts)?;
    }
    Ok(())
}

fn encode_tuple<W: Write>(
    py: Python<'_>,
    a_tuple: &Bound<'_, PyTuple>,
    depth: u32,
    buffer: &mut W,
    opts: &mut Options,
) -> PyResult<()> {
    let len = a_tuple.len();
    if len == 0 {
        let _ = buffer.write_all(&[Variant::TupleEmpty as u8]);
        return Ok(());
    }
    match len {
        2 => {
            let _ = buffer.write_all(&[Variant::Tuple2 as u8]);
        }
        3 => {
            let _ = buffer.write_all(&[Variant::Tuple3 as u8]);
        }
        4 => {
            let _ = buffer.write_all(&[Variant::Tuple4 as u8]);
        }
        5 => {
            let _ = buffer.write_all(&[Variant::Tuple5 as u8]);
        }
        _ => {
            let _ = buffer.write_all(&[Variant::Tuple as u8]);
            encode_varint(a_tuple.len() as u64, buffer);
        }
    }
    for item in a_tuple.iter() {
        encode(py, item, buffer, depth, opts)?;
    }
    Ok(())
}

fn encode_list<W: Write>(
    py: Python<'_>,
    list: &Bound<'_, PyList>,
    depth: u32,
    buffer: &mut W,
    opts: &mut Options,
) -> PyResult<()> {
    let len = list.len();
    if len == 0 {
        let _ = buffer.write_all(&[Variant::ListEmpty as u8]);
        return Ok(());
    }
    match len {
        x if x > 0 && x <= 10 => {
            let _ = buffer.write_all(&[(len + LIST_INDEX) as u8]);
        }
        _ => {
            let _ = buffer.write_all(&[Variant::List as u8]);
            encode_varint(len as u64, buffer);
        }
    }
    for item in list.iter() {
        encode(py, item, buffer, depth, opts)?;
    }
    Ok(())
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
    let mut opts = Options::new(real_depth, real_string, real_float);
    encode(py, data, &mut buffer, 1, &mut opts)?;
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
    encode(py, data, &mut buffered_writer, 1, &mut opts)?;
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

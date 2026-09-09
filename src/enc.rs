use crate::arc::compress;
use crate::constants::*;
use crate::pure::dec_places;
use pyo3::exceptions::{PyAttributeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{
    PyAnyMethods, PyBytes, PyDateTime, PyDelta, PyDeltaAccess, PyNone, PySet, PyString,
    PyTzInfoAccess,
};
use pyo3::types::{PyBool, PyFloat, PyInt, PyList, PyTuple};
use pyo3::types::{PyDict, PyListMethods};

struct Options {
    max_depth: u32,
    string_length_limit: usize,
    float_limit: f64,
}

pub fn var_int(mut number: u64, buffer: &mut Vec<u8>) {
    let mut buf = [0u8; 10];
    let mut idx = 0;
    while number > 0 {
        let mut byte = (number & 0x7F) as u8;
        number >>= 7;
        if number > 0 {
            byte |= 0x80;
        }
        buf[idx] = byte;
        idx += 1;
    }
    buffer.extend_from_slice(&buf[..idx]);
}

fn e_int(num: i64, buffer: &mut Vec<u8>) {
    if num == 0 {
        buffer.push(Variant::IntZero as u8);
        return;
    }
    if num >= 1 && num <= 13 {
        let tag = INT_INDEX + num as usize; // cause INT_1=61 etc.
        buffer.push(tag as u8);
        return;
    }
    match num {
        15 => {
            buffer.push(Variant::Int15 as u8);
            return;
        }
        20 => {
            buffer.push(Variant::Int20 as u8);
            return;
        }
        24 => {
            buffer.push(Variant::Int24 as u8);
            return;
        }
        50 => {
            buffer.push(Variant::Int50 as u8);
            return;
        }
        100 => {
            buffer.push(Variant::Int100 as u8);
            return;
        }
        1000 => {
            buffer.push(Variant::Int1000 as u8);
            return;
        }
        _ => (),
    }
    let tag = if num < 0 {
        Variant::IntNegative
    } else {
        Variant::IntPositive
    };
    let r_num: u64 = if num < 0 { -num as u64 } else { num as u64 };
    buffer.push(tag as u8);
    var_int(r_num, buffer);
}

fn e_none(buffer: &mut Vec<u8>) {
    buffer.push(Variant::Null as u8)
}

fn e_bool(value: bool, buffer: &mut Vec<u8>) {
    if value {
        buffer.push(Variant::BoolTrue as u8)
    } else {
        buffer.push(Variant::BoolFalse as u8)
    }
}

pub fn e_float(number: f64, buffer: &mut Vec<u8>, float_limit: f64) {
    if number == 0.0 {
        buffer.push(Variant::FloatZero as u8);
        return;
    }
    let r_number = if number < 0.0 { -number } else { number };
    if float_limit > 0.0 && r_number <= float_limit {
        let dec_places = dec_places(r_number);
        if dec_places < 7 {
            let tag = tag_by_decimal_places(dec_places, number < 0.0);
            if dec_places == 0 {
                buffer.push(tag);
                var_int(r_number as u64, buffer);
                return;
            }
            let pow = TEN.pow(dec_places as u32) as f64;
            let limit = float_limit / pow;
            if r_number < limit {
                let int_value = (r_number * pow).round() as u64;
                buffer.push(tag);
                var_int(int_value, buffer);
                return;
            }
        }
    }
    buffer.push(Variant::Float as u8);
    buffer.extend_from_slice(&number.to_be_bytes());
}

fn e_dt(py: Python<'_>, item: Bound<PyDateTime>, buffer: &mut Vec<u8>) -> PyResult<()> {
    let timestamp: f64 = item.call_method0("timestamp")?.extract()?;
    match item.get_tzinfo() {
        Some(tz_info) => match tz_info.getattr("key") {
            Ok(key) => {
                let key_str = key.to_string();
                buffer.push(Variant::DateTimeIana as u8);
                e_float(timestamp, buffer, FLOAT_DEFAULT_LIMIT);
                e_string(&key_str, buffer, 100);
            }
            Err(_) => {
                let delta: Bound<PyDelta> = tz_info
                    .call_method1("utcoffset", (PyNone::get(py),))
                    .unwrap()
                    .extract()?;
                let dt_offset: i32 = delta.get_seconds();
                buffer.push(Variant::DateTimeOffset as u8);
                e_float(timestamp, buffer, FLOAT_DEFAULT_LIMIT);
                e_int(dt_offset as i64, buffer);
            }
        },
        None => {
            buffer.push(Variant::DateTimeNoTz as u8);
            e_float(timestamp, buffer, FLOAT_DEFAULT_LIMIT);
        }
    }
    Ok(())
}

fn e_string(value: &str, buffer: &mut Vec<u8>, string_limit: usize) {
    if value.is_empty() {
        buffer.push(Variant::StringEmpty as u8);
        return;
    }
    let encoded = value.as_bytes();
    let bytes_len = encoded.len();
    if bytes_len <= 15 {
        let tag = STRING_INDEX + bytes_len; // cause STRING_1=31 etc.
        buffer.push(tag as u8);
        buffer.extend(encoded);
        return;
    }
    if string_limit > 0 && bytes_len > string_limit {
        let compressed = compress(encoded).unwrap();
        if compressed.len() < bytes_len + 3 {
            buffer.push(Variant::StringCompressed as u8);
            var_int(compressed.len() as u64, buffer);
            buffer.extend(compressed);
            return;
        }
    }
    buffer.push(Variant::String as u8);
    var_int(bytes_len as u64, buffer);
    buffer.extend(encoded);
}

fn e_bytes(value: Vec<u8>, buffer: &mut Vec<u8>) {
    if value.len() == 0 {
        buffer.push(Variant::BytesEmpty as u8)
    } else {
        buffer.push(Variant::Bytes as u8);
        var_int(value.len() as u64, buffer);
        buffer.extend(value);
    }
}

fn _parse_item(
    py: Python<'_>,
    item: Bound<PyAny>,
    buffer: &mut Vec<u8>,
    depth: u32,
    opts: &Options,
) -> PyResult<()> {
    if opts.max_depth > 0 && depth > opts.max_depth {
        return Err(PyValueError::new_err("Depth exceeded maximum"));
    }
    let py_type = item.get_type();
    if py_type.is(&py.get_type::<PyBool>()) {
        let val: bool = item.extract()?;
        e_bool(val, buffer);
    } else if py_type.is(&py.get_type::<PyDateTime>()) {
        let val = item.cast_into::<PyDateTime>()?;
        e_dt(py, val, buffer)?;
    } else if py_type.is(&py.get_type::<PyInt>()) {
        let val: i64 = item.extract()?;
        e_int(val, buffer);
    } else if py_type.is(&py.get_type::<PyFloat>()) {
        let val: f64 = item.extract()?;
        e_float(val, buffer, opts.float_limit);
    } else if py_type.is(&py.get_type::<PyNone>()) {
        e_none(buffer);
    } else if py_type.is(&py.get_type::<PyString>()) {
        let val: &str = item.extract()?;
        e_string(val, buffer, opts.string_length_limit);
    } else if py_type.is(&py.get_type::<PyBytes>()) {
        let val: Vec<u8> = item.extract()?;
        e_bytes(val, buffer);
    } else if py_type.is(&py.get_type::<PyList>()) {
        let sub_list: &Bound<'_, PyList> = item.cast::<PyList>().unwrap();
        e_list(py, &sub_list, depth + 1, buffer, opts)?;
    } else if py_type.is(&py.get_type::<PyTuple>()) {
        let sub_list: &Bound<'_, PyTuple> = item.cast::<PyTuple>().unwrap();
        e_tuple(py, &sub_list, depth + 1, buffer, opts)?;
    } else if py_type.is(&py.get_type::<PySet>()) {
        let sub_list: &Bound<'_, PySet> = item.cast::<PySet>().unwrap();
        e_set(py, &sub_list, depth + 1, buffer, opts)?;
    } else if py_type.is(&py.get_type::<PyDict>()) {
        let sub_list: &Bound<'_, PyDict> = item.cast::<PyDict>().unwrap();
        e_dict(py, &sub_list, depth + 1, buffer, opts)?;
    } else {
        let name = py_type.name()?.to_string();
        let e_m = format!("Unsupported type-{}", name);
        return Err(PyAttributeError::new_err(e_m));
    }
    Ok(())
}

fn e_dict(
    py: Python<'_>,
    list: &Bound<'_, PyDict>,
    depth: u32,
    buffer: &mut Vec<u8>,
    opts: &Options,
) -> PyResult<()> {
    if list.len() == 0 {
        buffer.push(Variant::DictEmpty as u8);
        return Ok(());
    }
    buffer.push(Variant::Dict as u8);
    var_int(list.len() as u64, buffer);
    for (key, value) in list.iter() {
        _parse_item(py, key, buffer, depth, opts)?;
        _parse_item(py, value, buffer, depth, opts)?;
    }
    Ok(())
}
fn e_set(
    py: Python<'_>,
    list: &Bound<'_, PySet>,
    depth: u32,
    buffer: &mut Vec<u8>,
    opts: &Options,
) -> PyResult<()> {
    if list.len() == 0 {
        buffer.push(Variant::SetEmpty as u8);
        return Ok(());
    }
    buffer.push(Variant::Set as u8);
    var_int(list.len() as u64, buffer);
    for item in list.iter() {
        _parse_item(py, item, buffer, depth, opts)?;
    }
    Ok(())
}

fn e_tuple(
    py: Python<'_>,
    a_tuple: &Bound<'_, PyTuple>,
    depth: u32,
    buffer: &mut Vec<u8>,
    opts: &Options,
) -> PyResult<()> {
    let len = a_tuple.len();
    if len == 0 {
        buffer.push(Variant::TupleEmpty as u8);
        return Ok(());
    }
    match len {
        2 => buffer.push(Variant::Tuple2 as u8),
        3 => buffer.push(Variant::Tuple3 as u8),
        4 => buffer.push(Variant::Tuple4 as u8),
        5 => buffer.push(Variant::Tuple5 as u8),
        _ => {
            buffer.push(Variant::Tuple as u8);
            var_int(a_tuple.len() as u64, buffer);
        }
    }
    for item in a_tuple.iter() {
        _parse_item(py, item, buffer, depth, opts)?;
    }
    Ok(())
}

fn e_list(
    py: Python<'_>,
    list: &Bound<'_, PyList>,
    depth: u32,
    buffer: &mut Vec<u8>,
    opts: &Options,
) -> PyResult<()> {
    let len = list.len();
    if len == 0 {
        buffer.push(Variant::ListEmpty as u8);
        return Ok(());
    }
    match len {
        x if x > 0 && x <= 10 => buffer.push((len + LIST_INDEX) as u8),
        _ => {
            buffer.push(Variant::List as u8);
            var_int(len as u64, buffer);
        }
    }
    for item in list.iter() {
        _parse_item(py, item, buffer, depth, opts)?;
    }
    Ok(())
}

pub fn enc(
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
    let opts = Options {
        max_depth: real_depth,
        string_length_limit: real_string,
        float_limit: real_float,
    };
    _parse_item(py, data, &mut buffer, 1, &opts)?;
    Ok(buffer)
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

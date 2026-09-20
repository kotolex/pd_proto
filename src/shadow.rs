use crate::constants::{FLOAT_BYTES, LIST_INDEX, TEN, Variant};
use crate::dec::{EMPTY_DICT, EMPTY_VEC, ParsedData, decode_optimized_int};
use crate::options::DecodeOptions;
use crate::utils::decompress;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::io::Write;

fn get_tabs(current_depth:u32) -> String {
    let tab_count= if current_depth > 10 {10} else {current_depth - 1};
    "\t".repeat(tab_count as usize)
}

pub fn decode_varint<W: Write>(bts: &[u8], mut offset: usize, result: &mut W, current_depth: u32) -> PyResult<(u64, usize)> {
    let tabs = get_tabs(current_depth);
    let e_m = format!("{}Parsing of a var_int started at offset {}\n", tabs, offset);
    result.write_all(e_m.as_bytes())?;
    let start = offset;
    let mut number: u64 = 0;
    let mut shift = 0;
    loop {
        let e_m = format!("{}Var_int parsing, offset {}\n", tabs, offset);
        result.write_all(e_m.as_bytes())?;
        let &byte = bts.get(offset).ok_or_else(|| {
            PyValueError::new_err(format!("While parsing var_int - no data to read at offset {}\n", offset))
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
                "While parsing var_int - int is too long or data corrupted at offset: {}\n",
                offset
            )));
        }
    }
    let e_m = format!("{}Var_int parsed {}, {} bytes read\n", tabs, number, offset-start);
    result.write_all(e_m.as_bytes())?;
    Ok((number, offset - start))
}

fn decode_float<W: Write>(buffer: &[u8], offset: usize, result: &mut W, current_depth: u32) -> PyResult<(f64, usize)> {
    let tabs = get_tabs(current_depth);
    let e_m = format!("{}Parsing of an 8-byte float started at offset {}\n", tabs, offset);
    result.write_all(e_m.as_bytes())?;
    if buffer.len() < offset + FLOAT_BYTES {
        let e_m = format!(
            "Not enough bytes, need {}, but have only {} bytes left at offset {}\n",
            FLOAT_BYTES,
            buffer.len() - offset,
            offset
        );
        return Err(PyValueError::new_err(e_m));
    }
    match buffer[offset..offset + FLOAT_BYTES].try_into() {
        Ok(x) => {
            let res = f64::from_be_bytes(x);
            let e_m = format!("{}An 8-byte float ({}) parsed; 8 bytes read\n", tabs, res);
            result.write_all(e_m.as_bytes())?;
            Ok((res, FLOAT_BYTES)) },
        Err(e) => {
            let e_m = format!("Wrong data for float, error: {}\n", e);
            Err(PyValueError::new_err(e_m)) },
    }
}

fn decode_optimized_float<W: Write>(buffer: &[u8], offset: usize, tag: Variant, result: &mut W, current_depth: u32) -> PyResult<(f64, usize)> {
    let tabs = get_tabs(current_depth);
    let e_m = format!("{}Parsing of an optimized float started; expecting an int at offset {}\n", tabs, offset);
    result.write_all(e_m.as_bytes())?;
    let (value, read) = decode_varint(buffer, offset, result, current_depth)?;
    if tag == Variant::FloatNoDecimals {
        let e_m = format!("{}Positive float without decimals parsed: {}\n", tabs, value);
        result.write_all(e_m.as_bytes())?;
        return Ok((value as f64, read));
    } else if tag == Variant::FloatNoDecimalsNeg {
        let e_m = format!("{}Negative float without decimals parsed: {}\n", tabs,  -(value as f64));
        result.write_all(e_m.as_bytes())?;
        return Ok((-(value as f64), read));
    }
    if tag > Variant::FloatNoDecimals && tag <= Variant::Float6 {
        let dec_places = tag as u8 - 20; // cause FLOAT_1 = 21, FLOAT_2=22 etc.
        let pow = TEN.pow(dec_places as u32) as f64;
        let res = value as f64 / pow;
        let e_m = format!("{}Positive float parsed: {}\n", tabs, res);
        result.write_all(e_m.as_bytes())?;
        Ok((res, read))
    } else {
        let dec_places = tag as u8 - 30; // cause FLOAT_1_NEG = 31, FLOAT_2_NEG=32 etc.
        let pow = TEN.pow(dec_places as u32) as f64;
        let res = value as f64 / pow;
        let e_m = format!("{}Negative float parsed: {}\n", tabs, res);
        result.write_all(e_m.as_bytes())?;
        Ok((-res, read))
    }
}

fn decode_string<W: Write>(buffer: &[u8], offset: usize, tag: Variant, result: &mut W, current_depth: u32) -> PyResult<(String, usize)> {
    let tabs = get_tabs(current_depth);
    let e_m = format!("{}Parsing of a string started at offset {}\n", tabs, offset);
    result.write_all(e_m.as_bytes())?;
    let last_index;
    let mut new_offset = offset;
    if tag == Variant::String || tag == Variant::StringCompressed {
        let e_m = format!("{}Expected integer (string size) at offset {}\n", tabs, offset);
        result.write_all(e_m.as_bytes())?;
        let (value, read) = decode_varint(buffer, offset, result, current_depth)?;
        last_index = read + (value as usize) + offset;
        let e_m = format!("{}Expected string size: {}\n", tabs, value);
        result.write_all(e_m.as_bytes())?;
        new_offset = offset + read;
        if buffer.len() < last_index {
            let e_m = format!(
                "Not enough bytes, need {}, but have only {} bytes left at offset {}\n",
                value,
                buffer.len() - new_offset,
                new_offset
            );
            return Err(PyValueError::new_err(e_m));
        }
    } else {
        last_index = (tag as u8 - 40) as usize + offset; // cause STRING_1=41 etc.
        let e_m = format!("{}It is an optimized string, no need to parse its size; expected string size is {}\n", tabs, (tag as u8 - 40));
        result.write_all(e_m.as_bytes())?;
        if buffer.len() < last_index {
            let e_m = format!(
                "Not enough bytes, need {}, but have only {} bytes left\n",
                (tag as u8 - 40),
                buffer.len() - offset
            );
            return Err(PyValueError::new_err(e_m));
        }
    }
    let sub = &buffer[new_offset..last_index];
    let text = if tag == Variant::StringCompressed {
        let e_m = format!("{}It is compressed string, so decompress it...\n", tabs);
        result.write_all(e_m.as_bytes())?;
        decompress(sub)?
    } else {
        Vec::from(sub)
    };
    let string = String::from_utf8(text)?;
    Ok((string, last_index))
}

fn decode_list<W: Write>(
    buffer: &[u8],
    offset: usize,
    tag: Variant,
    opts: &mut DecodeOptions,
    current_depth: u32,
    res: &mut W,
    is_list: bool,
) -> PyResult<(Vec<ParsedData>, usize)> {
    let name = if is_list {"List"} else {"Set"};
    let tabs = get_tabs(current_depth);
    let (elements_count, read) = if tag >= Variant::List1 && tag <= Variant::List10 {
        let elements_count: u64 = tag as u64 - LIST_INDEX as u64;
        let e_m = format!("{}It is an optimized list, no need to parse its size\n", tabs);
        res.write_all(e_m.as_bytes())?;
        (elements_count, 0)
    } else {
        let e_m = format!("{}Expect {} size, parse int at {}\n", tabs, name, offset);
        res.write_all(e_m.as_bytes())?;
        decode_varint(buffer, offset, res, current_depth)?
    };
    let e_m = format!("{}{} expects {} elements\n", tabs, name, elements_count);
    res.write_all(e_m.as_bytes())?;
    let mut result = Vec::with_capacity(elements_count as usize);
    let mut new_offset = offset + read;
    for i in 0..elements_count {
        let e_m = format!("{}Processed {} element {}\n", tabs, name, i);
        res.write_all(e_m.as_bytes())?;
        let (el, off) = decode(buffer, new_offset, opts, current_depth, res)?;
        result.push(el);
        new_offset = off
    }
    let e_m = format!("{}{} with {} elements fully parsed\n", tabs, name, elements_count);
    res.write_all(e_m.as_bytes())?;
    Ok((result, new_offset))
}

fn decode_tuple<W: Write>(
    buffer: &[u8],
    offset: usize,
    tag: Variant,
    opts: &mut DecodeOptions,
    current_depth: u32,
    res: &mut W
) -> PyResult<(Vec<ParsedData>, usize)> {
    let tabs = get_tabs(current_depth);
    let (elements_count, read) = if tag >= Variant::Tuple2 && tag <= Variant::Tuple5 {
        let elements_count: u64 = match tag {
            Variant::Tuple2 => 2,
            Variant::Tuple3 => 3,
            Variant::Tuple4 => 4,
            _ => 5,
        };
        let e_m = format!("{}It is optimized tuple, no need to parse its size\n", tabs);
        res.write_all(e_m.as_bytes())?;
        (elements_count, 0)
    } else {
        let e_m = format!("{}Expect tuple size, parse int at {}\n", tabs, offset);
        res.write_all(e_m.as_bytes())?;
        decode_varint(buffer, offset, res, current_depth)?
    };
    let e_m = format!("{}Tuple expects {} elements\n", tabs, elements_count);
    res.write_all(e_m.as_bytes())?;
    let mut result = Vec::with_capacity(elements_count as usize);
    let mut new_offset = offset + read;
    for i in 0..elements_count {
        let e_m = format!("{}Processed tuple element {}\n", tabs, i);
        res.write_all(e_m.as_bytes())?;
        let (el, off) = decode(buffer, new_offset, opts, current_depth, res)?;
        result.push(el);
        new_offset = off
    }
    let e_m = format!("{}Tuple with {} elements fully parsed\n", tabs, elements_count);
    res.write_all(e_m.as_bytes())?;
    Ok((result, new_offset))
}

fn decode_set<W: Write>(
    buffer: &[u8],
    offset: usize,
    tag: Variant,
    opts: &mut DecodeOptions,
    current_depth: u32,
    res: &mut W
) -> PyResult<(Vec<ParsedData>, usize)> {
    let (result, new_offset) = decode_list(buffer, offset, tag, opts, current_depth, res, false)?;
    Ok((result, new_offset))
}

fn decode_dict<W: Write>(
    buffer: &[u8],
    offset: usize,
    opts: &mut DecodeOptions,
    current_depth: u32,
    res: &mut W
) -> PyResult<(Vec<(ParsedData, ParsedData)>, usize)> {
    let tabs = get_tabs(current_depth);
    let e_m = format!("{}Expect dict size, parse int at {}\n", tabs, offset);
    res.write_all(e_m.as_bytes())?;
    let (elements_count, read) = decode_varint(buffer, offset, res, current_depth)?;
    let e_m = format!("{}Dict expects {} elements(pairs)\n", tabs, elements_count);
    res.write_all(e_m.as_bytes())?;
    let mut result: Vec<(ParsedData, ParsedData)> = Vec::with_capacity(elements_count as usize);
    let mut new_offset = offset + read;
    for i in 0..elements_count {
        let e_m = format!("{}Start parsing {} element (pair) for dict\n", tabs, i);
        res.write_all(e_m.as_bytes())?;
        let (key, off) = decode(buffer, new_offset, opts, current_depth, res)?;
        new_offset = off;
        let (value, off) = decode(buffer, new_offset, opts, current_depth, res)?;
        result.push((key, value));
        new_offset = off;
    }
    let e_m = format!("{}Dict with {} elements (pairs) fully parsed\n", tabs, elements_count);
    res.write_all(e_m.as_bytes())?;
    Ok((result, new_offset))
}

fn decode_bytes<W: Write>(buffer: &[u8], offset: usize, res: &mut W, current_depth: u32) -> PyResult<(Vec<u8>, usize)> {
    let tabs = get_tabs(current_depth);
    let e_m = format!("{}Expect bytes size, parse int at {}\n", tabs, offset);
    res.write_all(e_m.as_bytes())?;
    let (size, read) = decode_varint(buffer, offset, res, current_depth)?;
    let e_m = format!("{}Bytes block expect size {}\n", tabs, size);
    res.write_all(e_m.as_bytes())?;
    let new_offset = offset + read;
    let last_index = new_offset + size as usize;
    if buffer.len() < last_index {
        let e_m = format!(
            "Not enough bytes, need {}, but have only {} bytes left at offset {}\n",
            last_index - new_offset,
            buffer.len() - new_offset,
            new_offset
        );
        return Err(PyValueError::new_err(e_m));
    }
    let data = buffer[new_offset..new_offset + size as usize].to_vec();
    Ok((data, new_offset + size as usize))
}

fn parse_float<W: Write>(
    buffer: &[u8],
    offset: usize,
    opts: &mut DecodeOptions,
    current_depth: u32,
    res: &mut W
) -> PyResult<(f64, usize)> {
    let (pd, new_offset) = decode(buffer, offset, opts, current_depth, res)?;
    match pd {
        ParsedData::Float(v) => Ok((v, new_offset)),
        _ => Err(PyValueError::new_err(
            "Unexpected type while parsing DateTime, expected Float\n",
        )),
    }
}

fn decode_cached_int<W: Write>(
    buffer: &[u8],
    offset: usize,
    opts: &mut DecodeOptions,
    res: &mut W,
    current_depth: u32,
) -> PyResult<(ParsedData, usize)> {
    let tabs = get_tabs(current_depth);
    let e_m = format!("{}Integer cache tag found; expecting a 1-byte cache index at offset {}\n", tabs, offset);
    res.write_all(e_m.as_bytes())?;
    if let Some(&index) = buffer.get(offset) {
        let e_m = format!("{}Integer cache requested index {}\n", tabs, index);
        res.write_all(e_m.as_bytes())?;
        match opts.get_int(index) {
            Some(value) => {
                let e_m = format!("{}Retrieved from integer cache: index={}, value={}\n", tabs, index, value);
                res.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Int(*value), offset + 1)) },
            None => {
                let e_m = format!(
                    "Unexpected integer cache failure - no data at index {}\n",
                    index
                );
                Err(PyValueError::new_err(e_m))
            }
        }
    } else {
        let e_m = format!("Expect integer cache index, but nothing to read at offset {}\n", offset);
        Err(PyValueError::new_err(e_m))
    }
}

fn decode_cached_float<W: Write>(
    buffer: &[u8],
    offset: usize,
    opts: &mut DecodeOptions,
    res: &mut W,
    current_depth: u32,
) -> PyResult<(ParsedData, usize)> {
    let tabs = get_tabs(current_depth);
    let e_m = format!("{}Float cache tag found; expecting a 1-byte cache index at offset {}\n", tabs, offset);
    res.write_all(e_m.as_bytes())?;
    if let Some(&index) = buffer.get(offset) {
        let e_m = format!("{}Float cache requested index {}\n", tabs, index);
        res.write_all(e_m.as_bytes())?;
        match opts.get_float(index) {
            Some(value) => {
                let e_m = format!("{}Retrieved from float cache: index={}, value={}\n", tabs, index, value);
                res.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Float(*value), offset + 1)) },
            None => {
                let e_m = format!(
                    "Unexpected float cache failure - no data at index {}\n",
                    index
                );
                Err(PyValueError::new_err(e_m))
            }
        }
    } else {
        let e_m = format!("Expect float cache index, but nothing to read at offset {}\n", offset);
        Err(PyValueError::new_err(e_m))
    }
}

fn decode_cached_string<W: Write>(
    buffer: &[u8],
    offset: usize,
    opts: &mut DecodeOptions,
    res: &mut W,
    current_depth: u32,
) -> PyResult<(ParsedData, usize)> {
    let tabs = get_tabs(current_depth);
    let e_m = format!("{}String cache tag found; expecting a 1-byte cache index at offset {}\n", tabs, offset);
    res.write_all(e_m.as_bytes())?;
    if let Some(&index) = buffer.get(offset) {
        let e_m = format!("{}String cache requested index {}\n", tabs, index);
        res.write_all(e_m.as_bytes())?;
        match opts.get_string(index) {
            Some(value) => {
                let e_m = format!("{}Retrieved from string cache: index={}, value='{}'\n", tabs, index, value);
                res.write_all(e_m.as_bytes())?;
                Ok((ParsedData::String((*value).parse()?), offset + 1)) },
            None => {
                let e_m = format!(
                    "Unexpected string cache failure - no data at index {}\n",
                    index
                );
                Err(PyValueError::new_err(e_m))
            }
        }
    } else {
        let e_m = format!("No string cache index at offset {}\n", offset);
        Err(PyValueError::new_err(e_m))
    }
}

fn decode<W: Write>(
    buffer: &[u8],
    offset: usize,
    opts: &mut DecodeOptions,
    current_depth: u32,
    result: &mut W,
) -> PyResult<(ParsedData, usize)> {
    if opts.max_depth > 0 && current_depth > opts.max_depth {
        let e_m = format!(
            "Nesting depth {} exceeded maximum of {} at offset {}\n",
            current_depth,  opts.max_depth, offset
        );
        result.write_all(e_m.as_bytes())?;
        return Err(PyValueError::new_err(e_m));
    }
    if let Some(&tag) = buffer.get(offset) {
        let e_m = format!(
            "-------------- [Offset {}] [Tag {} / {:#X?}] [Nesting level {}]---------------\n", offset, tag, tag, current_depth);
        result.write_all(e_m.as_bytes())?;
        let new_offset = offset + 1;
        let tabs = get_tabs(current_depth);
        match Variant::try_from(tag) {
            Ok(Variant::Null) => {
                let e_m = format!("{}None object parsed.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Null, new_offset)) },
            Ok(Variant::BoolTrue) => {
                let e_m = format!("{}Boolean True parsed.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::BoolTrue, new_offset))
            },
            Ok(Variant::BoolFalse) => {
                let e_m = format!("{}Boolean False parsed.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::BoolFalse, new_offset))
            },
            Ok(Variant::FloatZero) => {
                let e_m = format!("{}Float zero 0.0 parsed.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Float(0.0), new_offset))
            },
            Ok(Variant::StringEmpty) => {
                let e_m = format!("{}Empty string '' parsed.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::String("".to_string()), new_offset))
            },
            Ok(Variant::IntZero) => {
                let e_m = format!("{}Integer zero 0 parsed.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Int(0), new_offset))
            },
            Ok(Variant::ListEmpty) => {
                let e_m = format!("{}Empty list [] parsed.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::List(EMPTY_VEC), new_offset))
            },
            Ok(Variant::TupleEmpty) => {
                let e_m = format!("{}Empty tuple (,) parsed.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Tuple(EMPTY_VEC), new_offset))
            },
            Ok(Variant::SetEmpty) => {
                let e_m = format!("{}Empty set parsed.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Set(EMPTY_VEC), new_offset))
            },
            Ok(Variant::DictEmpty) => {
                let e_m = format!("{}Empty dict parsed.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Dict(EMPTY_DICT), new_offset))
            },
            Ok(Variant::BytesEmpty) => {
                let e_m = format!("{}Empty bytes b'' parsed.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::BytesEmpty, new_offset))
            },
            Ok(Variant::CacheInt) => decode_cached_int(buffer, new_offset, opts, result, current_depth),
            Ok(Variant::CacheFloat) => decode_cached_float(buffer, new_offset, opts, result, current_depth),
            Ok(Variant::CacheString) => decode_cached_string(buffer, new_offset, opts, result, current_depth),
            Ok(Variant::DateTimeNoTz) => {
                let e_m = format!("{}Datetime without timezone tag found. A float (timestamp) is now expected.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (f, new_offset) = parse_float(buffer, new_offset, opts, current_depth, result)?;
                let e_m = format!("{}Datetime without timezone (timestamp={}) parsed\n", tabs, f);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::DateTimeNoTz(f), new_offset))
            }
            Ok(Variant::DateTimeOffset) => {
                let e_m = format!("{}Datetime with offset tag found. A float (timestamp) is now expected.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (f, new_offset) = parse_float(buffer, new_offset, opts, current_depth, result)?;
                let e_m = format!("{}Timestamp={} parsed. An integer (offset) is now expected.\n", tabs, f);
                result.write_all(e_m.as_bytes())?;
                let (pd, new_offset) = decode(buffer, new_offset, opts, current_depth, result)?;
                match pd {
                    ParsedData::Int(v) => {
                        let e_m = format!("{}Datetime with offset (timestamp={}, offset={}) parsed\n", tabs, f, v);
                        result.write_all(e_m.as_bytes())?;
                        Ok((ParsedData::DateTimeOffset((f, v)), new_offset)) },
                    _ => Err(PyValueError::new_err(
                        "Unexpected type while parsing DateTime, expected Int\n",
                    )),
                }
            }
            Ok(Variant::DateTimeIana) => {
                let e_m = format!("{}Datetime with IANA tag found. A float (timestamp) is now expected.\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (f, new_offset) = parse_float(buffer, new_offset, opts, current_depth, result)?;
                let e_m = format!("{}Timestamp={} parsed. A string (IANA) is now expected.\n", tabs, f);
                result.write_all(e_m.as_bytes())?;
                let (pd, new_offset) = decode(buffer, new_offset, opts, current_depth, result)?;
                match pd {
                    ParsedData::String(v) => {
                        let e_m = format!("{}Datetime with IANA (timestamp={}, IANA='{}') parsed\n", tabs, f, v);
                        result.write_all(e_m.as_bytes())?;
                        Ok((ParsedData::DateTimeIana((f, v)), new_offset)) },
                    _ => Err(PyValueError::new_err(
                        "Unexpected type while parsing DateTimeIana, expected String\n",
                    )),
                }
            }
            Ok(Variant::Float) => {
                let e_m = format!("{}Float tag found, attempting to parse it\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (value, off) = decode_float(buffer, new_offset, result, current_depth)?;
                let e_m = format!("{}Float={} parsed, pushing to cache\n", tabs, value);
                result.write_all(e_m.as_bytes())?;
                opts.add_float(value);
                Ok((ParsedData::Float(value), new_offset + off))
            }
            Ok(i) if i >= Variant::Int1000 && i <= Variant::Int100 => {
                let e_m = format!("{}Optimized int tag found, attempting to parse it\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let value = decode_optimized_int(i);
                let e_m = format!("{}Integer={} parsed\n", tabs, value);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Int(value), new_offset))
            }
            Ok(t)
            if (t >= Variant::FloatNoDecimals && t <= Variant::Float6)
                || (t >= Variant::FloatNoDecimalsNeg && t <= Variant::Float6Neg) =>
                {
                    let e_m = format!("{}Optimized float tag found, attempting to parse it\n", tabs);
                    result.write_all(e_m.as_bytes())?;
                    let (value, off) = decode_optimized_float(buffer, new_offset, t, result, current_depth)?;
                    let e_m = format!("{}Float={} parsed, pushing to cache\n", tabs, value);
                    result.write_all(e_m.as_bytes())?;
                    opts.add_float(value);
                    Ok((ParsedData::Float(value), new_offset + off))
                }
            Ok(Variant::IntPositive) => {
                let e_m = format!("{}Positive integer tag found, attempting to parse it\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (value, off) = decode_varint(buffer, new_offset, result, current_depth)?;
                let e_m = format!("{}Integer={} parsed, pushing to cache\n", tabs, value);
                result.write_all(e_m.as_bytes())?;
                opts.add_int(value as i64);
                Ok((ParsedData::Int(value as i64), new_offset + off))
            }
            Ok(Variant::IntNegative) => {
                let e_m = format!("{}Negative integer tag found, attempting to parse it\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (value, off) = decode_varint(buffer, new_offset, result, current_depth)?;
                let e_m = format!("{}Integer={} parsed, pushing to cache\n", tabs, value);
                result.write_all(e_m.as_bytes())?;
                opts.add_int(-(value as i64));
                Ok((ParsedData::Int(-(value as i64)), new_offset + off))
            }
            Ok(t) if (t >= Variant::StringCompressed && t <= Variant::String15) => {
                let e_m = format!("{}String tag found, attempting to parse it\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_string(buffer, new_offset, t, result, current_depth)?;
                let e_m = format!("{}String with len={} parsed, pushing to cache\n", tabs, value.len());
                result.write_all(e_m.as_bytes())?;
                opts.add_string(&value);
                Ok((ParsedData::String(value), offset))
            }
            Ok(Variant::String) => {
                let e_m = format!("{}String tag found, attempting to parse it\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_string(buffer, new_offset, Variant::String, result, current_depth)?;
                let e_m = format!("{}String with len={} parsed, pushing to cache\n", tabs, value.len());
                result.write_all(e_m.as_bytes())?;
                opts.add_string(&value);
                Ok((ParsedData::String(value), offset))
            }
            Ok(t) if t == Variant::List || (t >= Variant::List1 && t <= Variant::List10) => {
                let e_m = format!("{}List tag found, attempting to parse it\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_list(buffer, new_offset, t, opts, current_depth + 1, result, true)?;
                Ok((ParsedData::List(value), offset))
            }
            Ok(t) if t == Variant::Tuple || (t >= Variant::Tuple2 && t <= Variant::Tuple5) => {
                let e_m = format!("{}Tuple tag found, attempting to parse it\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_tuple(buffer, new_offset, t, opts, current_depth + 1, result)?;
                Ok((ParsedData::Tuple(value), offset))
            }
            Ok(s) if s == Variant::Set => {
                let e_m = format!("{}Set tag found, attempting to parse it\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_set(buffer, new_offset, s, opts, current_depth + 1, result)?;
                Ok((ParsedData::Set(value), offset))
            }
            Ok(Variant::Dict) => {
                let e_m = format!("{}Dict tag found, attempting to parse it\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_dict(buffer, new_offset, opts, current_depth + 1, result)?;
                Ok((ParsedData::Dict(value), offset))
            }
            Ok(Variant::Bytes) => {
                let e_m = format!("{}Bytes tag found, attempting to parse it\n", tabs);
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_bytes(buffer, new_offset, result, current_depth)?;
                let e_m = format!("{}Bytes with len={} parsed\n", tabs, value.len());
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Bytes(value), offset))
            }
            _ => {
                let e_m = format!("Unknown tag {}\n", tag);
                result.write_all(e_m.as_bytes())?;
                Err(PyValueError::new_err(e_m))
            }
        }
    } else {
        let e_m = format!("No data at offset {}\n", offset);
        result.write_all(e_m.as_bytes())?;
        Err(PyValueError::new_err(e_m))
    }
}

pub fn explains(
    buffer: Vec<u8>,
    offset: usize,
    max_depth: i32,
) -> PyResult<String> {
    let real_depth = if max_depth < 0 { 0 } else { max_depth as u32 };
    let mut opts = DecodeOptions::new(real_depth);
    let mut result: Vec<u8> = Vec::new();
    let length = buffer.len();
    let e_m = format!("Parsing started at offset {}, total length {}\n", offset, length);
    result.write_all(e_m.as_bytes())?;
    match decode(&buffer, offset, &mut opts, 1, &mut result){
        Ok((_, s))=> {
            let e_m = format!("Parsing stopped at offset {}\n", s);
            result.write_all(e_m.as_bytes())?;
            if s < length {
                let e_m = format!("[ERROR] Corrupt data, finished at offset {}, but still have {} bytes unparsed, total data length {}\n", s, length-s, length);
                result.write_all(e_m.as_bytes())?;
            }
        }
        Err(e) => {
                if let Some((_, message)) = e.to_string().split_once(':') {
                    let e_m = format!("[ERROR] Parsing stopped: {}\n", message.trim());
                    result.write_all(e_m.as_bytes())?;
                }
                else {
                    let e_m = format!("[ERROR] Parsing stopped on {}\n", e);
                    result.write_all(e_m.as_bytes())?;
                }
        }
    }
    Ok(String::from_utf8(result)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none() {
        let b:Vec<u8> = vec![1, 0];
        let expected = "Parsing started at offset 1, total length 2\n-------------- [Offset 1] [Tag 0 / 0x0] [Nesting level 1]---------------\nNone object parsed.\nParsing stopped at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_true() {
        let b:Vec<u8> = vec![1, 1];
        let expected = "Parsing started at offset 1, total length 2\n-------------- [Offset 1] [Tag 1 / 0x1] [Nesting level 1]---------------\nBoolean True parsed.\nParsing stopped at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_false() {
        let b:Vec<u8> = vec![1, 2];
        let expected = "Parsing started at offset 1, total length 2\n-------------- [Offset 1] [Tag 2 / 0x2] [Nesting level 1]---------------\nBoolean False parsed.\nParsing stopped at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_float_zero() {
        let b:Vec<u8> = vec![1, 3];
        let expected = "Parsing started at offset 1, total length 2\n-------------- [Offset 1] [Tag 3 / 0x3] [Nesting level 1]---------------\nFloat zero 0.0 parsed.\nParsing stopped at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_string_empty() {
        let b:Vec<u8> = vec![1, 4];
        let expected = "Parsing started at offset 1, total length 2\n-------------- [Offset 1] [Tag 4 / 0x4] [Nesting level 1]---------------\nEmpty string '' parsed.\nParsing stopped at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_list() {
        let b:Vec<u8> = vec![1, 5];
        let expected = "Parsing started at offset 1, total length 2\n-------------- [Offset 1] [Tag 5 / 0x5] [Nesting level 1]---------------\nEmpty list [] parsed.\nParsing stopped at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_tuple() {
        let b:Vec<u8> = vec![1, 6];
        let expected = "Parsing started at offset 1, total length 2\n-------------- [Offset 1] [Tag 6 / 0x6] [Nesting level 1]---------------\nEmpty tuple (,) parsed.\nParsing stopped at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_set() {
        let b:Vec<u8> = vec![1, 7];
        let expected = "Parsing started at offset 1, total length 2\n-------------- [Offset 1] [Tag 7 / 0x7] [Nesting level 1]---------------\nEmpty set parsed.\nParsing stopped at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_dict() {
        let b:Vec<u8> = vec![1, 8];
        let expected = "Parsing started at offset 1, total length 2\n-------------- [Offset 1] [Tag 8 / 0x8] [Nesting level 1]---------------\nEmpty dict parsed.\nParsing stopped at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_int_zero() {
        let b:Vec<u8> = vec![1, 9];
        let expected = "Parsing started at offset 1, total length 2\n-------------- [Offset 1] [Tag 9 / 0x9] [Nesting level 1]---------------\nInteger zero 0 parsed.\nParsing stopped at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_bytes() {
        let b:Vec<u8> = vec![1, 18];
        let expected = "Parsing started at offset 1, total length 2\n-------------- [Offset 1] [Tag 18 / 0x12] [Nesting level 1]---------------\nEmpty bytes b'' parsed.\nParsing stopped at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dt_naive() {
        let b:Vec<u8> = vec![1, 27, 12, 65, 218, 171, 143, 165, 210, 133, 228];
        let expected = "Parsing started at offset 1, total length 11\n-------------- [Offset 1] [Tag 27 / 0x1B] [Nesting level 1]---------------\nDatetime without timezone tag found. A float (timestamp) is now expected.\n-------------- [Offset 2] [Tag 12 / 0xC] [Nesting level 1]---------------\nFloat tag found, attempting to parse it\nParsing of an 8-byte float started at offset 3\nAn 8-byte float (1789804183.289422) parsed; 8 bytes read\nFloat=1789804183.289422 parsed, pushing to cache\nDatetime without timezone (timestamp=1789804183.289422) parsed\nParsing stopped at offset 11\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_none_and_1_byte() {
        let b:Vec<u8> = vec![1, 0, 1];
        let expected = "Parsing started at offset 1, total length 3\n-------------- [Offset 1] [Tag 0 / 0x0] [Nesting level 1]---------------\nNone object parsed.\nParsing stopped at offset 2\n[ERROR] Corrupt data, finished at offset 2, but still have 1 bytes unparsed, total data length 3\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dt_offset() {
        let b:Vec<u8> = vec![1, 28, 12, 65, 218, 171, 144, 155, 38, 221, 160, 9];
        let expected = "Parsing started at offset 1, total length 12\n-------------- [Offset 1] [Tag 28 / 0x1C] [Nesting level 1]---------------\nDatetime with offset tag found. A float (timestamp) is now expected.\n-------------- [Offset 2] [Tag 12 / 0xC] [Nesting level 1]---------------\nFloat tag found, attempting to parse it\nParsing of an 8-byte float started at offset 3\nAn 8-byte float (1789805164.607277) parsed; 8 bytes read\nFloat=1789805164.607277 parsed, pushing to cache\nTimestamp=1789805164.607277 parsed. An integer (offset) is now expected.\n-------------- [Offset 11] [Tag 9 / 0x9] [Nesting level 1]---------------\nInteger zero 0 parsed.\nDatetime with offset (timestamp=1789805164.607277, offset=0) parsed\nParsing stopped at offset 12\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dt_iana() {
        let b:Vec<u8> = vec![1, 29, 12, 65, 218, 168, 19, 252, 0, 0, 0, 53, 69, 117, 114, 111, 112, 101, 47, 77, 111, 115, 99, 111, 119];
        let expected = "Parsing started at offset 1, total length 25\n-------------- [Offset 1] [Tag 29 / 0x1D] [Nesting level 1]---------------\nDatetime with IANA tag found. A float (timestamp) is now expected.\n-------------- [Offset 2] [Tag 12 / 0xC] [Nesting level 1]---------------\nFloat tag found, attempting to parse it\nParsing of an 8-byte float started at offset 3\nAn 8-byte float (1788891120) parsed; 8 bytes read\nFloat=1788891120 parsed, pushing to cache\nTimestamp=1788891120 parsed. A string (IANA) is now expected.\n-------------- [Offset 11] [Tag 53 / 0x35] [Nesting level 1]---------------\nString tag found, attempting to parse it\nParsing of a string started at offset 12\nIt is an optimized string, no need to parse its size; expected string size is 13\nString with len=13 parsed, pushing to cache\nDatetime with IANA (timestamp=1788891120, IANA='Europe/Moscow') parsed\nParsing stopped at offset 25\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_int_1() {
        let b:Vec<u8> = vec![1, 61];
        let expected = "Parsing started at offset 1, total length 2\n-------------- [Offset 1] [Tag 61 / 0x3D] [Nesting level 1]---------------\nOptimized int tag found, attempting to parse it\nInteger=1 parsed\nParsing stopped at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_float_pi() {
        let b:Vec<u8> = vec![1, 22, 186, 2];
        let expected = "Parsing started at offset 1, total length 4\n-------------- [Offset 1] [Tag 22 / 0x16] [Nesting level 1]---------------\nOptimized float tag found, attempting to parse it\nParsing of an optimized float started; expecting an int at offset 2\nParsing of a var_int started at offset 2\nVar_int parsing, offset 2\nVar_int parsing, offset 3\nVar_int parsed 314, 2 bytes read\nPositive float parsed: 3.14\nFloat=3.14 parsed, pushing to cache\nParsing stopped at offset 4\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_float_negative() {
        let b:Vec<u8> = vec![1, 12, 191, 241, 249, 173, 187, 143, 141, 167];
        let expected = "Parsing started at offset 1, total length 10\n-------------- [Offset 1] [Tag 12 / 0xC] [Nesting level 1]---------------\nFloat tag found, attempting to parse it\nParsing of an 8-byte float started at offset 2\nAn 8-byte float (-1.1234567) parsed; 8 bytes read\nFloat=-1.1234567 parsed, pushing to cache\nParsing stopped at offset 10\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_neg_int() {
        let b:Vec<u8> = vec![1, 11, 100];
        let expected = "Parsing started at offset 1, total length 3\n-------------- [Offset 1] [Tag 11 / 0xB] [Nesting level 1]---------------\nNegative integer tag found, attempting to parse it\nParsing of a var_int started at offset 2\nVar_int parsing, offset 2\nVar_int parsed 100, 1 bytes read\nInteger=100 parsed, pushing to cache\nParsing stopped at offset 3\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_short_string() {
        let b:Vec<u8> = vec![1, 44, 116, 101, 120, 116];
        let expected = "Parsing started at offset 1, total length 6\n-------------- [Offset 1] [Tag 44 / 0x2C] [Nesting level 1]---------------\nString tag found, attempting to parse it\nParsing of a string started at offset 2\nIt is an optimized string, no need to parse its size; expected string size is 4\nString with len=4 parsed, pushing to cache\nParsing stopped at offset 6\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_long_string() {
        let b:Vec<u8> = vec![1, 13, 16, 116, 101, 120, 116, 116, 101, 120, 116, 116, 101, 120, 116, 116, 101, 120, 116];
        let expected = "Parsing started at offset 1, total length 19\n-------------- [Offset 1] [Tag 13 / 0xD] [Nesting level 1]---------------\nString tag found, attempting to parse it\nParsing of a string started at offset 2\nExpected integer (string size) at offset 2\nParsing of a var_int started at offset 2\nVar_int parsing, offset 2\nVar_int parsed 16, 1 bytes read\nExpected string size: 16\nString with len=16 parsed, pushing to cache\nParsing stopped at offset 19\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_compressed_string() {
        let b:Vec<u8> = vec![1, 40, 14, 120, 156, 43, 73, 173, 40, 41, 193, 131, 1, 233, 104, 14, 41];
        let expected = "Parsing started at offset 1, total length 17\n-------------- [Offset 1] [Tag 40 / 0x28] [Nesting level 1]---------------\nString tag found, attempting to parse it\nParsing of a string started at offset 2\nExpected integer (string size) at offset 2\nParsing of a var_int started at offset 2\nVar_int parsing, offset 2\nVar_int parsed 14, 1 bytes read\nExpected string size: 14\nIt is compressed string, so decompress it...\nString with len=32 parsed, pushing to cache\nParsing stopped at offset 17\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
        // pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_list() {
        let b:Vec<u8> = vec![1, 82, 61, 62];
        let expected = "Parsing started at offset 1, total length 4\n-------------- [Offset 1] [Tag 82 / 0x52] [Nesting level 1]---------------\nList tag found, attempting to parse it\n\tIt is an optimized list, no need to parse its size\n\tList expects 2 elements\n\tProcessed List element 0\n-------------- [Offset 2] [Tag 61 / 0x3D] [Nesting level 2]---------------\n\tOptimized int tag found, attempting to parse it\n\tInteger=1 parsed\n\tProcessed List element 1\n-------------- [Offset 3] [Tag 62 / 0x3E] [Nesting level 2]---------------\n\tOptimized int tag found, attempting to parse it\n\tInteger=2 parsed\n\tList with 2 elements fully parsed\nParsing stopped at offset 4\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
        // pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_nested_list() {
        let b:Vec<u8> = vec![1, 82, 82, 61, 62, 61];
        let expected = "Parsing started at offset 1, total length 6\n-------------- [Offset 1] [Tag 82 / 0x52] [Nesting level 1]---------------\nList tag found, attempting to parse it\n\tIt is an optimized list, no need to parse its size\n\tList expects 2 elements\n\tProcessed List element 0\n-------------- [Offset 2] [Tag 82 / 0x52] [Nesting level 2]---------------\n\tList tag found, attempting to parse it\n\t\tIt is an optimized list, no need to parse its size\n\t\tList expects 2 elements\n\t\tProcessed List element 0\n-------------- [Offset 3] [Tag 61 / 0x3D] [Nesting level 3]---------------\n\t\tOptimized int tag found, attempting to parse it\n\t\tInteger=1 parsed\n\t\tProcessed List element 1\n-------------- [Offset 4] [Tag 62 / 0x3E] [Nesting level 3]---------------\n\t\tOptimized int tag found, attempting to parse it\n\t\tInteger=2 parsed\n\t\tList with 2 elements fully parsed\n\tProcessed List element 1\n-------------- [Offset 5] [Tag 61 / 0x3D] [Nesting level 2]---------------\n\tOptimized int tag found, attempting to parse it\n\tInteger=1 parsed\n\tList with 2 elements fully parsed\nParsing stopped at offset 6\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tuple() {
        let b:Vec<u8> = vec![1, 56, 61, 62];
        let expected = "Parsing started at offset 1, total length 4\n-------------- [Offset 1] [Tag 56 / 0x38] [Nesting level 1]---------------\nTuple tag found, attempting to parse it\n\tIt is optimized tuple, no need to parse its size\n\tTuple expects 2 elements\n\tProcessed tuple element 0\n-------------- [Offset 2] [Tag 61 / 0x3D] [Nesting level 2]---------------\n\tOptimized int tag found, attempting to parse it\n\tInteger=1 parsed\n\tProcessed tuple element 1\n-------------- [Offset 3] [Tag 62 / 0x3E] [Nesting level 2]---------------\n\tOptimized int tag found, attempting to parse it\n\tInteger=2 parsed\n\tTuple with 2 elements fully parsed\nParsing stopped at offset 4\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_set() {
        let b:Vec<u8> = vec![1, 16, 3, 61, 62, 63];
        let expected = "Parsing started at offset 1, total length 6\n-------------- [Offset 1] [Tag 16 / 0x10] [Nesting level 1]---------------\nSet tag found, attempting to parse it\n\tExpect Set size, parse int at 2\n\tParsing of a var_int started at offset 2\n\tVar_int parsing, offset 2\n\tVar_int parsed 3, 1 bytes read\n\tSet expects 3 elements\n\tProcessed Set element 0\n-------------- [Offset 3] [Tag 61 / 0x3D] [Nesting level 2]---------------\n\tOptimized int tag found, attempting to parse it\n\tInteger=1 parsed\n\tProcessed Set element 1\n-------------- [Offset 4] [Tag 62 / 0x3E] [Nesting level 2]---------------\n\tOptimized int tag found, attempting to parse it\n\tInteger=2 parsed\n\tProcessed Set element 2\n-------------- [Offset 5] [Tag 63 / 0x3F] [Nesting level 2]---------------\n\tOptimized int tag found, attempting to parse it\n\tInteger=3 parsed\n\tSet with 3 elements fully parsed\nParsing stopped at offset 6\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dict() {
        let b:Vec<u8> = vec![1, 17, 1, 61, 61];
        let expected = "Parsing started at offset 1, total length 5\n-------------- [Offset 1] [Tag 17 / 0x11] [Nesting level 1]---------------\nDict tag found, attempting to parse it\n\tExpect dict size, parse int at 2\n\tParsing of a var_int started at offset 2\n\tVar_int parsing, offset 2\n\tVar_int parsed 1, 1 bytes read\n\tDict expects 1 elements(pairs)\n\tStart parsing 0 element (pair) for dict\n-------------- [Offset 3] [Tag 61 / 0x3D] [Nesting level 2]---------------\n\tOptimized int tag found, attempting to parse it\n\tInteger=1 parsed\n-------------- [Offset 4] [Tag 61 / 0x3D] [Nesting level 2]---------------\n\tOptimized int tag found, attempting to parse it\n\tInteger=1 parsed\n\tDict with 1 elements (pairs) fully parsed\nParsing stopped at offset 5\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_list_and_caches() {
        let b:Vec<u8> = vec![1, 86, 22, 186, 2, 44, 116, 101, 115, 116, 10, 232, 132, 1, 37, 0, 38, 0, 39, 0];
        let expected = "Parsing started at offset 1, total length 20\n-------------- [Offset 1] [Tag 86 / 0x56] [Nesting level 1]---------------\nList tag found, attempting to parse it\n\tIt is an optimized list, no need to parse its size\n\tList expects 6 elements\n\tProcessed List element 0\n-------------- [Offset 2] [Tag 22 / 0x16] [Nesting level 2]---------------\n\tOptimized float tag found, attempting to parse it\n\tParsing of an optimized float started; expecting an int at offset 3\n\tParsing of a var_int started at offset 3\n\tVar_int parsing, offset 3\n\tVar_int parsing, offset 4\n\tVar_int parsed 314, 2 bytes read\n\tPositive float parsed: 3.14\n\tFloat=3.14 parsed, pushing to cache\n\tProcessed List element 1\n-------------- [Offset 5] [Tag 44 / 0x2C] [Nesting level 2]---------------\n\tString tag found, attempting to parse it\n\tParsing of a string started at offset 6\n\tIt is an optimized string, no need to parse its size; expected string size is 4\n\tString with len=4 parsed, pushing to cache\n\tProcessed List element 2\n-------------- [Offset 10] [Tag 10 / 0xA] [Nesting level 2]---------------\n\tPositive integer tag found, attempting to parse it\n\tParsing of a var_int started at offset 11\n\tVar_int parsing, offset 11\n\tVar_int parsing, offset 12\n\tVar_int parsing, offset 13\n\tVar_int parsed 17000, 3 bytes read\n\tInteger=17000 parsed, pushing to cache\n\tProcessed List element 3\n-------------- [Offset 14] [Tag 37 / 0x25] [Nesting level 2]---------------\n\tString cache tag found; expecting a 1-byte cache index at offset 15\n\tString cache requested index 0\n\tRetrieved from string cache: index=0, value='test'\n\tProcessed List element 4\n-------------- [Offset 16] [Tag 38 / 0x26] [Nesting level 2]---------------\n\tFloat cache tag found; expecting a 1-byte cache index at offset 17\n\tFloat cache requested index 0\n\tRetrieved from float cache: index=0, value=3.14\n\tProcessed List element 5\n-------------- [Offset 18] [Tag 39 / 0x27] [Nesting level 2]---------------\n\tInteger cache tag found; expecting a 1-byte cache index at offset 19\n\tInteger cache requested index 0\n\tRetrieved from integer cache: index=0, value=17000\n\tList with 6 elements fully parsed\nParsing stopped at offset 20\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }
}
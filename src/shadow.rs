use crate::constants::{Variant, FLOAT_BYTES, LIST_INDEX, TEN};
use crate::dec::{EMPTY_DICT, EMPTY_VEC, ParsedData, decode_optimized_int};
use crate::options::DecodeOptions;
use crate::utils::decompress;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::io::Write;


pub fn decode_varint<W: Write>(bts: &[u8], mut offset: usize, result: &mut W) -> PyResult<(u64, usize)> {
    let e_m = format!("Start parsing var_int at offset {}\n", offset);
    result.write_all(e_m.as_bytes())?;
    let start = offset;
    let mut number: u64 = 0;
    let mut shift = 0;
    loop {
        let &byte = bts.get(offset).ok_or_else(|| {
            PyValueError::new_err(format!("Nothing to read at offset: {}\n", offset))
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
                "Int is too long or data corrupted, offset: {}\n",
                offset
            )));
        }
    }
    let e_m = format!("Var_int parsed {}, bytes read {}\n", number, offset-start);
    result.write_all(e_m.as_bytes())?;
    Ok((number, offset - start))
}

fn decode_float<W: Write>(buffer: &[u8], offset: usize, result: &mut W) -> PyResult<(f64, usize)> {
    let e_m = format!("Start parsing float 8-bytes long at offset {}\n", offset);
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
            let e_m = format!("Float 8-bytes long parsed {}, bytes read - 8\n", res);
            result.write_all(e_m.as_bytes())?;
            Ok((res, FLOAT_BYTES)) },
        Err(e) => {
            let e_m = format!("Wrong data for float, error: {}\n", e);
            Err(PyValueError::new_err(e_m)) },
    }
}

fn decode_optimized_float<W: Write>(buffer: &[u8], offset: usize, tag: Variant, result: &mut W) -> PyResult<(f64, usize)> {
    let e_m = format!("Start parsing optimized float, expect int at {}\n", offset);
    result.write_all(e_m.as_bytes())?;
    let (value, read) = decode_varint(buffer, offset, result)?;
    if tag == Variant::FloatNoDecimals {
        let e_m = format!("Positive float without decimals parsed: {}\n", value);
        result.write_all(e_m.as_bytes())?;
        return Ok((value as f64, read));
    } else if tag == Variant::FloatNoDecimalsNeg {
        let e_m = format!("Negative float without decimals parsed: {}\n", -(value as f64));
        result.write_all(e_m.as_bytes())?;
        return Ok((-(value as f64), read));
    }
    if tag > Variant::FloatNoDecimals && tag <= Variant::Float6 {
        let dec_places = tag as u8 - 20; // cause FLOAT_1 = 21, FLOAT_2=22 etc.
        let pow = TEN.pow(dec_places as u32) as f64;
        let res = value as f64 / pow;
        let e_m = format!("Positive float parsed: {}\n", res);
        result.write_all(e_m.as_bytes())?;
        Ok((res, read))
    } else {
        let dec_places = tag as u8 - 30; // cause FLOAT_1_NEG = 31, FLOAT_2_NEG=32 etc.
        let pow = TEN.pow(dec_places as u32) as f64;
        let res = value as f64 / pow;
        let e_m = format!("Negative float parsed: {}\n", res);
        result.write_all(e_m.as_bytes())?;
        Ok((-res, read))
    }
}

fn decode_string<W: Write>(buffer: &[u8], offset: usize, tag: Variant, result: &mut W) -> PyResult<(String, usize)> {
    let e_m = format!("Start parsing string at offset {}\n", offset);
    result.write_all(e_m.as_bytes())?;
    let last_index;
    let mut new_offset = offset;
    if tag == Variant::String || tag == Variant::StringCompressed {
        let e_m = format!("We expect int here - size for string, at offset {}\n", offset);
        result.write_all(e_m.as_bytes())?;
        let (value, read) = decode_varint(buffer, offset, result)?;
        last_index = read + (value as usize) + offset;
        let e_m = format!("We expect size for string {}\n", value);
        result.write_all(e_m.as_bytes())?;
        if buffer.len() < last_index - 1 {
            let e_m = format!(
                "Not enough bytes, need {}, but have only {} bytes left at offset {}\n",
                value,
                buffer.len() - offset,
                offset
            );
            return Err(PyValueError::new_err(e_m));
        }
        new_offset = offset + read;
    } else {
        last_index = (tag as u8 - 40) as usize + offset; // cause STRING_1=41 etc.
        let e_m = format!("We expect size for string {}\n", (tag as u8 - 40));
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
        let e_m = "Decompress string...\n";
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
    let (elements_count, read) = if tag >= Variant::List1 && tag <= Variant::List10 {
        let elements_count: u64 = tag as u64 - LIST_INDEX as u64;
        let e_m = "It is optimized list, no need to parse size for it\n";
        res.write_all(e_m.as_bytes())?;
        (elements_count, 0)
    } else {
        let e_m = format!("Expect {} size, parse int at {}\n", name, offset);
        res.write_all(e_m.as_bytes())?;
        decode_varint(buffer, offset, res)?
    };
    let e_m = format!("{} expect {} elements\n", name, elements_count);
    res.write_all(e_m.as_bytes())?;
    let mut result = Vec::with_capacity(elements_count as usize);
    let mut new_offset = offset + read;
    for i in 0..elements_count {
        let e_m = format!("{} element {} at work\n", name, i);
        res.write_all(e_m.as_bytes())?;
        let (el, off) = decode(buffer, new_offset, opts, current_depth, res)?;
        result.push(el);
        new_offset = off
    }
    let e_m = format!("{} with {} elements fully parsed\n", name, elements_count);
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
    let (elements_count, read) = if tag >= Variant::Tuple2 && tag <= Variant::Tuple5 {
        let elements_count: u64 = match tag {
            Variant::Tuple2 => 2,
            Variant::Tuple3 => 3,
            Variant::Tuple4 => 4,
            _ => 5,
        };
        let e_m = "It is optimized tuple, no need to parse size for it\n";
        res.write_all(e_m.as_bytes())?;
        (elements_count, 0)
    } else {
        let e_m = format!("Expect tuple size, parse int at {}\n", offset);
        res.write_all(e_m.as_bytes())?;
        decode_varint(buffer, offset, res)?
    };
    let e_m = format!("Tuple expect {} elements\n", elements_count);
    res.write_all(e_m.as_bytes())?;
    let mut result = Vec::with_capacity(elements_count as usize);
    let mut new_offset = offset + read;
    for i in 0..elements_count {
        let e_m = format!("Tuple element {} at work\n", i);
        res.write_all(e_m.as_bytes())?;
        let (el, off) = decode(buffer, new_offset, opts, current_depth, res)?;
        result.push(el);
        new_offset = off
    }
    let e_m = format!("Tuple with {} elements fully parsed\n", elements_count);
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
    let e_m = format!("Expect dict size, parse int at {}\n", offset);
    res.write_all(e_m.as_bytes())?;
    let (elements_count, read) = decode_varint(buffer, offset, res)?;
    let e_m = format!("Dict expect {} elements(pairs)\n", elements_count);
    res.write_all(e_m.as_bytes())?;
    let mut result: Vec<(ParsedData, ParsedData)> = Vec::with_capacity(elements_count as usize);
    let mut new_offset = offset + read;
    for i in 0..elements_count {
        let e_m = format!("Start parsing {} pair for dict\n", i);
        res.write_all(e_m.as_bytes())?;
        let (key, off) = decode(buffer, new_offset, opts, current_depth, res)?;
        new_offset = off;
        let (value, off) = decode(buffer, new_offset, opts, current_depth, res)?;
        result.push((key, value));
        new_offset = off;
    }
    let e_m = format!("Dict with {} pairs fully parsed\n", elements_count);
    res.write_all(e_m.as_bytes())?;
    Ok((result, new_offset))
}

fn decode_bytes<W: Write>(buffer: &[u8], offset: usize, res: &mut W) -> PyResult<(Vec<u8>, usize)> {
    let e_m = format!("Expect bytes size, parse int at {}\n", offset);
    res.write_all(e_m.as_bytes())?;
    let (size, read) = decode_varint(buffer, offset, res)?;
    let e_m = format!("Bytes block expect size {}\n", size);
    res.write_all(e_m.as_bytes())?;
    let new_offset = offset + read;
    let last_index = new_offset + size as usize;
    if buffer.len() < last_index {
        let e_m = format!(
            "Not enough bytes, need {}, but have only {} bytes left at offset {}\n",
            last_index - offset,
            buffer.len() - offset,
            offset
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
    res: &mut W
) -> PyResult<(ParsedData, usize)> {
    if let Some(&index) = buffer.get(offset) {
        let e_m = format!("Integer cache requests for index {}\n", index);
        res.write_all(e_m.as_bytes())?;
        match opts.get_int(index) {
            Some(value) => {
                let e_m = format!("Get from integer cache from index {} value {}\n", index, value);
                res.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Int(*value), offset + 1)) },
            None => {
                let e_m = format!(
                    "Unexpected integer cache fail: nothing at index {}\n",
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
    res: &mut W
) -> PyResult<(ParsedData, usize)> {
    if let Some(&index) = buffer.get(offset) {
        let e_m = format!("Float cache requests for index {}\n", index);
        res.write_all(e_m.as_bytes())?;
        match opts.get_float(index) {
            Some(value) => {
                let e_m = format!("Get from float cache from index {} value {}\n", index, value);
                res.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Float(*value), offset + 1)) },
            None => {
                let e_m = format!(
                    "Unexpected float cache fail: nothing at index {}\n",
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
    res: &mut W
) -> PyResult<(ParsedData, usize)> {
    if let Some(&index) = buffer.get(offset) {
        let e_m = format!("String cache requests for index {}\n", index);
        res.write_all(e_m.as_bytes())?;
        match opts.get_string(index) {
            Some(value) => {
                let e_m = format!("Get from string cache from index {} value {}\n", index, value);
                res.write_all(e_m.as_bytes())?;
                Ok((ParsedData::String((*value).parse()?), offset + 1)) },
            None => {
                let e_m = format!(
                    "[CACHE] Unexpected string cache fail: nothing at index {}\n",
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
            "[OFFSET: {}] [DEPTH: {}] Nesting depth exceeded maximum of {}\n", offset,
            current_depth,
            opts.max_depth
        );
        result.write_all(e_m.as_bytes())?;
        return Err(PyValueError::new_err(e_m));
    }
    if let Some(&tag) = buffer.get(offset) {
        let e_m = format!(
            "-------------- [Offset {}] [Tag {} / {:#X?}] [Nesting level {}]---------------\n", offset, tag, tag, current_depth);
        result.write_all(e_m.as_bytes())?;
        let new_offset = offset + 1;
        match Variant::try_from(tag) {
            Ok(Variant::Null) => {
                let e_m = "\t None object parsed.\n";
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Null, new_offset)) },
            Ok(Variant::BoolTrue) => {
                let e_m = "\t Boolean True parsed.\n";
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::BoolTrue, new_offset))
            },
            Ok(Variant::BoolFalse) => {
                let e_m = "\t Boolean False parsed.\n";
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::BoolFalse, new_offset))
            },
            Ok(Variant::FloatZero) => {
                let e_m = "\t Float zero 0.0 parsed.\n";
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Float(0.0), new_offset))
            },
            Ok(Variant::StringEmpty) => {
                let e_m = "\t Empty string '' parsed.\n";
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::String("".to_string()), new_offset))
            },
            Ok(Variant::IntZero) => {
                let e_m = "\t Integer zero 0 parsed.\n";
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Int(0), new_offset))
            },
            Ok(Variant::ListEmpty) => {
                let e_m = "\t Empty list [] parsed.\n";
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::List(EMPTY_VEC), new_offset))
            },
            Ok(Variant::TupleEmpty) => {
                let e_m = "\t Empty tuple (,) parsed.\n";
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Tuple(EMPTY_VEC), new_offset))
            },
            Ok(Variant::SetEmpty) => {
                let e_m = "\t Empty set parsed.\n";
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Set(EMPTY_VEC), new_offset))
            },
            Ok(Variant::DictEmpty) => {
                let e_m = "\t Empty dict {} parsed.\n";
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Dict(EMPTY_DICT), new_offset))
            },
            Ok(Variant::BytesEmpty) => {
                let e_m = "\t Empty bytes b'' parsed.\n";
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::BytesEmpty, new_offset))
            },
            Ok(Variant::CacheInt) => decode_cached_int(buffer, new_offset, opts, result),
            Ok(Variant::CacheFloat) => decode_cached_float(buffer, new_offset, opts, result),
            Ok(Variant::CacheString) => decode_cached_string(buffer, new_offset, opts, result),
            Ok(Variant::DateTimeNoTz) => {
                let e_m = "\t Datetime without timezone found. Now we expect float (timestamp)\n";
                result.write_all(e_m.as_bytes())?;
                let (f, new_offset) = parse_float(buffer, new_offset, opts, current_depth, result)?;
                let e_m = format!("Datetime without timezone (timestamp={}) parsed\n", f);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::DateTimeNoTz(f), new_offset))
            }
            Ok(Variant::DateTimeOffset) => {
                let e_m = "\t Datetime with offset found. Now we expect float (timestamp)\n";
                result.write_all(e_m.as_bytes())?;
                let (f, new_offset) = parse_float(buffer, new_offset, opts, current_depth, result)?;
                let e_m = format!("Timestamp={} parsed. Now we expect integer (offset)\n", f);
                result.write_all(e_m.as_bytes())?;
                let (pd, new_offset) = decode(buffer, new_offset, opts, current_depth, result)?;
                match pd {
                    ParsedData::Int(v) => {
                        let e_m = format!("Datetime with offset (timestamp={}, offset={}) parsed\n", f, v);
                        result.write_all(e_m.as_bytes())?;
                        Ok((ParsedData::DateTimeOffset((f, v)), new_offset)) },
                    _ => Err(PyValueError::new_err(
                        "Unexpected type while parsing DateTime, expected Int\n",
                    )),
                }
            }
            Ok(Variant::DateTimeIana) => {
                let e_m = "\t Datetime with IANA found. Now we expect float (timestamp)\n";
                result.write_all(e_m.as_bytes())?;
                let (f, new_offset) = parse_float(buffer, new_offset, opts, current_depth, result)?;
                let e_m = format!("Timestamp={} parsed. Now we expect string (IANA)\n", f);
                result.write_all(e_m.as_bytes())?;
                let (pd, new_offset) = decode(buffer, new_offset, opts, current_depth, result)?;
                match pd {
                    ParsedData::String(v) => {
                        let e_m = format!("Datetime with IANA (timestamp={}, IANA={}) parsed\n", f, v);
                        result.write_all(e_m.as_bytes())?;
                        Ok((ParsedData::DateTimeIana((f, v)), new_offset)) },
                    _ => Err(PyValueError::new_err(
                        "Unexpected type while parsing DateTimeIana, expected String\n",
                    )),
                }
            }
            Ok(Variant::Float) => {
                let e_m = "\t Float tag found. Try to parse it\n";
                result.write_all(e_m.as_bytes())?;
                let (value, off) = decode_float(buffer, new_offset, result)?;
                let e_m = format!("Float={} parsed, push it to cache\n", value);
                result.write_all(e_m.as_bytes())?;
                opts.add_float(value);
                Ok((ParsedData::Float(value), new_offset + off))
            }
            Ok(i) if i >= Variant::Int1000 && i <= Variant::Int100 => {
                let e_m = "\t Optimized int tag found, try to parse it\n";
                result.write_all(e_m.as_bytes())?;
                let value = decode_optimized_int(i);
                let e_m = format!("Integer={} parsed", value);
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Int(value), new_offset))
            }
            Ok(t)
            if (t >= Variant::FloatNoDecimals && t <= Variant::Float6)
                || (t >= Variant::FloatNoDecimalsNeg && t <= Variant::Float6Neg) =>
                {
                    let e_m = "\t Optimized float tag found, try to parse it\n";
                    result.write_all(e_m.as_bytes())?;
                    let (value, off) = decode_optimized_float(buffer, new_offset, t, result)?;
                    let e_m = format!("Float={} parsed, push it to cache\n", value);
                    result.write_all(e_m.as_bytes())?;
                    opts.add_float(value);
                    Ok((ParsedData::Float(value), new_offset + off))
                }
            Ok(Variant::IntPositive) => {
                let e_m = "\t Positive integer tag found, try to parse it\n";
                result.write_all(e_m.as_bytes())?;
                let (value, off) = decode_varint(buffer, new_offset, result)?;
                let e_m = format!("Integer={} parsed, push it to cache\n", value);
                result.write_all(e_m.as_bytes())?;
                opts.add_int(value as i64);
                Ok((ParsedData::Int(value as i64), new_offset + off))
            }
            Ok(Variant::IntNegative) => {
                let e_m = "\t Negative integer tag found, try to parse it\n";
                result.write_all(e_m.as_bytes())?;
                let (value, off) = decode_varint(buffer, new_offset, result)?;
                let e_m = format!("Integer={} parsed, push it to cache\n", value);
                result.write_all(e_m.as_bytes())?;
                opts.add_int(-(value as i64));
                Ok((ParsedData::Int(-(value as i64)), new_offset + off))
            }
            Ok(t) if (t >= Variant::StringCompressed && t <= Variant::String15) => {
                let e_m = "\t String tag found, try to parse it\n";
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_string(buffer, new_offset, t, result)?;
                let e_m = format!("String with len={} parsed, push it to cache\n", value.len());
                result.write_all(e_m.as_bytes())?;
                opts.add_string(&value);
                Ok((ParsedData::String(value), offset))
            }
            Ok(Variant::String) => {
                let e_m = "\t String tag found, try to parse it\n";
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_string(buffer, new_offset, Variant::String, result)?;
                let e_m = format!("String with len={} parsed, push it to cache\n", value.len());
                result.write_all(e_m.as_bytes())?;
                opts.add_string(&value);
                Ok((ParsedData::String(value), offset))
            }
            Ok(t) if t == Variant::List || (t >= Variant::List1 && t <= Variant::List10) => {
                let e_m = "\t List tag found, try to parse it\n";
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_list(buffer, new_offset, t, opts, current_depth + 1, result, true)?;
                let e_m = format!("List with len={} parsed\n", value.len());
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::List(value), offset))
            }
            Ok(t) if t == Variant::Tuple || (t >= Variant::Tuple2 && t <= Variant::Tuple5) => {
                let e_m = "\t Tuple tag found, try to parse it\n";
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_tuple(buffer, new_offset, t, opts, current_depth + 1, result)?;
                let e_m = format!("Tuple with len={} parsed\n", value.len());
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Tuple(value), offset))
            }
            Ok(s) if s == Variant::Set => {
                let e_m = "\t Set tag found, try to parse it\n";
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_set(buffer, new_offset, s, opts, current_depth + 1, result)?;
                let e_m = format!("Set with len={} parsed\n", value.len());
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Set(value), offset))
            }
            Ok(Variant::Dict) => {
                let e_m = "\t Dict tag found, try to parse it\n";
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_dict(buffer, new_offset, opts, current_depth + 1, result)?;
                let e_m = format!("Dict with len={} parsed\n", value.len());
                result.write_all(e_m.as_bytes())?;
                Ok((ParsedData::Dict(value), offset))
            }
            Ok(Variant::Bytes) => {
                let e_m = "\t Bytes tag found, try to parse it\n";
                result.write_all(e_m.as_bytes())?;
                let (value, offset) = decode_bytes(buffer, new_offset, result)?;
                let e_m = format!("Bytes with len={} parsed\n", value.len());
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
    let e_m = format!("Starts parsing at offset {}\n", offset);
    result.write_all(e_m.as_bytes())?;
    match decode(&buffer, offset, &mut opts, 1, &mut result){
        Ok((_, s))=> {
            let e_m = format!("Stop parsing at offset {}\n", s);
            result.write_all(e_m.as_bytes())?;
            if s < buffer.len() {
                let e_m = format!("Corrupt data, finished at offset {}, but still have {} bytes unparsed\n", s, buffer.len()-s);
                result.write_all(e_m.as_bytes())?;
            }
        }
        Err(e) => {
            let e_m = format!("Stop parsing on error: {}\n", e);
            result.write_all(e_m.as_bytes())?;
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
        let expected = "Starts parsing at offset 1\n-------------- [Offset 1] [Tag 0 / 0x0] [Nesting level 1]---------------\n\t None object parsed.\nStop parsing at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_true() {
        let b:Vec<u8> = vec![1, 1];
        let expected = "Starts parsing at offset 1\n-------------- [Offset 1] [Tag 1 / 0x1] [Nesting level 1]---------------\n\t Boolean True parsed.\nStop parsing at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_false() {
        let b:Vec<u8> = vec![1, 2];
        let expected = "Starts parsing at offset 1\n-------------- [Offset 1] [Tag 2 / 0x2] [Nesting level 1]---------------\n\t Boolean False parsed.\nStop parsing at offset 2\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_none_and_1_byte() {
        let b:Vec<u8> = vec![1, 0, 1];
        let expected = "Starts parsing at offset 1\n-------------- [Offset 1] [Tag 0 / 0x0] [Nesting level 1]---------------\n\t None object parsed.\nStop parsing at offset 2\nCorrupt data, finished at offset 2, but still have 1 bytes unparsed\n";
        let result = explains(b, 1, 1000).ok().unwrap();
        assert_eq!(result, expected);
    }
}
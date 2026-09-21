//! Constants to use in all libraries

use std::cmp::Ordering;
use std::convert::TryFrom;

pub const INT_INDEX: usize = 60; // cause optimized ints starts with 61
pub const LIST_INDEX: usize = 80; // cause optimized lists starts with 81
pub const FLOAT_DEFAULT_LIMIT: f64 = 268_435_455.0; // see Protocol Specification
pub const TEN: u64 = 10;
pub const DEFAULT_CACHE_CAPACITY: usize = 20;
pub const DEFAULT_CACHE_STRING_LIMIT: usize = 250;
pub const DEFAULT_INT_LIMIT: i64 = 16384; // cause smaller ints took less than 3 bytes
pub const FLOAT_BYTES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Variant {
    Null = 0,
    BoolTrue = 1,
    BoolFalse = 2,
    FloatZero = 3,
    StringEmpty = 4,
    ListEmpty = 5,
    TupleEmpty = 6,
    SetEmpty = 7,
    DictEmpty = 8,
    IntZero = 9,
    IntPositive = 10,
    IntNegative = 11,
    Float = 12,
    String = 13,
    List = 14,
    Tuple = 15,
    Set = 16,
    Dict = 17,
    BytesEmpty = 18,
    Bytes = 19,
    FloatNoDecimals = 20,
    Float1 = 21,
    Float2 = 22,
    Float3 = 23,
    Float4 = 24,
    Float5 = 25,
    Float6 = 26,
    DateTimeNoTz = 27,
    DateTimeOffset = 28,
    DateTimeIana = 29,
    FloatNoDecimalsNeg = 30,
    Float1Neg = 31,
    Float2Neg = 32,
    Float3Neg = 33,
    Float4Neg = 34,
    Float5Neg = 35,
    Float6Neg = 36,
    CacheString = 37,
    CacheFloat = 38,
    CacheInt = 39,
    StringCompressed = 40,
    String1 = 41,
    String2 = 42,
    String3 = 43,
    String4 = 44,
    String5 = 45,
    String6 = 46,
    String7 = 47,
    String8 = 48,
    String9 = 49,
    String10 = 50,
    String11 = 51,
    String12 = 52,
    String13 = 53,
    String14 = 54,
    String15 = 55,
    Tuple2 = 56,
    Tuple3 = 57,
    Tuple4 = 58,
    Tuple5 = 59,
    Int1000 = 60,
    Int1 = 61,
    Int2 = 62,
    Int3 = 63,
    Int4 = 64,
    Int5 = 65,
    Int6 = 66,
    Int7 = 67,
    Int8 = 68,
    Int9 = 69,
    Int10 = 70,
    Int11 = 71,
    Int12 = 72,
    Int13 = 73,
    Int15 = 74,
    Int20 = 75,
    Int24 = 76,
    Int50 = 77,
    Int100 = 78,
    List1 = 81,
    List2 = 82,
    List3 = 83,
    List4 = 84,
    List5 = 85,
    List6 = 86,
    List7 = 87,
    List8 = 88,
    List9 = 89,
    List10 = 90,
}

impl Variant {
    pub const NULL_TAG: [u8; 1] = [Variant::Null as u8];
    pub const BOOL_TRUE_TAG: [u8; 1] = [Variant::BoolTrue as u8];
    pub const BOOL_FALSE_TAG: [u8; 1] = [Variant::BoolFalse as u8];
    pub const INT_POSITIVE_TAG: [u8; 1] = [Variant::IntPositive as u8];
    pub const INT_NEGATIVE_TAG: [u8; 1] = [Variant::IntNegative as u8];
    pub const INT_ZERO_TAG: [u8; 1] = [Variant::IntZero as u8];
    pub const FLOAT_ZERO_TAG: [u8; 1] = [Variant::FloatZero as u8];
    pub const STRING_EMPTY_TAG: [u8; 1] = [Variant::StringEmpty as u8];
    pub const BYTES_EMPTY_TAG: [u8; 1] = [Variant::BytesEmpty as u8];
    pub const DICT_EMPTY_TAG: [u8; 1] = [Variant::DictEmpty as u8];
    pub const SET_EMPTY_TAG: [u8; 1] = [Variant::SetEmpty as u8];
    pub const TUPLE_EMPTY_TAG: [u8; 1] = [Variant::TupleEmpty as u8];
    pub const LIST_EMPTY_TAG: [u8; 1] = [Variant::ListEmpty as u8];
    pub const BYTES_TAG: [u8; 1] = [Variant::Bytes as u8];
    pub const TUPLE_TAG: [u8; 1] = [Variant::Tuple as u8];
    pub const TUPLE_TAG2: [u8; 1] = [Variant::Tuple2 as u8];
    pub const TUPLE_TAG3: [u8; 1] = [Variant::Tuple3 as u8];
    pub const TUPLE_TAG4: [u8; 1] = [Variant::Tuple4 as u8];
    pub const TUPLE_TAG5: [u8; 1] = [Variant::Tuple5 as u8];
    pub const SET_TAG: [u8; 1] = [Variant::Set as u8];
    pub const DICT_TAG: [u8; 1] = [Variant::Dict as u8];
    pub const STRING_COMPRESSED_TAG: [u8; 1] = [Variant::StringCompressed as u8];
    pub const STRING_TAG: [u8; 1] = [Variant::String as u8];
    pub const LIST_TAG: [u8; 1] = [Variant::List as u8];
    pub const INT_TAG1: [u8; 1] = [Variant::Int1 as u8];
    pub const INT_TAG2: [u8; 1] = [Variant::Int2 as u8];
    pub const INT_TAG3: [u8; 1] = [Variant::Int3 as u8];
    pub const INT_TAG4: [u8; 1] = [Variant::Int4 as u8];
    pub const INT_TAG5: [u8; 1] = [Variant::Int5 as u8];
    pub const INT_TAG6: [u8; 1] = [Variant::Int6 as u8];
    pub const INT_TAG7: [u8; 1] = [Variant::Int7 as u8];
    pub const INT_TAG8: [u8; 1] = [Variant::Int8 as u8];
    pub const INT_TAG9: [u8; 1] = [Variant::Int9 as u8];
    pub const INT_TAG10: [u8; 1] = [Variant::Int10 as u8];
    pub const INT_TAG11: [u8; 1] = [Variant::Int11 as u8];
    pub const INT_TAG12: [u8; 1] = [Variant::Int12 as u8];
    pub const INT_TAG13: [u8; 1] = [Variant::Int13 as u8];
    pub const INT_TAG15: [u8; 1] = [Variant::Int15 as u8];
    pub const INT_TAG20: [u8; 1] = [Variant::Int20 as u8];
    pub const INT_TAG24: [u8; 1] = [Variant::Int24 as u8];
    pub const INT_TAG50: [u8; 1] = [Variant::Int50 as u8];
    pub const INT_TAG100: [u8; 1] = [Variant::Int100 as u8];
    pub const INT_TAG1000: [u8; 1] = [Variant::Int1000 as u8];
    pub const DT_IANA_TAG: [u8; 1] = [Variant::DateTimeIana as u8];
    pub const DT_OFFSET: [u8; 1] = [Variant::DateTimeOffset as u8];
    pub const DT_NAIVE: [u8; 1] = [Variant::DateTimeNoTz as u8];
    pub const STRING_TAG1: [u8; 1] = [Variant::String1 as u8];
    pub const STRING_TAG2: [u8; 1] = [Variant::String2 as u8];
    pub const STRING_TAG3: [u8; 1] = [Variant::String3 as u8];
    pub const STRING_TAG4: [u8; 1] = [Variant::String4 as u8];
    pub const STRING_TAG5: [u8; 1] = [Variant::String5 as u8];
    pub const STRING_TAG6: [u8; 1] = [Variant::String6 as u8];
    pub const STRING_TAG7: [u8; 1] = [Variant::String7 as u8];
    pub const STRING_TAG8: [u8; 1] = [Variant::String8 as u8];
    pub const STRING_TAG9: [u8; 1] = [Variant::String9 as u8];
    pub const STRING_TAG10: [u8; 1] = [Variant::String10 as u8];
    pub const STRING_TAG11: [u8; 1] = [Variant::String11 as u8];
    pub const STRING_TAG12: [u8; 1] = [Variant::String12 as u8];
    pub const STRING_TAG13: [u8; 1] = [Variant::String13 as u8];
    pub const STRING_TAG14: [u8; 1] = [Variant::String14 as u8];
    pub const STRING_TAG15: [u8; 1] = [Variant::String15 as u8];
    pub const LIST_TAG1: [u8; 1] = [Variant::List1 as u8];
    pub const LIST_TAG2: [u8; 1] = [Variant::List2 as u8];
    pub const LIST_TAG3: [u8; 1] = [Variant::List3 as u8];
    pub const LIST_TAG4: [u8; 1] = [Variant::List4 as u8];
    pub const LIST_TAG5: [u8; 1] = [Variant::List5 as u8];
    pub const LIST_TAG6: [u8; 1] = [Variant::List6 as u8];
    pub const LIST_TAG7: [u8; 1] = [Variant::List7 as u8];
    pub const LIST_TAG8: [u8; 1] = [Variant::List8 as u8];
    pub const LIST_TAG9: [u8; 1] = [Variant::List9 as u8];
    pub const LIST_TAG10: [u8; 1] = [Variant::List10 as u8];

    pub fn opt_int_tag(index: i64) -> [u8; 1] {
        match index {
            1 => Self::INT_TAG1,
            2 => Self::INT_TAG2,
            3 => Self::INT_TAG3,
            4 => Self::INT_TAG4,
            5 => Self::INT_TAG5,
            6 => Self::INT_TAG6,
            7 => Self::INT_TAG7,
            8 => Self::INT_TAG8,
            9 => Self::INT_TAG9,
            10 => Self::INT_TAG10,
            11 => Self::INT_TAG11,
            12 => Self::INT_TAG12,
            _ => Self::INT_TAG13,
        }
    }

    pub fn opt_string_tag(index: usize) -> [u8; 1] {
        match index {
            1 => Self::STRING_TAG1,
            2 => Self::STRING_TAG2,
            3 => Self::STRING_TAG3,
            4 => Self::STRING_TAG4,
            5 => Self::STRING_TAG5,
            6 => Self::STRING_TAG6,
            7 => Self::STRING_TAG7,
            8 => Self::STRING_TAG8,
            9 => Self::STRING_TAG9,
            10 => Self::STRING_TAG10,
            11 => Self::STRING_TAG11,
            12 => Self::STRING_TAG12,
            13 => Self::STRING_TAG13,
            14 => Self::STRING_TAG14,
            _ => Self::STRING_TAG15,
        }
    }

    pub fn opt_list_tag(index: usize) -> [u8; 1] {
        match index {
            1 => Self::LIST_TAG1,
            2 => Self::LIST_TAG2,
            3 => Self::LIST_TAG3,
            4 => Self::LIST_TAG4,
            5 => Self::LIST_TAG5,
            6 => Self::LIST_TAG6,
            7 => Self::LIST_TAG7,
            8 => Self::LIST_TAG8,
            9 => Self::LIST_TAG9,
            _ => Self::LIST_TAG10,
        }
    }
}
impl From<Variant> for u8 {
    #[inline(always)]
    fn from(variant: Variant) -> Self {
        variant as u8
    }
}

impl PartialOrd for Variant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (*self as u8).partial_cmp(&(*other as u8))
    }
}

impl TryFrom<u8> for Variant {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Variant::Null),
            1 => Ok(Variant::BoolTrue),
            2 => Ok(Variant::BoolFalse),
            3 => Ok(Variant::FloatZero),
            4 => Ok(Variant::StringEmpty),
            5 => Ok(Variant::ListEmpty),
            6 => Ok(Variant::TupleEmpty),
            7 => Ok(Variant::SetEmpty),
            8 => Ok(Variant::DictEmpty),
            9 => Ok(Variant::IntZero),
            10 => Ok(Variant::IntPositive),
            11 => Ok(Variant::IntNegative),
            12 => Ok(Variant::Float),
            13 => Ok(Variant::String),
            14 => Ok(Variant::List),
            15 => Ok(Variant::Tuple),
            16 => Ok(Variant::Set),
            17 => Ok(Variant::Dict),
            18 => Ok(Variant::BytesEmpty),
            19 => Ok(Variant::Bytes),
            20 => Ok(Variant::FloatNoDecimals),
            21 => Ok(Variant::Float1),
            22 => Ok(Variant::Float2),
            23 => Ok(Variant::Float3),
            24 => Ok(Variant::Float4),
            25 => Ok(Variant::Float5),
            26 => Ok(Variant::Float6),
            27 => Ok(Variant::DateTimeNoTz),
            28 => Ok(Variant::DateTimeOffset),
            29 => Ok(Variant::DateTimeIana),
            30 => Ok(Variant::FloatNoDecimalsNeg),
            31 => Ok(Variant::Float1Neg),
            32 => Ok(Variant::Float2Neg),
            33 => Ok(Variant::Float3Neg),
            34 => Ok(Variant::Float4Neg),
            35 => Ok(Variant::Float5Neg),
            36 => Ok(Variant::Float6Neg),
            37 => Ok(Variant::CacheString),
            38 => Ok(Variant::CacheFloat),
            39 => Ok(Variant::CacheInt),
            40 => Ok(Variant::StringCompressed),
            41 => Ok(Variant::String1),
            42 => Ok(Variant::String2),
            43 => Ok(Variant::String3),
            44 => Ok(Variant::String4),
            45 => Ok(Variant::String5),
            46 => Ok(Variant::String6),
            47 => Ok(Variant::String7),
            48 => Ok(Variant::String8),
            49 => Ok(Variant::String9),
            50 => Ok(Variant::String10),
            51 => Ok(Variant::String11),
            52 => Ok(Variant::String12),
            53 => Ok(Variant::String13),
            54 => Ok(Variant::String14),
            55 => Ok(Variant::String15),
            56 => Ok(Variant::Tuple2),
            57 => Ok(Variant::Tuple3),
            58 => Ok(Variant::Tuple4),
            59 => Ok(Variant::Tuple5),
            60 => Ok(Variant::Int1000),
            61 => Ok(Variant::Int1),
            62 => Ok(Variant::Int2),
            63 => Ok(Variant::Int3),
            64 => Ok(Variant::Int4),
            65 => Ok(Variant::Int5),
            66 => Ok(Variant::Int6),
            67 => Ok(Variant::Int7),
            68 => Ok(Variant::Int8),
            69 => Ok(Variant::Int9),
            70 => Ok(Variant::Int10),
            71 => Ok(Variant::Int11),
            72 => Ok(Variant::Int12),
            73 => Ok(Variant::Int13),
            74 => Ok(Variant::Int15),
            75 => Ok(Variant::Int20),
            76 => Ok(Variant::Int24),
            77 => Ok(Variant::Int50),
            78 => Ok(Variant::Int100),
            81 => Ok(Variant::List1),
            82 => Ok(Variant::List2),
            83 => Ok(Variant::List3),
            84 => Ok(Variant::List4),
            85 => Ok(Variant::List5),
            86 => Ok(Variant::List6),
            87 => Ok(Variant::List7),
            88 => Ok(Variant::List8),
            89 => Ok(Variant::List9),
            90 => Ok(Variant::List10),
            _ => Err("Invalid Variant byte code"),
        }
    }
}

pub fn tag_by_decimal_places(dec_places: usize, is_negative: bool) -> u8 {
    let tag = match dec_places {
        0 => Variant::FloatNoDecimals as u8,
        1 => Variant::Float1 as u8,
        2 => Variant::Float2 as u8,
        3 => Variant::Float3 as u8,
        4 => Variant::Float4 as u8,
        5 => Variant::Float5 as u8,
        6 => Variant::Float6 as u8,
        _ => panic!("Invalid decimal place code {}", dec_places),
    };
    if is_negative {
        return tag + 10; // cause FloatNeg1 =31 and Float1=21
    }
    tag
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_by_decimal_places() {
        assert_eq!(
            Variant::FloatNoDecimals as u8,
            tag_by_decimal_places(0, false)
        );
        assert_eq!(Variant::Float1 as u8, tag_by_decimal_places(1, false));
        assert_eq!(Variant::Float2 as u8, tag_by_decimal_places(2, false));
        assert_eq!(Variant::Float3 as u8, tag_by_decimal_places(3, false));
        assert_eq!(Variant::Float4 as u8, tag_by_decimal_places(4, false));
        assert_eq!(Variant::Float5 as u8, tag_by_decimal_places(5, false));
        assert_eq!(Variant::Float6 as u8, tag_by_decimal_places(6, false));
    }

    #[test]
    fn test_tag_by_decimal_places_negative() {
        assert_eq!(
            Variant::FloatNoDecimalsNeg as u8,
            tag_by_decimal_places(0, true)
        );
        assert_eq!(Variant::Float1Neg as u8, tag_by_decimal_places(1, true));
        assert_eq!(Variant::Float2Neg as u8, tag_by_decimal_places(2, true));
        assert_eq!(Variant::Float3Neg as u8, tag_by_decimal_places(3, true));
        assert_eq!(Variant::Float4Neg as u8, tag_by_decimal_places(4, true));
        assert_eq!(Variant::Float5Neg as u8, tag_by_decimal_places(5, true));
        assert_eq!(Variant::Float6Neg as u8, tag_by_decimal_places(6, true));
    }
}

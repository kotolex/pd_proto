use std::convert::TryFrom;

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
    FloatNoDecimals = 20,
    Float1 = 21,
    Float2 = 22,
    Float3 = 23,
    Float4 = 24,
    Float5 = 25,
    Float6 = 26,
    StringCompressed = 30,
    String1 = 31,
    String2 = 32,
    String3 = 33,
    String4 = 34,
    String5 = 35,
    String6 = 36,
    String7 = 37,
    String8 = 38,
    String9 = 39,
    String10 = 40,
    String11 = 41,
    String12 = 42,
    String13 = 43,
    String14 = 44,
    String15 = 45,
}

impl From<Variant> for u8 {
    #[inline(always)]
    fn from(variant: Variant) -> Self {
        variant as u8
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
            20 => Ok(Variant::FloatNoDecimals),
            21 => Ok(Variant::Float1),
            22 => Ok(Variant::Float2),
            23 => Ok(Variant::Float3),
            24 => Ok(Variant::Float4),
            25 => Ok(Variant::Float5),
            26 => Ok(Variant::Float6),
            30 => Ok(Variant::StringCompressed),
            31 => Ok(Variant::String1),
            32 => Ok(Variant::String2),
            33 => Ok(Variant::String3),
            34 => Ok(Variant::String4),
            35 => Ok(Variant::String5),
            36 => Ok(Variant::String6),
            37 => Ok(Variant::String7),
            38 => Ok(Variant::String8),
            39 => Ok(Variant::String9),
            40 => Ok(Variant::String10),
            41 => Ok(Variant::String11),
            42 => Ok(Variant::String12),
            43 => Ok(Variant::String13),
            44 => Ok(Variant::String14),
            45 => Ok(Variant::String15),
            _ => Err("Invalid Variant byte code"),
        }
    }
}

pub fn tag_by_decimal_places(dec_places: usize) -> u8 {
    match dec_places {
        0 => Variant::FloatNoDecimals as u8,
        1 => Variant::Float1 as u8,
        2 => Variant::Float2 as u8,
        3 => Variant::Float3 as u8,
        4 => Variant::Float4 as u8,
        5 => Variant::Float5 as u8,
        6 => Variant::Float6 as u8,
        _ => panic!("Invalid decimal place code {}", dec_places),
    }
}

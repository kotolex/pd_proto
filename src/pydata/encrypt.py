import struct

from src.pydata.const import (FLOAT_FORMAT, FLOAT_LIMIT, PROTOCOL_VERSION,
                              UTF_8, SupportedTypes, Variant,
                              tag_by_decimal_places)
from src.pydata.errors import UnsupportedTypeError
from src.pydata.utils import exponent


def encode_varint(number: int) -> bytearray:
    """
    Converts a positive integer into bytes using the VarInt format
    :param number: a positive integer
    :return: presentation bytes of the number
    """
    result = bytearray()
    while number > 0:
        byte = number & 0x7F
        number >>= 7
        if number > 0:
            byte |= 0x80
        result.append(byte)
    return result


def encode_none() -> bytes:
    """
    Returns byte representation for None
    :return: bytes representation of the None
    """
    return bytearray([Variant.NULL.value])


def encode_float(number: float) -> bytearray:
    """
    Converts a standard 8-byte floating-point number into bytes
    :param number: a standard Python floating-point number
    :return: bytes representation of the number
    """
    if number == 0.0:
        return bytearray([Variant.FLOAT_ZER0.value])
    if number <= FLOAT_LIMIT:
        dec_places = exponent(number)
        if dec_places < 7:
            tag = tag_by_decimal_places(dec_places)
            result = bytearray([tag])
            if dec_places == 0:
                result.extend(encode_varint(int(number)))
                return result
            limit = FLOAT_LIMIT / (10 ** dec_places)
            if number < limit:
                int_value = int(number * (10 ** dec_places))
                result.extend(encode_varint(int_value))
                return result
    result = bytearray([Variant.FLOAT.value])
    value = struct.pack(FLOAT_FORMAT, number)
    return result + value


def encode_bool(value: bool) -> bytearray:
    """
    Converts a boolean value into bytes
    :param value: a standard Python bool
    :return: bytes representation of True or False
    """
    tag = Variant.BOOL_TRUE.value if value else Variant.BOOL_FALSE.value
    return bytearray([tag])


def encode_string(value: str) -> bytearray:
    """
    Converts a string into bytes using the VarInt format for length and utf-8 encoding for content
    :param value: a string
    :return: bytes representation of string in utf-8
    """
    if not value:
        return bytearray([Variant.STRING_EMPTY.value])
    result = bytearray([Variant.STRING.value])
    encoded_str = value.encode(UTF_8)
    bytes_len = len(encoded_str)
    result.extend(encode_varint(bytes_len))
    return result + encoded_str


def encode_int(number: int) -> bytearray:
    """
    Converts integer into bytes using the VarInt format, for negative ints just use another code and same conversion
    :param number: any integer
    :return: bytes representation of the number
    """
    if number == 0:
        return bytearray([Variant.INT_ZERO.value])
    tag = Variant.INT_POSITIVE.value if number > 0 else Variant.INT_NEGATIVE.value
    if number < 0:
        number = number * (-1)
    value = encode_varint(number)
    result = bytearray([tag])
    return result + value


def _encode_collection(collection, empty: Variant, full: Variant) -> bytearray:
    if not collection:
        return bytearray([empty.value])
    result = bytearray([full.value])
    length = encode_varint(len(collection))
    result.extend(length)
    for e in collection:
        value = _encrypt_base(e)
        result.extend(value)
    return result


def encode_list(a_list: list) -> bytearray:
    """
    Converts list of supported types into bytes
    :param a_list: a list containing objects of supported types
    :return: bytes representation of the list
    """
    return _encode_collection(a_list, Variant.LIST_EMPTY, Variant.LIST)


def encode_tuple(a_tuple: tuple[SupportedTypes]) -> bytearray:
    """
    Converts tuple of supported types into bytes
    :param a_tuple: a tuple containing objects of supported types
    :return: bytes representation of the tuple
    """
    return _encode_collection(a_tuple, Variant.TUPLE_EMPTY, Variant.TUPLE)


def encode_set(a_set: set) -> bytearray:
    """
    Converts set of supported types into bytes
    :param a_set: a tuple containing objects of supported types
    :return: bytes representation of the tuple
    """
    return _encode_collection(a_set, Variant.SET_EMPTY, Variant.SET)


def encode_dict(a_dict: dict) -> bytearray:
    """
    Converts dict of supported types into bytes
    :param a_dict: a dict containing objects of supported types
    :return: bytes representation of the dict
    """
    if not a_dict:
        return bytearray([Variant.DICT_EMPTY.value])
    result = bytearray([Variant.DICT.value])
    length = encode_varint(len(a_dict))
    result.extend(length)
    for key, value in a_dict.items():
        key_encoded = _encrypt_base(key)
        result.extend(key_encoded)
        value_encoded = _encrypt_base(value)
        result.extend(value_encoded)
    return result


def _encrypt_base(data: SupportedTypes) -> bytearray:
    """
    Main and recursive function to encrypt different objects f supported types
    :param data: any object of supported types
    :return: array of bytes representation
    :raises UnsupportedTypeError when type is not supported
    """
    result = bytearray()
    match data:
        case n if n is None:
            result.extend(encode_none())
        case bool() as b:
            result.extend(encode_bool(b))
        case int() as i:
            result.extend(encode_int(i))
        case float() as f:
            result.extend(encode_float(f))
        case str() as s:
            result.extend(encode_string(s))
        case list() as a_list:
            result.extend(encode_list(a_list))
        case tuple() as t:
            result.extend(encode_tuple(t))
        case set() as a_set:
            result.extend(encode_set(a_set))
        case dict() as a_dict:
            result.extend(encode_dict(a_dict))
        case _:
            raise UnsupportedTypeError(f"Value of unsupported type -{data}-: {type(data)}")
    return result


def encrypt(data: SupportedTypes) -> bytes:
    """
    Converts a supported Python type into a sequence of bytes
    :param data: an object of any allowed type
    :return: bytes representation of data
    :raises UnsupportedTypeError when type is not supported
    """
    final = bytearray()
    final.append(PROTOCOL_VERSION)
    tail = _encrypt_base(data)
    final.extend(tail)
    return bytes(final)

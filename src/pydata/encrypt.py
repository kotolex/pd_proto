import struct

from src.pydata.const import (FLOAT_FORMAT, PROTOCOL_VERSION, SupportedTypes,
                              int_by_type)


def encode_float(number: float) -> bytes:
    """
    Converts a standard 8-byte floating-point number into bytes
    :param number: a standard Python floating-point number
    :return: presentation bytes of the number
    """
    return struct.pack(FLOAT_FORMAT, number)


def _encrypt_base(data: SupportedTypes) -> bytearray:
    result = bytearray()
    result.append(int_by_type(data))
    match data:
        case bool():
            pass  # just to leave the match and do not go to int cluse, bool is int
        case int() as y:
            if y < 0:
                y = (-1) * y
            result.extend(encode_varint(y))
        case float() as z:
            if z != 0.0:
                result.extend(encode_float(z))
        case str() as s:
            if s:
                pass
    return result


def encode_varint(number: int) -> bytearray:
    """
    Converts a non-negative integer into bytes using the VarInt format
    :param number: a non-negative integer (or 0)
    :return: presentation bytes of the number
    """
    if number < 0:
        raise ValueError("Positive numbers only!")
    if number == 0:
        return bytearray([0])
    result = bytearray()
    while number > 0:
        byte = number & 0x7F
        number >>= 7
        if number > 0:
            byte |= 0x80
        result.append(byte)
    return result


def encrypt(data: SupportedTypes) -> bytes:
    """
    Converts a supported Python type into a sequence of bytes
    :param data: an object of any allowed type
    :return: bytes representation of data
    """
    final = bytearray()
    final.append(PROTOCOL_VERSION)
    tail = _encrypt_base(data)
    final.extend(tail)
    return bytes(final)

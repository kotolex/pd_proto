import struct

from src.pydata.const import (FLOAT_FORMAT, PROTOCOL_VERSION, UTF_8, Variant,
                              obj_by_code)
from src.pydata.errors import EmptyDataError, ProtocolError


def decode_float(data: bytes) -> float:
    """
    Decode floating point number from bytes.
    :param data: a sequence of bytes
    :return: floating point number
    """
    return struct.unpack(FLOAT_FORMAT, data)[0]


def decode_varint(buffer: bytes) -> tuple[int, int]:
    """
    Decodes a VarInt representation into a number
    :param buffer: a sequence of bytes
    :return: a non-negative integer (or 0) and the number of bytes read
    """
    number = 0
    shift = 0
    bytes_read = 0

    for byte in buffer:
        bytes_read += 1
        number |= (byte & 0x7F) << shift
        if not byte & 0x80:
            break
        shift += 7
    return number, bytes_read


def decrypt(bts: bytes) -> Variant:
    """
    Decodes bytes into an object of one of the supported types
    :param bts: bytes representation of some object
    :return: an object of one of the supported types
    :raise AttributeError: in case of data corruption or protocol mismatch
    """
    if len(bts) <= 1:
        raise EmptyDataError("Empty data or unsupported protocol version")
    if bts[0] != PROTOCOL_VERSION:
        raise ProtocolError(f"Supported protocol version is less or equal {PROTOCOL_VERSION}")
    result = []
    index = 1
    continuation = False
    while True:
        b = bts[index]
        next_token = obj_by_code(b)
        if b >= Variant.INT_POSITIVE.value:
            if b in (Variant.INT_POSITIVE.value, Variant.INT_NEGATIVE.value):
                val, read = decode_varint(bts[index + 1:])
                result.append(next_token * val)
                index += read
            elif b == Variant.FLOAT.value:
                val = decode_float(bts[index + 1:index + 9])
                result.append(val)
                index += 9
            elif b == Variant.STRING.value:
                val, read = decode_varint(bts[index + 1:])
                index += read
                text = bts[index + 1:index + val + 1]
                result.append(text.decode(UTF_8))
                index += val + 1
        else:
            result.append(next_token)
        if not continuation:
            break
    if index + 1 >= len(bts):
        return result[0]
    raise ProtocolError(f"Corrupt data, parse bytes {index}, total length is {len(bts)}")

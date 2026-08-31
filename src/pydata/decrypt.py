import struct

from src.pydata.const import (FLOAT_FORMAT, PROTOCOL_VERSION, UTF_8, Variant, SupportedTypes)
from src.pydata.errors import EmptyDataError, ProtocolError


def decode_float(data: bytes, offset:int) -> tuple[float, int]:
    """
    Decode floating point number from bytes.
    :param data: a sequence of bytes
    :param offset: index to read from
    :return: floating point number
    """
    const = 8  # 8 bytes for float for now, TODO optimize
    if len(data) < const + offset - 1:
        raise ProtocolError(f"Corrupt data, not enough bytes, need {const}, but have only {len(data)} bytes left")
    result = struct.unpack(FLOAT_FORMAT, data[offset:const + offset])[0]
    return result, const


def decode_varint(buffer: bytes, offset:int) -> tuple[int, int]:
    """
    Decodes a VarInt representation into a number
    :param buffer: a sequence of bytes
    :param offset: index to read from
    :return: a non-negative integer (or 0) and the number of bytes read
    """
    number = 0
    shift = 0
    bytes_read = 0

    for byte in buffer[offset:]:
        bytes_read += 1
        number |= (byte & 0x7F) << shift
        if not byte & 0x80:
            break
        shift += 7
    return number, bytes_read


def decode_string(bts: bytes, offset:int) -> tuple[str, int]:
    val, read = decode_varint(bts, offset)
    last_index = read + val + offset + 1
    if len(bts) < last_index - 1:
        raise ProtocolError(f"Corrupt data, not enough bytes, need {last_index}, but have only {len(bts)} bytes left")
    offset+=read
    text = bts[offset:last_index]
    value = text.decode(UTF_8)
    return value, read + val + offset

def decode_list(bts: bytes, offset:int) -> tuple[list[SupportedTypes], int]:
    elements_count, read = decode_varint(bts, offset)
    result = []
    offset += read
    for _ in range(elements_count):
        el, new_offset = _decrypt_base(bts, offset)
        offset +=new_offset
        result.append(el)
    return result, offset

def _decrypt_base(bts: bytes, offset:int) -> tuple[SupportedTypes, int]:
    tag = bts[offset]
    offset+=1
    match tag:
        case Variant.NULL.value:
            return None, 1
        case Variant.BOOL_TRUE.value:
            return True, 1
        case Variant.BOOL_FALSE.value:
            return False, 1
        case Variant.FLOAT_ZER0.value:
            return 0.0, 1
        case Variant.STRING_EMPTY.value:
            return "", 1
        case Variant.INT_ZERO.value:
            return 0, 1
        case Variant.LIST_EMPTY.value:
            return [], 1
        case Variant.TUPLE_EMPTY.value:
            return tuple(), 1
        case Variant.SET_EMPTY.value:
            return set(), 1
        case Variant.DICT_EMPTY.value:
            return {}, 1
        case Variant.FLOAT.value:
            value, offset = decode_float(bts, offset)
            return value, offset + 1
        case Variant.INT_POSITIVE.value:
            value, offset = decode_varint(bts, offset)
            return value, offset + 1
        case Variant.INT_NEGATIVE.value:
            value, offset = decode_varint(bts, offset)
            return (-1) * value, offset + 1
        case Variant.STRING.value:
            value, offset = decode_string(bts, offset)
            return value, offset + 1
        case Variant.LIST.value:
            value, offset = decode_list(bts, offset)
            return value, offset + 1
        case _:
            raise ProtocolError(f"Corrupt data, unknown tag {tag} for current protocol version {PROTOCOL_VERSION}")


def decrypt(bts: bytes) -> SupportedTypes:
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
    result, offset = _decrypt_base(bts, 1)
    diff = len(bts) - offset - 1
    if diff > 0:
        raise ProtocolError(f"Corrupt data, finish on parse bytes {offset}, but still have {diff} bytes unparsed")
    return result

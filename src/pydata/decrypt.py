import struct

from src.pydata.const import (FLOAT_FORMAT, PROTOCOL_VERSION, UTF_8,
                              SupportedTypes, Variant)
from src.pydata.errors import (BytesLeftError, DecryptFloatError,
                               DecryptStringError, EmptyDataError,
                               ProtocolError, WrongTagError)


def decode_float(bts: bytes, offset: int) -> tuple[float, int]:
    """
    Decode floating point number from bytes.
    :param bts: a sequence of bytes
    :param offset: index to read from
    :return: tuple of floating point number and offset
    :raise: DecryptFloatError if failed to decode
    """
    const = 8  # 8 bytes for float for now, TODO optimize
    if len(bts) < const + offset:
        raise DecryptFloatError(f"Need {const} bytes, but have only {len(bts) - offset} bytes left at index {offset}")
    result = struct.unpack(FLOAT_FORMAT, bts[offset:const + offset])[0]
    return result, const


def decode_varint(bts: bytes, offset: int) -> tuple[int, int]:
    """
    Decodes a VarInt representation into a number
    :param bts: a sequence of bytes
    :param offset: index to read from
    :return: a non-negative integer (or 0) and the number of bytes read
    """
    number = 0
    shift = 0
    bytes_read = 0

    for byte in bts[offset:]:
        bytes_read += 1
        number |= (byte & 0x7F) << shift
        if not byte & 0x80:
            break
        shift += 7
    return number, bytes_read


def decode_string(bts: bytes, offset: int) -> tuple[str, int]:
    """
    Decodes a string representation into a string, always use UTF-8 encoding as python default one.
    :param bts: a sequence of bytes
    :param offset: index to read from
    :return: tuple of string and offset
    :raise: DecryptStringError if failed to decode
    """
    size, read = decode_varint(bts, offset)
    last_index = read + size + offset
    if len(bts) < last_index - 1:
        raise DecryptStringError(f"Not enough bytes, need {size}, but have only {len(bts) - offset} bytes left")
    offset += read
    text = bts[offset:last_index]
    value = text.decode(UTF_8)
    return value, last_index


def decode_list(bts: bytes, offset: int) -> tuple[list, int]:
    """
    Decodes a list representation into a list
    :param bts: a sequence of bytes
    :param offset: index to read from
    :return: tuple of list and offset
    """
    elements_count, read = decode_varint(bts, offset)
    result = []
    offset += read
    for _ in range(elements_count):
        el, offset = _decrypt_base(bts, offset)
        result.append(el)
    return result, offset


def decode_tuple(bts: bytes, offset: int) -> tuple[tuple, int]:
    """
    Decodes a tuple representation into a tuple
    :param bts: a sequence of bytes
    :param offset: index to read from
    :return: tuple and offset
    """
    result, offset = decode_list(bts, offset)
    return tuple(result), offset


def decode_set(bts: bytes, offset: int) -> tuple[set, int]:
    """
    Decodes a set representation into a set
    :param bts: a sequence of bytes
    :param offset: index to read from
    :return: set and offset
    """
    elements_count, read = decode_varint(bts, offset)
    result = set()
    offset += read
    for _ in range(elements_count):
        el, offset = _decrypt_base(bts, offset)
        result.add(el)
    return result, offset


def _decrypt_base(bts: bytes, offset: int) -> tuple[SupportedTypes, int]:
    tag = bts[offset]
    offset += 1
    match tag:
        case Variant.NULL.value:
            return None, offset
        case Variant.BOOL_TRUE.value:
            return True, offset
        case Variant.BOOL_FALSE.value:
            return False, offset
        case Variant.FLOAT_ZER0.value:
            return 0.0, offset
        case Variant.STRING_EMPTY.value:
            return "", offset
        case Variant.INT_ZERO.value:
            return 0, offset
        case Variant.LIST_EMPTY.value:
            return [], offset
        case Variant.TUPLE_EMPTY.value:
            return tuple(), offset
        case Variant.SET_EMPTY.value:
            return set(), offset
        case Variant.DICT_EMPTY.value:
            return {}, offset
        case Variant.FLOAT.value:
            value, off = decode_float(bts, offset)
            return value, offset + off
        case Variant.INT_POSITIVE.value:
            value, off = decode_varint(bts, offset)
            return value, offset + off
        case Variant.INT_NEGATIVE.value:
            value, off = decode_varint(bts, offset)
            return (-1) * value, offset + off
        case Variant.STRING.value:
            value, offset = decode_string(bts, offset)
            return value, offset
        case Variant.LIST.value:
            value, offset = decode_list(bts, offset)
            return value, offset
        case Variant.TUPLE.value:
            value, offset = decode_tuple(bts, offset)
            return value, offset
        case Variant.SET.value:
            value, offset = decode_set(bts, offset)
            return value, offset
        case _:
            raise WrongTagError(f"Unknown tag {tag} for current protocol version {PROTOCOL_VERSION}")


def decrypt(bts: bytes) -> SupportedTypes:
    """
    Decodes bytes into an object of one of the supported types
    :param bts: bytes representation of some object
    :return: an object of one of the supported types
    :raise EmptyDataError: if no data can be decoded
    :raise ProtocolError: if a protocol version does not match the current one
    :raise BytesLeftError: if not all bytes was parsed
    """
    if len(bts) <= 1:
        raise EmptyDataError("Nothing to decrypt")
    if bts[0] != PROTOCOL_VERSION:
        raise ProtocolError(f"Supported protocol version is less or equal {PROTOCOL_VERSION}")
    result, offset = _decrypt_base(bts, 1)
    diff = len(bts) - offset - 1
    if diff > 0:
        raise BytesLeftError(f"Corrupt data, finish on parse bytes {offset}, but still have {diff} bytes unparsed")
    return result

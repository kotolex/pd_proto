from pd_proto import SupportedTypes

def encode_varint(number: int) -> bytearray:
    """
    Converts a positive integer into bytes using the VarInt format
    :param number: a positive integer
    :return: presentation bytes of the number
    """
    ...


def encode_float(number: float) -> bytearray:
    """
    Converts a standard 8-byte floating-point number into bytes
    :param number: a standard Python floating-point number
    :return: bytes representation of the number
    """
    ...


def pack(data: SupportedTypes, proto_version: int, max_depth: int, float_limit: float,
                 string_length_limit: int) -> bytes:
    """
    Converts a supported Python type into a sequence of bytes. Implemented in Rust.

    :param data: An object of any allowed type.
    :param proto_version: The version of the protocol to use.
    :param max_depth: Maximum nesting depth for collections; raises an error if exceeded.
                      Set to 0 to disable this check (warning: can lead to errors).
    :param float_limit: Threshold for float optimization. If a float is less than this value,
                        pd_proto will attempt to optimize it. Set to 0 to disable optimization.
    :param string_length_limit: Threshold for string optimization. If a string is longer than this value,
                                pd_proto will attempt to compress it. Set to 0 to disable optimization.
    :return: A bytes representation of the data.
    :raises AttributeError: If the object type is not supported.
    :raises ValueError: If the maximum nesting depth is exceeded.
    """
    ...

def decode_varint(bts: bytes, offset: int) -> tuple[int, int]:
    """
    Decodes a VarInt representation into an integer.

    :param bts: A sequence of bytes.
    :param offset: The byte index to start reading from.
    :return: A tuple containing the non-negative integer and the number of bytes read.
    """
    ...


def unpack(bts: bytes, offset: int, max_depth:int) -> tuple[SupportedTypes, int]:
    """
    Decodes bytes into an object of a supported type. Written in Rust.

    :param bts: A bytes representation of an object.
    :param max_depth: Maximum nesting depth for collections; raises an error if exceeded.
                      Set to 0 to disable this check (warning: can lead to errors).
    :param offset: The byte index to start reading from.
    :return: A tuple containing the decoded object and the new offset value.
    :raises ValueError: If no data can be decoded, not all bytes were parsed, or the data is corrupted.
    """
    ...

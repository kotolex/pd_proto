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


def real_encrypt(data: SupportedTypes, proto_version: int, max_depth: int, float_limit: float,
                 string_length_limit: int) -> bytes:
    """
    Converts a supported Python type into a sequence of bytes. Implemented in Rust

    :param data: an object of any allowed type
    :param proto_version: version of the protocol
    :param max_depth: maximum nesting depth for collections, raise an error if exceeded. Use 0 to disabled it, but it
    can lead to error
    :param float_limit: limit for float optimisation, if float less than that value, pd_proto will try to optimize it.
    Use 0 to disable optimisation
    :param string_length_limit: limit for string optimisation, if string greater than that value, pd_proto will try
    to compress it. Use 0 to disable optimisation
    :return: bytes representation of data
    :raises AttributeError when type is not supported
    :raises ValueError when parsing depth exceeded
    """
    ...

def decode_varint(bts: bytes, offset: int) -> tuple[int, int]:
    """
    Decodes a VarInt representation into a number
    :param bts: a sequence of bytes
    :param offset: index to read from
    :return: a non-negative integer (or 0) and the number of bytes read
    """
    ...


def real_decrypt(bts: bytes, offset: int, max_depth:int) -> tuple[SupportedTypes, int]:
    """
    Decodes bytes into an object of one of the supported types. Written in Rust.
    :param bts: bytes representation of some object
    :param max_depth: maximum nesting depth for collections, raise an error if exceeded. Use 0 to disabled it, but it
    can lead to error
    :param offset: index to read from
    :return: a pair of an object of one of the supported types and offset value
    :raise ValueError: if no data can be decoded, not all bytes was parsed, or data corrupted
    """
    ...

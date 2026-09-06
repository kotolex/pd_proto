from pd_proto import SupportedTypes


def exponent(number: float) -> int:
    """
    Returns the number of decimal places
    :param number: float number
    :return: int
    """
    ...


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


def real_encrypt(data: SupportedTypes, proto_version: int) -> bytes:
    """
    Converts a supported Python type into a sequence of bytes. Implemented in Rust

    :param data: an object of any allowed type
    :param proto_version: version of the protocol
    :return: bytes representation of data
    :raises AttributeError when type is not supported
    :raises ValueError when parsing depth exceeded
    """
    ...

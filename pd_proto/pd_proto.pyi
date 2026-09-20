from pd_proto import SupportedTypes
from pd_proto.const import SupportsWrite

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


def packf(file: SupportsWrite, data: SupportedTypes, proto_version: int, max_depth: int, float_limit: float,
          string_length_limit: int) -> bytes:
    """
    Serializes a supported Python object into a byte sequence and writes it directly to a file using efficient
    buffered I/O. Implemented in Rust.

    :param file: A file opened to write bytes.
    :param data: An object of any allowed type.
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


def unpack(bts: bytes, offset: int, max_depth: int) -> tuple[SupportedTypes, int]:
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

def unpackf(file_descriptor: int, offset: int, max_depth: int) -> tuple[SupportedTypes, int]:
    """
    Decodes bytes from binary file into an object of a supported type. Written in Rust.

    :param file_descriptor: A file_descriptor of real file opened to read bytes.
    :param max_depth: Maximum nesting depth for collections; raises an error if exceeded.
                      Set to 0 to disable this check (warning: can lead to errors).
    :param offset: The byte index to start reading from.
    :return: A tuple containing the decoded object and the new offset value.
    :raises ValueError: If no data can be decoded, not all bytes were parsed, or the data is corrupted.
    """
    ...

def checksum(bts:bytes) -> int:
    """
    Calculates the Adler-32 checksum of the provided byte data. Written in Rust.

    :param bts: The input binary data to be checksummed.
    :return: An integer representing the calculated checksum.
    """
    ...

def checksum_file(file_descriptor: int) -> int:
    """
    Calculates the Adler-32 checksum of the provided binary file. Written in Rust.

    :param file_descriptor: A file_descriptor of real file opened to read bytes.
    :return: An integer representing the calculated checksum.
    """
    ...

def explains(bts: bytes, offset: int, max_depth: int) -> str:
    """
    This function provides a detailed step-by-step data unpacking algorithm.
    It deserializes the provided bytes and returns a comprehensive report as a string, while discarding the actual
    unpacked result. This Rust-implemented function is designed specifically for data and protocol debugging.

    :param bts: A bytes representation of an object.
    :param offset: The byte index to start reading from.
    :param max_depth: Maximum nesting depth for collections; raises an error if exceeded.
                      Set to 0 to disable this check (warning: can lead to errors).
    :return: Detailed report as a string.
    """
    ...


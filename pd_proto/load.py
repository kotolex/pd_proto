import os

from pd_proto import _unpack, _unpackf, _explains
from pd_proto.const import (DEPTH_LIMIT, PROTOCOL_VERSION,
                            SupportedTypes, SupportsRead)
from pd_proto.errors import (BytesLeftError, CycleLinksError,
                             DataCorruptionError, EmptyDataError,
                             ParseFloatError, ParseStringError, PDProtoError,
                             ProtocolError, WrongTagError)
from pd_proto.utils import check_binary_file_for_reading, get_sys_handle

BYTES = "[BYTES]"
CACHE = "[CACHE]"
DATA = "[DATA]"
DEPTH = "[DEPTH]"
END = "[END]"
FLOAT = "[FLOAT]"
STRING = "[STRING]"
TAG = "[TAG]"
UNPARSED = "[UNPARSED]"

ERROR_MAPPING = {
    BYTES: DataCorruptionError,
    END: DataCorruptionError,
    DATA: DataCorruptionError,
    CACHE: DataCorruptionError,
    TAG: WrongTagError,
    FLOAT: ParseFloatError,
    STRING: ParseStringError,
    DEPTH: CycleLinksError,
    UNPARSED: BytesLeftError,
}

def _check_valid_bytes(bts:bytes):
    if len(bts) <= 1:
        raise EmptyDataError("Nothing to decrypt")
    if bts[0] > PROTOCOL_VERSION:
        raise ProtocolError(f"Supported protocol version is less or equal {PROTOCOL_VERSION}")

def _load(use_file: bool, *args):
    action = _unpackf if use_file else _unpack
    try:
        result, offset = action(*args)
    except ValueError as e:
        str_error = str(e)
        for key, error in ERROR_MAPPING.items():
            if str_error.startswith(key):
                raise error(str_error) from None
        raise PDProtoError(f"Rust backend fail: {str_error}") from e
    return result


def loads(bts: bytes, max_depth: int = DEPTH_LIMIT) -> SupportedTypes:
    """
    Decodes bytes into an object of a supported type.

    :param bts: A bytes representation of an object.
    :param max_depth: Maximum nesting depth for collections; raises an error if exceeded.
                      Set to 0 to disable this check (warning: can lead to errors).
    :return: An object of one of the supported types.
    :raises EmptyDataError: If no data can be decoded.
    :raises ProtocolError: If the protocol version does not match the current one.
    :raises BytesLeftError: If not all bytes were parsed.
    :raises WrongTagError: If an invalid tag appears in the data.
    :raises PDProtoError: For any other error in the Rust backend.
    """
    _check_valid_bytes(bts)
    return _load(False, bts, 1, max_depth)


def load(file: SupportsRead, max_depth: int = DEPTH_LIMIT) -> SupportedTypes:
    """
    Decodes bytes into an object of a supported type.

    :param file: A file opened to read bytes.
    :param max_depth: Maximum nesting depth for collections; raises an error if exceeded.
                      Set to 0 to disable this check (warning: can lead to errors).
    :return: An object of one of the supported types.
    :raises EmptyDataError: If no data can be decoded.
    :raises ProtocolError: If the protocol version does not match the current one.
    :raises BytesLeftError: If not all bytes were parsed.
    :raises WrongTagError: If an invalid tag appears in the data.
    :raises BinaryFileError: if it is not a real file, file is not readable, or opened for read text
    :raises PDProtoError: For any other error in the Rust backend.
    """
    check_binary_file_for_reading(file)
    fd = get_sys_handle(file)
    file_size = os.fstat(file.fileno()).st_size
    if file_size <= 2:
        raise EmptyDataError("Nothing to decrypt")
    if file.read(1)[0] > PROTOCOL_VERSION:
        raise ProtocolError(f"Supported protocol version is less or equal {PROTOCOL_VERSION}")
    return _load(True, fd, 1, max_depth)


def explains(bts: bytes, max_depth: int = DEPTH_LIMIT) -> str:
    """
    This function provides a detailed step-by-step data unpacking algorithm.
    It deserializes the provided bytes and returns a comprehensive report as a string, while discarding the actual
    unpacked result.

    :param bts: A bytes representation of an object.
    :param max_depth: Maximum nesting depth for collections; raises an error if exceeded.
                      Set to 0 to disable this check (warning: can lead to errors).
    :return: Detailed report as a string.
    :raises EmptyDataError: If no data can be decoded.
    :raises ProtocolError: If the protocol version does not match the current one.
    """
    _check_valid_bytes(bts)
    return _explains(bts, 1, max_depth)
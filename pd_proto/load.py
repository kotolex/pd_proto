import os
import sys

from pd_proto import unpack, unpackf
from pd_proto.const import (DEPTH_LIMIT, ENCODING, FILENO, PROTOCOL_VERSION,
                            READABLE, SupportedTypes, SupportsRead)
from pd_proto.errors import (BinaryFileError, BytesLeftError, CycleLinksError,
                             DataCorruptionError, EmptyDataError,
                             ParseFloatError, ParseStringError, PDProtoError,
                             ProtocolError, WrongTagError)

BYTES = "[BYTES]"
CACHE = "[CACHE]"
DATA = "[DATA]"
DEPTH = "[DEPTH]"
END = "[END]"
FLOAT = "[FLOAT]"
STRING = "[STRING]"
TAG = "[TAG]"
ERROR_MAPPING = {
    BYTES: DataCorruptionError,
    END: DataCorruptionError,
    DATA: DataCorruptionError,
    CACHE: DataCorruptionError,
    TAG: WrongTagError,
    FLOAT: ParseFloatError,
    STRING: ParseStringError,
    DEPTH: CycleLinksError
}


def get_sys_handle(file) -> int:
    """
    Get the OS file descriptor or handle.

    :param file: A real file-like object present in the filesystem.
    :return: The system descriptor or handle as an integer.
    """
    fd = file.fileno()
    if sys.platform == "win32":
        import msvcrt  # pylint: disable=import-outside-toplevel
        return msvcrt.get_osfhandle(fd)
    return fd


def _load(use_file: bool, length, *args):
    action = unpackf if use_file else unpack
    try:
        result, offset = action(*args)
    except ValueError as e:
        str_error = str(e)
        for key, error in ERROR_MAPPING.items():
            if str_error.startswith(key):
                raise error(str_error) from None
        raise PDProtoError(f"Rust backend fail: {str_error}") from e
    diff = length - offset - 1
    if diff > 0:
        raise BytesLeftError(f"Corrupt data, finish on parse bytes {offset}, but still have {diff} bytes unparsed")
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
    if len(bts) <= 1:
        raise EmptyDataError("Nothing to decrypt")
    if bts[0] > PROTOCOL_VERSION:
        raise ProtocolError(f"Supported protocol version is less or equal {PROTOCOL_VERSION}")
    return _load(False, len(bts), bts, 1, max_depth)


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
    if not getattr(file, READABLE, lambda: False)():
        raise BinaryFileError("The 'file' parameter must be real file opened for reading bytes.")
    if not hasattr(file, FILENO):
        raise BinaryFileError("Expected a real file on disk, not an in-memory stream.")
    if hasattr(file, ENCODING):
        raise BinaryFileError("Param 'file' must be a binary file opened for reading (unexpected 'encoding' attribute)")
    fd = get_sys_handle(file)
    file_size = os.fstat(file.fileno()).st_size
    if file_size <= 2:
        raise EmptyDataError("Nothing to decrypt")
    if file.read(1)[0] > PROTOCOL_VERSION:
        raise ProtocolError(f"Supported protocol version is less or equal {PROTOCOL_VERSION}")
    return _load(True, file_size, fd, 1, max_depth)

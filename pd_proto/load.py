from pd_proto import _unpack, _unpackf, _explains, _explain
from pd_proto.const import (DEPTH_LIMIT, PROTOCOL_VERSION,
                            SupportedTypes, SupportsRead, SupportsWriteText, ENCODING, WRITABLE)
from pd_proto.errors import (BytesLeftError, CycleLinksError,
                             DataCorruptionError, EmptyDataError,
                             ParseFloatError, ParseStringError, PDProtoError,
                             ProtocolError, WrongTagError, TextFileError)
from pd_proto.utils import check_binary_and_get_descriptor

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


def _check_valid_bytes(bts: bytes):
    if len(bts) <= 1:
        raise EmptyDataError("Nothing to decrypt")
    if bts[0] > PROTOCOL_VERSION:
        raise ProtocolError(f"Supported protocol version is less or equal {PROTOCOL_VERSION}")


def _load(use_file: bool, *args):
    action = _unpackf if use_file else _unpack
    try:
        result, _ = action(*args)
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
    fd = check_binary_and_get_descriptor(file)
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


def explain(file_src: SupportsRead, file_dst: SupportsWriteText, max_depth: int = DEPTH_LIMIT) -> None:
    """
    This function implements a detailed step-by-step data unpacking algorithm.
    It deserializes the provided bytes and writes them to the specified text file,
    while discarding the actual unpacked result.

    :param file_src: A file opened to read bytes.
    :param file_dst: A file-like object opened for writing text in UTF-8.
    :param max_depth: Maximum nesting depth for collections; raises an error if exceeded.
                      Set to 0 to disable this check (warning: can lead to errors).
    :raises EmptyDataError: If no data can be decoded.
    :raises ProtocolError: If the protocol version does not match the current one.
    :raises BytesLeftError: If not all bytes were parsed.
    :raises WrongTagError: If an invalid tag appears in the data.
    :raises BinaryFileError: if file_src is not a real file, file is not readable, or opened for read text
    :raises TextFileError: if file_dst is not a real file, file is not writable, or opened for read text
    :raises PDProtoError: For any other error in the Rust backend.
    """
    fd = check_binary_and_get_descriptor(file_src)
    if not hasattr(file_dst, ENCODING):
        raise TextFileError("Param 'file_dst' must be a text file opened for writing (expected 'encoding' attribute)")
    if not getattr(file_dst, WRITABLE, lambda: False)():
        raise TextFileError("The file is not opened for writing or lacks 'writable()' method.")
    return _explain(fd, file_dst, 1, max_depth)

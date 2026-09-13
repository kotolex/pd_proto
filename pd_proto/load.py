from pd_proto import unpack
from pd_proto.const import (PROTOCOL_VERSION, SupportedTypes, DEPTH_LIMIT)
from pd_proto.errors import (BytesLeftError, EmptyDataError, ProtocolError, ParseStringError, DataCorruptionError,
                             WrongTagError, ParseFloatError, PDProtoError, CycleLinksError)

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
    try:
        result, offset = unpack(bts, 1, max_depth)
    except ValueError as e:
        str_error = str(e)
        for key, error in ERROR_MAPPING.items():
            if str_error.startswith(key):
                raise error(str_error) from None
        raise PDProtoError(f"Rust backend fail: {str_error}") from e
    diff = len(bts) - offset - 1
    if diff > 0:
        raise BytesLeftError(f"Corrupt data, finish on parse bytes {offset}, but still have {diff} bytes unparsed")
    return result

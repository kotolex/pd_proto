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
    Decodes bytes into an object of one of the supported types
    :param bts: bytes representation of some object
    :param max_depth: maximum nesting depth for collections, raise an error if exceeded. Use 0 to disable it, but it
    can lead to error
    :return: an object of one of the supported types
    :raise EmptyDataError: if no data can be decoded
    :raise ProtocolError: if a protocol version does not match the current one
    :raise BytesLeftError: if not all bytes was parsed
    :raise WrongTagError: if wrong tag appears in data
    :raise PDProtoError: on any other error in Rust backend
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

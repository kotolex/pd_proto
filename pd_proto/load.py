from pd_proto import unpack
from pd_proto.const import (PROTOCOL_VERSION, SupportedTypes, DEPTH_LIMIT)
from pd_proto.errors import (BytesLeftError, EmptyDataError, ProtocolError, ParseStringError, DataCorruptionError,
                             WrongTagError, ParseFloatError, PDProtoError, CycleLinksError)

DATA = "[DATA]"
DEPTH = "[DEPTH]"
END = "[END]"
FLOAT = "[FLOAT]"
STRING = "[STRING]"
TAG = "[TAG]"


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
    if bts[0] != PROTOCOL_VERSION:
        raise ProtocolError(f"Supported protocol version is less or equal {PROTOCOL_VERSION}")
    try:
        result, offset = unpack(bts, 1, max_depth)
    except ValueError as e:
        str_error = str(e)
        if STRING in str_error:
            raise ParseStringError(str_error.replace(STRING, "")) from None
        if END in str_error or DATA in str_error:
            raise DataCorruptionError(str_error.replace(END, "").replace(DATA, "")) from None
        if TAG in str_error:
            raise WrongTagError(str_error.replace(TAG, "")) from None
        if FLOAT in str_error:
            raise ParseFloatError(str_error.replace(FLOAT, "")) from None
        if DEPTH in str_error:
            raise CycleLinksError(str_error.replace(DEPTH, "")) from None
        raise PDProtoError("Unexpected error") from e
    diff = len(bts) - offset - 1
    if diff > 0:
        raise BytesLeftError(f"Corrupt data, finish on parse bytes {offset}, but still have {diff} bytes unparsed")
    return result

from pd_proto import real_decrypt
from pd_proto.const import (PROTOCOL_VERSION, SupportedTypes)
from pd_proto.errors import (BytesLeftError, EmptyDataError, ProtocolError, DecryptStringError, DataCorruptionError,
                             WrongTagError, DecryptFloatError)

STRING = "[STRING]"
END = "[END]"
DATA = "[DATA]"
TAG = "[TAG]"
FLOAT = "[FLOAT]"


def decrypt(bts: bytes) -> SupportedTypes:
    """
    Decodes bytes into an object of one of the supported types
    :param bts: bytes representation of some object
    :return: an object of one of the supported types
    :raise EmptyDataError: if no data can be decoded
    :raise ProtocolError: if a protocol version does not match the current one
    :raise BytesLeftError: if not all bytes was parsed
    """
    if len(bts) <= 1:
        raise EmptyDataError("Nothing to decrypt")
    if bts[0] != PROTOCOL_VERSION:
        raise ProtocolError(f"Supported protocol version is less or equal {PROTOCOL_VERSION}")
    try:
        result, offset = real_decrypt(bts, 1)
    except ValueError as e:
        str_error = str(e)
        if STRING in str_error:
            raise DecryptStringError(str_error.replace(STRING, "")) from None
        if END in str_error or DATA in str_error:
            raise DataCorruptionError(str_error.replace(END, "").replace(DATA, "")) from None
        if TAG in str_error:
            raise WrongTagError(str_error.replace(TAG, "")) from None
        if FLOAT in str_error:
            raise DecryptFloatError(str_error.replace(FLOAT, "")) from None
        raise
    diff = len(bts) - offset - 1
    if diff > 0:
        raise BytesLeftError(f"Corrupt data, finish on parse bytes {offset}, but still have {diff} bytes unparsed")
    return result

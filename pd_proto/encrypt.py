from pd_proto import real_encrypt
from pd_proto.const import (PROTOCOL_VERSION, SupportedTypes, FLOAT_LIMIT, STRING_LIMIT, DEPTH_LIMIT)
from pd_proto.errors import CycleLinksError, UnsupportedTypeError


def encrypt(data: SupportedTypes, max_depth: int = DEPTH_LIMIT, float_limit: float = FLOAT_LIMIT,
            string_length_limit: int = STRING_LIMIT) -> bytes:
    """
    Converts a supported Python type into a sequence of bytes
    :param data: an object of any allowed type
    :param max_depth: maximum nesting depth for collections, raise an error if exceeded. Use 0 to disable it, but it
    can lead to error
    :param float_limit: limit for float optimisation, if float less than that value, pd_proto will try to optimize it.
    Use 0 to disable optimisation
    :param string_length_limit: limit for string optimisation, if string greater than that value, pd_proto will try
    to compress it. Use 0 to disable optimisation
    :return: bytes representation of data
    :raises UnsupportedTypeError when type is not supported
    :raises CycleLinksError when collection contains link on self
    """
    try:
        result = real_encrypt(data, PROTOCOL_VERSION, max_depth, float_limit, string_length_limit)
    except AttributeError:
        raise UnsupportedTypeError("Value of unsupported type inside data") from None
    except ValueError:
        raise CycleLinksError("Cannot encrypt collections with link cycle") from None
    return result

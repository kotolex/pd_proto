from pd_proto import real_encrypt
from pd_proto.const import (PROTOCOL_VERSION, SupportedTypes)
from pd_proto.errors import CycleLinksError, UnsupportedTypeError


def encrypt(data: SupportedTypes) -> bytes:
    """
    Converts a supported Python type into a sequence of bytes
    :param data: an object of any allowed type
    :return: bytes representation of data
    :raises UnsupportedTypeError when type is not supported
    """
    try:
        result = real_encrypt(data, PROTOCOL_VERSION)
    except AttributeError:
        raise UnsupportedTypeError("Value of unsupported type inside data") from None
    except ValueError:
        raise CycleLinksError("Cannot encrypt collections with link cycle") from None
    return result

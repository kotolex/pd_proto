from pd_proto import pack
from pd_proto.const import (PROTOCOL_VERSION, SupportedTypes, FLOAT_LIMIT, STRING_LIMIT, DEPTH_LIMIT, MIN_INT, MAX_INT)
from pd_proto.errors import CycleLinksError, UnsupportedTypeError, PDProtoError, IntegerOutOfBoundsError


def dumps(data: SupportedTypes, max_depth: int = DEPTH_LIMIT, float_limit: float = FLOAT_LIMIT,
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
        result = pack(data, PROTOCOL_VERSION, max_depth, float_limit, string_length_limit)
    except AttributeError as e:
        type_name = str(e).split("-")[1]
        raise UnsupportedTypeError(f"Value of unsupported type inside data: {type_name}") from None
    except ValueError:
        raise CycleLinksError("Cannot encrypt collections with link cycle") from None
    except OverflowError:
        raise IntegerOutOfBoundsError(f"Data contains integers which is not in range [{MIN_INT}; {MAX_INT}]") from None
    except BaseException as e: # Rust panic error will be caught here
        if isinstance(e, (KeyboardInterrupt, SystemExit)):
            raise
        raise PDProtoError("Unexpected error on encrypting data! Please check your data is correct and report an issue here") from e # TODO git rep
    return result

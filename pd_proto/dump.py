from pd_proto import pack
from pd_proto.const import (PROTOCOL_VERSION, SupportedTypes, FLOAT_LIMIT, STRING_LIMIT, DEPTH_LIMIT, MIN_INT, MAX_INT)
from pd_proto.errors import CycleLinksError, UnsupportedTypeError, PDProtoError, IntegerOutOfBoundsError


def dumps(data: SupportedTypes, max_depth: int = DEPTH_LIMIT, float_limit: float = FLOAT_LIMIT,
          string_length_limit: int = STRING_LIMIT) -> bytes:
    """
    Converts a supported Python type into a sequence of bytes.

    :param data: An object of any allowed type.
    :param max_depth: Maximum nesting depth for collections; raises an error if exceeded.
                      Set to 0 to disable this check (warning: can lead to errors/stack overflow).
    :param float_limit: Threshold for float optimization. If a float is less than this value,
                        pd_proto will attempt to optimize it. Set to 0 to disable optimization.
    :param string_length_limit: Threshold for string optimization. If a string is longer than this value,
                                pd_proto will attempt to compress it. Set to 0 to disable optimization.
    :return: A bytes representation of the data.
    :raises UnsupportedTypeError: If the type is not supported.
    :raises CycleLinksError: If a collection contains a circular reference (link to itself).
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

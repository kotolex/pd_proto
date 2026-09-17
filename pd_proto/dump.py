from pd_proto import _pack, _packf
from pd_proto.const import (DEPTH_LIMIT, ENCODING, FLOAT_LIMIT, MAX_INT,
                            MIN_INT, PROTOCOL_VERSION, STRING_LIMIT, WRITABLE,
                            SupportedTypes, SupportsWrite)
from pd_proto.errors import (BinaryFileError, CycleLinksError,
                             IntegerOutOfBoundsError, PDProtoError,
                             UnsupportedTypeError)


def _dump(use_file: bool, *args):
    action = _packf if use_file else _pack
    try:
        result = action(*args)
    except AttributeError as e:
        type_name = str(e).split("-")[1]
        raise UnsupportedTypeError(f"Value of unsupported type inside data: {type_name}") from None
    except ValueError:
        raise CycleLinksError("Cannot encrypt collections with link cycle") from None
    except OverflowError:
        raise IntegerOutOfBoundsError(f"Data contains integers which is not in range [{MIN_INT}; {MAX_INT}]") from None
    except BaseException as e:  # Rust panic error will be caught here
        if isinstance(e, (KeyboardInterrupt, SystemExit)):
            raise
        raise PDProtoError("""Unexpected error on encrypting data!\n
        Please check your data is correct and report an issue here https://github.com/kotolex/pd_proto/issues""") from e
    return result


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
    return _dump(False, data, PROTOCOL_VERSION, max_depth, float_limit, string_length_limit)


def dump(file: SupportsWrite, data: SupportedTypes, max_depth: int = DEPTH_LIMIT, float_limit: float = FLOAT_LIMIT,
         string_length_limit: int = STRING_LIMIT) -> None:
    """
    Serializes a supported Python object into a byte sequence and writes it directly to a file using efficient
    buffered I/O.

    :param file: A file-like object opened in binary write mode ('wb') that supports buffered writing.
    :param data: The Python object of any supported type to serialize.
    :param max_depth: Maximum nesting depth for collections; raises an error if exceeded.
                      Set to 0 to disable this check (warning: can lead to errors/stack overflow).
    :param float_limit: Threshold for float optimization. If a float is less than this value,
                        pd_proto will attempt to optimize it. Set to 0 to disable optimization.
    :param string_length_limit: Threshold for string optimization. If a string is longer than this value,
                                pd_proto will attempt to compress it. Set to 0 to disable optimization.
    :raises UnsupportedTypeError: If the object type is not supported.
    :raises CycleLinksError: If a collection contains a circular reference (link to itself).
    :raises BinaryFileError: If there are any issues related to the binary file.
    """
    if hasattr(file, ENCODING):
        raise BinaryFileError("Param 'file' must be a binary file opened for writing (unexpected 'encoding' attribute)")
    if not getattr(file, WRITABLE, lambda: False)():
        raise BinaryFileError("The file is not opened for writing or lacks 'writable()' method.")
    _dump(True, file, data, PROTOCOL_VERSION, max_depth, float_limit, string_length_limit)

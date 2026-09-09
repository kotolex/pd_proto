class PDProtoError(Exception):
    """
    Parent for all types of errors in pd_proto, so you can use broader error in except clauses

    try:
        code() # some parsing
    except PDProtoError:
        report() # do something on error
    """


class UnsupportedTypeError(PDProtoError):
    """
    Raises when the type of object is not supported
    """


class EmptyDataError(PDProtoError):
    """
    Raises when the data is empty
    """


class ProtocolError(PDProtoError):
    """
    Raises when the protocol is wrong
    """


class DataCorruptionError(PDProtoError):
    """
    Raises when something wrong with encrypted data: not enough bytes to parse, bytes left after parsing, etc.
    """


class WrongTagError(DataCorruptionError):
    """
    Raises when the tag is wrong, which often mean data corrupted
    """


class ParseFloatError(DataCorruptionError):
    """
    Raises when the data is invalid and float value cannot be parsed
    """


class ParseStringError(DataCorruptionError):
    """
    Raises when the data is invalid and string value cannot be parsed
    """


class BytesLeftError(DataCorruptionError):
    """
    Raises when the parsing is over, but still have bytes left
    """


class CycleLinksError(DataCorruptionError):
    """
    Raises when there is a cycle in parsing and recursion limit exceeded
    """

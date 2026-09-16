class PDProtoError(Exception):
    """
    Base class for all errors in pd_proto.

    Can be used in try-except blocks to catch any library-specific exception.

    Example:
        try:
            code()  # some parsing
        except PDProtoError:
            report()  # handle the error
    """


class UnsupportedTypeError(PDProtoError):
    """
    Raised when the object type is not supported.
    """


class BinaryFileError(PDProtoError):
    """
    Raised for any issues related to the binary file.
    """


class EmptyDataError(PDProtoError):
    """
    Raised when the input data is empty.
    """


class ProtocolError(PDProtoError):
    """
    Raised when the protocol version is invalid or mismatched.
    """


class IntegerOutOfBoundsError(PDProtoError):
    """
    Raised when an integer exceeds the maximum or minimum allowed limit.
    """


class DataCorruptionError(PDProtoError):
    """
    Raised when data corruption is detected (e.g., missing bytes, unexpected trailing bytes).
    """


class WrongTagError(DataCorruptionError):
    """
    Raised when an invalid tag is encountered, which usually indicates data corruption.
    """


class ParseFloatError(DataCorruptionError):
    """
    Raised when a float value cannot be parsed due to invalid data.
    """


class ParseStringError(DataCorruptionError):
    """
    Raised when a string value cannot be parsed due to invalid data.
    """


class BytesLeftError(DataCorruptionError):
    """
    Raised when unparsed bytes remain after processing is complete.
    """


class CycleLinksError(DataCorruptionError):
    """
    Raised when a circular reference is detected and the nesting limit is exceeded.
    """

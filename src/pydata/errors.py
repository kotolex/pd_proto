class PyDataError(Exception):
    """
    Parent for all types of errors in pydata, so you can use broader error in except clauses

    try:
        code() # some parsing
    except PyDataError:
        report() # do something on error
    """


class UnsupportedTypeError(PyDataError):
    """
    Raises when the type of object is not supported
    """


class WrongCodeError(PyDataError):
    """
    Raises when the code for supported type is wrong
    """


class EmptyDataError(PyDataError):
    """
    Raises when the data is empty
    """


class ProtocolError(PyDataError):
    """
    Raises when the protocol is wrong or data is corrupted
    """


class WrongTagError(ProtocolError):
    """
    Raises when the tag is wrong, which often mean data corrupted
    """


class DecryptFloatError(ProtocolError):
    """
    Raises when the data is invalid and float value cannot be decrypted
    """


class DecryptStringError(ProtocolError):
    """
    Raises when the data is invalid and string value cannot be decrypted
    """


class BytesLeftError(ProtocolError):
    """
    Raises when the parsing is over, but still have bytes left
    """

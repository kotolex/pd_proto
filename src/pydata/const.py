from enum import IntEnum

from src.pydata.errors import UnsupportedTypeError, WrongCodeError

PROTOCOL_VERSION = 1
FLOAT_FORMAT = ">d"
UTF_8 = "utf-8"

SupportedTypes = None | bool | int | float | str


class SupportedType(IntEnum):
    """
    Supported types and their codes in the resulting encoding
    """
    NULL = 0
    BOOL_TRUE = 1
    BOOL_FALSE = 2
    FLOAT_ZER0 = 3
    STRING_EMPTY = 4
    LIST_EMPTY = 5
    TUPLE_EMPTY = 6
    SET_EMPTY = 7
    DICT_EMPTY = 8
    INT_POSITIVE = 10
    INT_NEGATIVE = 11
    FLOAT = 12
    STRING = 13

    @classmethod
    def all_codes(cls):
        """
        Return all supported codes
        """
        return [item.value for item in cls]


def int_by_type(value: SupportedTypes):
    """
    Returns an integer representing the code of each data type allowed for encryption
    :param value: an object of an supported type
    :return: the code for conversion
    :raise UnsupportedTypeError: if the type is not allowed
    """
    if value is None:
        return SupportedType.NULL.value
    if value is True:
        return SupportedType.BOOL_TRUE.value
    if value is False:
        return SupportedType.BOOL_FALSE.value
    if isinstance(value, int):
        if value >= 0:
            return SupportedType.INT_POSITIVE.value
        return SupportedType.INT_NEGATIVE.value
    if isinstance(value, float):
        if value == 0.0:
            return SupportedType.FLOAT_ZER0.value
        return SupportedType.FLOAT.value
    if isinstance(value, str):
        if not value:
            return SupportedType.STRING_EMPTY.value
        return SupportedType.STRING.value
    raise UnsupportedTypeError(f"Value of unsupported type -{value}-: {type(value)}")


def type_by_int(code: int) -> SupportedTypes:
    """
    Returns an object by its code; note that it returns the object itself of that type, not the type
    :param code: the object code
    :return: an object of this type
    :raise WrongCodeError: if the code is invalid
    """
    if code == SupportedType.NULL.value:
        return None
    if code == SupportedType.BOOL_TRUE.value:
        return True
    if code == SupportedType.BOOL_FALSE.value:
        return False
    if code == SupportedType.INT_POSITIVE.value:
        return 1
    if code == SupportedType.INT_NEGATIVE.value:
        return -1
    if code == SupportedType.FLOAT.value:
        return 1.0
    if code == SupportedType.FLOAT_ZER0.value:
        return 0.0
    if code in (SupportedType.STRING_EMPTY.value, SupportedType.STRING.value):
        return ""
    raise WrongCodeError(f"Wrong code, you can use one of {SupportedType.all_codes()}")

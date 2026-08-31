from enum import IntEnum

from src.pydata.errors import UnsupportedTypeError, WrongCodeError

PROTOCOL_VERSION = 1
FLOAT_FORMAT = ">d"
UTF_8 = "utf-8"

SupportedTypes = None | bool | int | float | str | list | tuple | dict | set
SupportedCollections = list | tuple | dict | set


class Variant(IntEnum):
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
    INT_ZERO = 9
    INT_POSITIVE = 10
    INT_NEGATIVE = 11
    FLOAT = 12
    STRING = 13
    LIST = 14
    TUPLE = 15
    SET = 16
    DICT = 17

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
        return Variant.NULL.value
    if value is True:
        return Variant.BOOL_TRUE.value
    if value is False:
        return Variant.BOOL_FALSE.value
    if isinstance(value, int):
        if value == 0:
            return Variant.INT_ZERO.value
        if value > 0:
            return Variant.INT_POSITIVE.value
        return Variant.INT_NEGATIVE.value
    if isinstance(value, float):
        if value == 0.0:
            return Variant.FLOAT_ZER0.value
        return Variant.FLOAT.value
    if isinstance(value, str):
        if not value:
            return Variant.STRING_EMPTY.value
        return Variant.STRING.value
    if isinstance(value, list):
        if not value:
            return Variant.LIST_EMPTY.value
        return Variant.LIST.value
    if isinstance(value, tuple):
        if not value:
            return Variant.TUPLE_EMPTY.value
        return Variant.TUPLE.value
    if isinstance(value, set):
        if not value:
            return Variant.SET_EMPTY.value
        return Variant.SET.value
    if isinstance(value, dict):
        if not value:
            return Variant.DICT_EMPTY.value
        return Variant.DICT.value
    raise UnsupportedTypeError(f"Value of unsupported type -{value}-: {type(value)}")


def obj_by_code(code: int) -> SupportedTypes:
    """
    Returns an object by its code; note that it returns the object itself of that type, not the type
    :param code: the object code
    :return: an object of this type
    :raise WrongCodeError: if the code is invalid
    """
    if code == Variant.NULL.value:
        return None
    if code == Variant.BOOL_TRUE.value:
        return True
    if code == Variant.BOOL_FALSE.value:
        return False
    if code == Variant.INT_ZERO.value:
        return 0
    if code == Variant.INT_POSITIVE.value:
        return 1
    if code == Variant.INT_NEGATIVE.value:
        return -1
    if code == Variant.FLOAT.value:
        return 1.0
    if code == Variant.FLOAT_ZER0.value:
        return 0.0
    if code in (Variant.STRING_EMPTY.value, Variant.STRING.value):
        return ""
    if code in (Variant.LIST_EMPTY.value, Variant.LIST.value):
        return []
    if code in (Variant.TUPLE_EMPTY.value, Variant.TUPLE.value):
        return tuple()
    if code in (Variant.SET_EMPTY.value, Variant.SET.value):
        return set()
    if code in (Variant.DICT_EMPTY.value, Variant.DICT.value):
        return {}
    raise WrongCodeError(f"Wrong code, you can use one of {Variant.all_codes()}")

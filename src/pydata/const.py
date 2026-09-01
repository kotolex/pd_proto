from enum import IntEnum

PROTOCOL_VERSION = 1
FLOAT_FORMAT = ">d"
FLOAT_LIMIT = 268_435_455
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
    FLOAT_NO_DECIMALS = 20
    FLOAT_1 = 21
    FLOAT_2 = 22
    FLOAT_3 = 23
    FLOAT_4 = 24
    FLOAT_5 = 25
    FLOAT_6 = 26


def tag_by_decimal_places(dec_places: int) -> int:
    """
    Return tag for float based on decimal places
    :param dec_places: number of digits after decimal point
    """
    match dec_places:
        case 0:
            return Variant.FLOAT_NO_DECIMALS.value
        case 1:
            return Variant.FLOAT_1.value
        case 2:
            return Variant.FLOAT_2.value
        case 3:
            return Variant.FLOAT_3.value
        case 4:
            return Variant.FLOAT_4.value
        case 5:
            return Variant.FLOAT_5.value
        case 6:
            return Variant.FLOAT_6.value
    raise ValueError(f"Unsupported decimal places {dec_places}")

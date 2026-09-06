from enum import IntEnum

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
    DICT = 17  # <- 18, 19
    FLOAT_NO_DECIMALS = 20
    FLOAT_1 = 21
    FLOAT_2 = 22
    FLOAT_3 = 23
    FLOAT_4 = 24
    FLOAT_5 = 25
    FLOAT_6 = 26  # <- 27, 28, 29
    STRING_COMPRESSED = 30
    STRING_1 = 31
    STRING_2 = 32
    STRING_3 = 33
    STRING_4 = 34
    STRING_5 = 35
    STRING_6 = 36
    STRING_7 = 37
    STRING_8 = 38
    STRING_9 = 39
    STRING_10 = 40
    STRING_11 = 41
    STRING_12 = 42
    STRING_13 = 43
    STRING_14 = 44
    STRING_15 = 45

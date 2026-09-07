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
    FLOAT_NO_DECIMALS_NEG = 30
    FLOAT_1_NEG = 31
    FLOAT_2_NEG = 32
    FLOAT_3_NEG = 33
    FLOAT_4_NEG = 34
    FLOAT_5_NEG = 35
    FLOAT_6_NEG = 36  # <- 37, 38, 39
    STRING_COMPRESSED = 40
    STRING_1 = 41
    STRING_2 = 42
    STRING_3 = 43
    STRING_4 = 44
    STRING_5 = 45
    STRING_6 = 46
    STRING_7 = 47
    STRING_8 = 48
    STRING_9 = 49
    STRING_10 = 50
    STRING_11 = 51
    STRING_12 = 52
    STRING_13 = 53
    STRING_14 = 54
    STRING_15 = 55

from enum import IntEnum
from datetime import datetime

PROTOCOL_VERSION = 1
FLOAT_LIMIT = 268_435_455.0
STRING_LIMIT = 100
DEPTH_LIMIT = 1000
MIN_INT = -9_223_372_036_854_775_808
MAX_INT = 9_223_372_036_854_775_807

SupportedTypes = None | bool | int | float | str | bytes | list | tuple | dict | set | datetime
SupportedCollections = list | tuple | dict | set


class Variant(IntEnum):
    """
    Supported types and their codes(tags) in the resulting data
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
    BYTES_EMPTY = 18
    BYTES = 19
    FLOAT_NO_DECIMALS = 20
    FLOAT_1 = 21
    FLOAT_2 = 22
    FLOAT_3 = 23
    FLOAT_4 = 24
    FLOAT_5 = 25
    FLOAT_6 = 26
    DATE_TIME_NO_TZ = 27
    DATE_TIME_OFFSET = 28
    DATE_TIME_IANA = 29
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
    TUPLE_2 = 56
    TUPLE_3 = 57
    TUPLE_4 = 58
    TUPLE_5 = 59
    INT_1000 = 60
    INT_1 = 61
    INT_2 = 62
    INT_3 = 63
    INT_4 = 64
    INT_5 = 65
    INT_6 = 66
    INT_7 = 67
    INT_8 = 68
    INT_9 = 69
    INT_10 = 70
    INT_11 = 71
    INT_12 = 72
    INT_13 = 73
    INT_15 = 74
    INT_20 = 75
    INT_24 = 76
    INT_50 = 77
    INT_100 = 78 # <- 79, 80
    LIST_1 = 81
    LIST_2 = 82
    LIST_3 = 83
    LIST_4 = 84
    LIST_5 = 85
    LIST_6 = 86
    LIST_7 = 87
    LIST_8 = 88
    LIST_9 = 89
    LIST_10 = 90

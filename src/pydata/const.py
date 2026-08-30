from enum import IntEnum

PROTOCOL_VERSION = 1

AllowedTypes = None | bool | int


class Type(IntEnum):
    NULL = 0,
    BOOL_TRUE = 1,
    BOOL_FALSE = 2,
    INT_POSITIVE = 3,
    INT_NEGATIVE = 4,

    # FLOAT = 5,
    # STRING=6,
    # LIST=7,
    # TUPLE=8,
    # SET=9
    # DICT=10


def int_by_type(value: AllowedTypes):
    if value is None:
        return 0
    if value is True:
        return 1
    if value is False:
        return 2
    if isinstance(value, int):
        if value >= 0:
            return 3
        else:
            return 4
    raise ValueError(f"Value of unsupported type {value}: {type(value)}")


def type_by_int(b: int) -> AllowedTypes:
    if b == Type.NULL.value:
        return None
    elif b == Type.BOOL_TRUE.value:
        return True
    elif b == Type.BOOL_FALSE.value:
        return False
    elif b == Type.INT_POSITIVE.value:
        return 1
    elif b == Type.INT_NEGATIVE.value:
        return -1
    raise ValueError("Corrupt data")
